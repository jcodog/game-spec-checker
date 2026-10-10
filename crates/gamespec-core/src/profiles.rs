use crate::*;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

pub fn sample() -> GameProfile {
    parse(include_str!("../../../data/icarus.json")).expect("valid bundled Icarus profile")
}
pub fn empty_requirements() -> Requirements {
    Requirements {
        cpu: vec![],
        gpu: vec![],
        ram: None,
        vram: None,
        windows_versions: vec![],
        architecture: None,
        graphics_api: None,
        feature_level: None,
        storage: None,
        ssd: SsdPolicy::Unspecified,
    }
}
pub fn new_profile(id: String) -> GameProfile {
    GameProfile {
        schema_version: 2,
        id,
        name: String::new(),
        version: String::new(),
        platform: "Windows PC".into(),
        provenance: Provenance {
            kind: ProvenanceKind::UserEntered,
            source_url: None,
            verified_date: None,
        },
        cpu_workload: Workload::CpuGaming,
        gpu_workload: Workload::GpuRaster,
        minimum: empty_requirements(),
        recommended: empty_requirements(),
        installation_headroom: None,
        notes: String::new(),
        migration_notes: vec![],
    }
}
pub fn valid_id(id: &str) -> bool {
    let upper = id.to_ascii_uppercase();
    !id.is_empty()
        && id.len() <= 80
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        && !["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        && !(1..=9).any(|n| upper == format!("COM{n}") || upper == format!("LPT{n}"))
}
pub fn valid_date(date: &str) -> bool {
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3 || parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 0,
    };
    (1990..=2100).contains(&y) && d > 0 && d <= days
}
pub fn valid_url(url: &str) -> bool {
    (url.starts_with("https://") || url.starts_with("http://"))
        && url.len() <= 2048
        && !url.chars().any(|c| c.is_whitespace() || c.is_control())
        && url
            .split("//")
            .nth(1)
            .is_some_and(|host| !host.is_empty() && !host.starts_with('/'))
}
pub fn validate_capacity(value: Capacity) -> Result<(), String> {
    if !value.value.is_finite() || !(0.0..=100000.0).contains(&value.value) {
        return Err("Capacity must be finite, nonnegative and at most 100,000.".into());
    }
    Ok(())
}
pub fn validate(p: &GameProfile) -> Result<(), String> {
    if p.schema_version != 2 {
        return Err("Unsupported schema. Import schema 1 through the migration parser.".into());
    }
    if !valid_id(&p.id) {
        return Err("Invalid or reserved Windows profile ID.".into());
    }
    if p.name.trim().is_empty()
        || p.name.len() > 200
        || p.version.len() > 200
        || p.platform.trim().is_empty()
        || p.platform.len() > 200
        || p.notes.len() > 20000
        || p.migration_notes.len() > 20
        || p.migration_notes.iter().any(|n| n.len() > 2000)
    {
        return Err("Enter a name/platform and keep profile text within supported limits.".into());
    }
    if p.provenance
        .source_url
        .as_deref()
        .is_some_and(|u| !valid_url(u))
        || p.provenance
            .verified_date
            .as_deref()
            .is_some_and(|d| !valid_date(d))
    {
        return Err("Enter a valid source URL and ISO date, YYYY-MM-DD.".into());
    }
    if p.provenance.kind == ProvenanceKind::Official
        && (p.provenance.source_url.is_none() || p.provenance.verified_date.is_none())
    {
        return Err("An official-source label requires its URL and verification date. The app does not independently authenticate this label.".into());
    }
    if !matches!(
        p.cpu_workload,
        Workload::CpuGaming | Workload::CpuSingleThread | Workload::CpuMultiThread
    ) || !matches!(
        p.gpu_workload,
        Workload::GpuRaster
            | Workload::GpuRayTracing
            | Workload::GpuSynthetic
            | Workload::GpuCompute
    ) {
        return Err("CPU and GPU evidence workloads must match their component.".into());
    }
    for req in [&p.minimum, &p.recommended] {
        for capacity in [req.ram, req.vram, req.storage].into_iter().flatten() {
            validate_capacity(capacity)?;
        }
        for models in [&req.cpu, &req.gpu] {
            if models.len() > 30 || models.iter().any(|s| s.trim().is_empty() || s.len() > 250) {
                return Err("Use up to 30 nonempty complete hardware alternatives.".into());
            }
        }
        if req.windows_versions.len() > 2
            || req.windows_versions.iter().any(|v| ![10, 11].contains(v))
        {
            return Err("Supported Windows requirements are 10 and 11.".into());
        }
        if req
            .architecture
            .as_deref()
            .is_some_and(|s| !["x86_64", "x86", "aarch64"].contains(&s))
            || req
                .graphics_api
                .as_ref()
                .is_some_and(|s| s.trim().is_empty() || s.len() > 80)
            || req
                .feature_level
                .as_deref()
                .is_some_and(|s| !["11_0", "11_1", "12_0", "12_1", "12_2"].contains(&s))
        {
            return Err("Invalid architecture, graphics API or feature level.".into());
        }
    }
    for (min, rec) in [
        (p.minimum.ram, p.recommended.ram),
        (p.minimum.vram, p.recommended.vram),
        (p.minimum.storage, p.recommended.storage),
    ] {
        if let (Some(a), Some(b)) = (min, rec) {
            if b.bounds().1 < a.bounds().0 {
                return Err("Recommended capacity cannot be below minimum.".into());
            }
        }
    }
    if p.minimum.ssd == SsdPolicy::Required && p.recommended.ssd != SsdPolicy::Required {
        return Err("An SSD required at minimum remains required at recommended.".into());
    }
    if let Some(headroom) = p.installation_headroom {
        validate_capacity(headroom)?;
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyRequirements {
    cpu: Vec<String>,
    gpu: Vec<String>,
    ram_gib: f64,
    vram_gib: f64,
    windows_versions: Vec<u32>,
    architecture: String,
    graphics_api: String,
    feature_level: Option<String>,
    storage_gb: f64,
    ssd: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyProfile {
    schema_version: u32,
    id: String,
    name: String,
    version: String,
    minimum: LegacyRequirements,
    recommended: LegacyRequirements,
    notes: String,
}
fn migrate_req(r: LegacyRequirements, required_ssd: bool) -> Requirements {
    Requirements {
        cpu: r.cpu,
        gpu: r.gpu,
        ram: Some(Capacity {
            value: r.ram_gib,
            unit: CapacityUnit::Gib,
        }),
        vram: Some(Capacity {
            value: r.vram_gib,
            unit: CapacityUnit::Gib,
        }),
        windows_versions: r.windows_versions,
        architecture: Some(r.architecture),
        graphics_api: Some(r.graphics_api),
        feature_level: r.feature_level,
        storage: Some(Capacity {
            value: r.storage_gb,
            unit: CapacityUnit::Gb,
        }),
        ssd: if required_ssd {
            SsdPolicy::Required
        } else if r.ssd {
            SsdPolicy::Recommended
        } else {
            SsdPolicy::Unspecified
        },
    }
}
pub fn parse(json: &str) -> Result<GameProfile, String> {
    if json.len() > 1_000_000 {
        return Err("Profile exceeds 1 MB.".into());
    }
    let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let p: GameProfile = match value.get("schema_version").and_then(|v| v.as_u64()) {
        Some(2) => serde_json::from_value(value).map_err(|e| e.to_string())?,
        Some(1) => {
            let old: LegacyProfile = serde_json::from_value(value).map_err(|e| e.to_string())?;
            if old.schema_version != 1 {
                return Err("Invalid legacy version.".into());
            }
            let required_ssd = old.minimum.ssd;
            let mut p = new_profile(old.id);
            p.name = old.name;
            p.version = old.version;
            p.notes = old.notes;
            p.minimum = migrate_req(old.minimum, required_ssd);
            p.recommended = migrate_req(old.recommended, required_ssd);
            p.provenance.kind = ProvenanceKind::Unverified;
            p.migration_notes.push("Migrated from schema 1. GiB values were preserved; review the publisher's original units. Earlier Add game copied Icarus requirements, so confirm every requirement and source. No requirements were silently corrected.".into());
            p
        }
        _ => return Err("Unsupported profile schema version.".into()),
    };
    validate(&p)?;
    Ok(p)
}
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
pub fn atomic_write(target: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = target.parent().ok_or("Missing parent directory.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp = parent.join(format!(
        ".gamespec-{}-{}.tmp",
        std::process::id(),
        TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        fs::write(&temp, bytes).map_err(|e| e.to_string())?;
        fs::rename(&temp, target).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
pub fn save(dir: &Path, p: &GameProfile) -> Result<(), String> {
    validate(p)?;
    atomic_write(
        &dir.join(format!("{}.json", p.id)),
        &serde_json::to_vec_pretty(p).map_err(|e| e.to_string())?,
    )
}
#[derive(Debug, Serialize)]
pub struct ProfileLibrary {
    pub profiles: Vec<GameProfile>,
    pub diagnostics: Vec<String>,
}
pub fn load(dir: &Path) -> Result<ProfileLibrary, String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if !dir.join(".initialized").exists() {
        if !dir.join("icarus.json").exists() {
            save(dir, &sample())?;
        }
        atomic_write(&dir.join(".initialized"), b"schema2")?;
    }
    let mut library = ProfileLibrary {
        profiles: vec![],
        diagnostics: vec![],
    };
    for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().is_some_and(|s| s == "json") {
            let result = (|| {
                if fs::metadata(&path).map_err(|e| e.to_string())?.len() > 1_000_000 {
                    return Err("Profile exceeds 1 MB.".into());
                }
                parse(&fs::read_to_string(&path).map_err(|e| e.to_string())?)
            })();
            let result = result.and_then(|p| {
                if path.file_stem().and_then(|s|s.to_str()) != Some(p.id.as_str()) {
                    return Err("Profile ID does not match its saved filename. Import this file for review.".into());
                }
                Ok(p)
            });
            match result {
                Ok(p) => library.profiles.push(p),
                Err(e) => library.diagnostics.push(format!("{}: {e}", path.display())),
            }
        }
    }
    library
        .profiles
        .sort_by_cached_key(|p| p.name.to_lowercase());
    Ok(library)
}
