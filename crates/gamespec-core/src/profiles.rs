use crate::GameProfile;
use std::{fs, path::Path};

pub fn sample() -> GameProfile {
    serde_json::from_str(include_str!("../../../data/icarus.json"))
        .expect("valid bundled Icarus profile")
}
pub fn validate(profile: &GameProfile) -> Result<(), String> {
    if profile.schema_version != 1 {
        return Err("Unsupported profile schema version. Expected 1.".into());
    }
    if profile.id.is_empty()
        || profile.id.len() > 80
        || !profile
            .id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    {
        return Err(
            "Profile ID must contain 1 to 80 letters, digits, hyphens or underscores.".into(),
        );
    }
    let uppercase_id = profile.id.to_ascii_uppercase();
    if ["CON", "PRN", "AUX", "NUL"].contains(&uppercase_id.as_str())
        || (1..=9).any(|n| uppercase_id == format!("COM{n}") || uppercase_id == format!("LPT{n}"))
    {
        return Err("Profile ID is a reserved Windows device name.".into());
    }
    if profile.name.trim().is_empty()
        || profile.name.len() > 200
        || profile.notes.len() > 20000
        || profile.version.len() > 200
    {
        return Err(
            "Enter a game name, up to 200 characters, and keep notes under 20,000 characters."
                .into(),
        );
    }
    for req in [&profile.minimum, &profile.recommended] {
        for value in [req.ram_gib, req.vram_gib, req.storage_gb] {
            if !value.is_finite() || !(0.0..=100000.0).contains(&value) {
                return Err(
                    "Memory and storage values must be finite, nonnegative and at most 100,000."
                        .into(),
                );
            }
        }
        for models in [&req.cpu, &req.gpu] {
            if models.is_empty()
                || models.len() > 30
                || models.iter().any(|s| s.trim().is_empty() || s.len() > 250)
            {
                return Err(
                    "Enter 1 to 30 CPU/GPU alternatives, each under 250 characters.".into(),
                );
            }
        }
        if req.windows_versions.is_empty()
            || req.windows_versions.iter().any(|v| ![10, 11].contains(v))
        {
            return Err("Supported Windows versions are 10 and 11.".into());
        }
        if !["x86_64", "x86", "aarch64"].contains(&req.architecture.as_str()) {
            return Err("Architecture must be x86_64, x86 or aarch64.".into());
        }
        if req.graphics_api.trim().is_empty() || req.graphics_api.len() > 80 {
            return Err("Enter a graphics API requirement.".into());
        }
        if req
            .feature_level
            .as_deref()
            .is_some_and(|s| !["11_0", "11_1", "12_0", "12_1", "12_2"].contains(&s))
        {
            return Err("Feature level must be 11_0, 11_1, 12_0, 12_1 or 12_2.".into());
        }
    }
    let m = &profile.minimum;
    let r = &profile.recommended;
    if r.ram_gib < m.ram_gib
        || r.vram_gib < m.vram_gib
        || r.storage_gb < m.storage_gb
        || (m.ssd && !r.ssd)
    {
        return Err(
            "Recommended numeric and SSD requirements cannot be lower than minimum.".into(),
        );
    }
    Ok(())
}
pub fn parse(json: &str) -> Result<GameProfile, String> {
    if json.len() > 1_000_000 {
        return Err("Profile exceeds 1 MB.".into());
    }
    let profile: GameProfile = serde_json::from_str(json).map_err(|e| e.to_string())?;
    validate(&profile)?;
    Ok(profile)
}
pub fn save(directory: &Path, profile: &GameProfile) -> Result<(), String> {
    validate(profile)?;
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let target = directory.join(format!("{}.json", profile.id));
    let temp = directory.join(format!("{}.json.tmp", profile.id));
    fs::write(
        &temp,
        serde_json::to_vec_pretty(profile).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::rename(temp, target).map_err(|e| e.to_string())
}
pub fn load(directory: &Path) -> Result<Vec<GameProfile>, String> {
    if !directory.exists() {
        return Ok(vec![sample()]);
    }
    let mut profiles = Vec::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|s| s == "json") {
            let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            profiles.push(parse(&text).map_err(|e| format!("{}: {e}", path.display()))?);
        }
    }
    if !profiles.iter().any(|p| p.id == "icarus") {
        profiles.push(sample());
    }
    profiles.sort_by_cached_key(|profile| profile.name.to_lowercase());
    Ok(profiles)
}
