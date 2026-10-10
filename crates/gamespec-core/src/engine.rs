use crate::{
    catalog::{compare, evidence},
    *,
};

fn numeric(actual: Option<u64>, required: f64, unit: f64, label: &str) -> Evidence {
    if required == 0.0 {
        return evidence(
            EvidenceResult::Pass,
            "No numeric requirement.",
            "Game profile",
        );
    }
    match actual {
        Some(value) => evidence(
            if value as f64 >= required * unit {
                EvidenceResult::Pass
            } else {
                EvidenceResult::Fail
            },
            &format!(
                "{:.2} detected; {:.2} required ({label}).",
                value as f64 / unit,
                required
            ),
            "Native API measurement",
        ),
        None => evidence(
            EvidenceResult::Unknown,
            "Measurement unavailable.",
            "Native scan",
        ),
    }
}
pub fn verdict(minimum: EvidenceResult, recommended: EvidenceResult) -> Verdict {
    match (minimum, recommended) {
        (EvidenceResult::Fail, _) => Verdict::BelowMinimum,
        (EvidenceResult::Unknown, _) => Verdict::Unknown,
        (EvidenceResult::Pass, EvidenceResult::Pass) => Verdict::MeetsRecommended,
        (EvidenceResult::Pass, EvidenceResult::Fail) => Verdict::MeetsMinimum,
        (EvidenceResult::Pass, EvidenceResult::Unknown) => Verdict::Unknown,
    }
}
fn aggregate(results: impl Iterator<Item = EvidenceResult>) -> EvidenceResult {
    let values: Vec<_> = results.collect();
    if values.contains(&EvidenceResult::Fail) {
        EvidenceResult::Fail
    } else if values.contains(&EvidenceResult::Unknown) {
        EvidenceResult::Unknown
    } else {
        EvidenceResult::Pass
    }
}
fn windows(os: &OperatingSystem, req: &Requirements) -> Evidence {
    if let (Some(edition), Some(build), Some(arch)) = (&os.edition, os.build, &os.architecture) {
        if !edition.contains("Windows") || edition.contains("Server") {
            return evidence(
                EvidenceResult::Fail,
                "A supported desktop Windows edition was not detected.",
                "WMI Win32_OperatingSystem",
            );
        }
        let version = if build >= 22000 {
            11
        } else if build >= 10240 {
            10
        } else {
            0
        };
        let pass = req.windows_versions.contains(&version) && arch == &req.architecture;
        evidence(
            if pass {
                EvidenceResult::Pass
            } else {
                EvidenceResult::Fail
            },
            &format!(
                "{edition}, build {build}, {arch}. Accepted versions: {:?}, {}.",
                req.windows_versions, req.architecture
            ),
            "WMI edition/build and native system architecture",
        )
    } else {
        evidence(
            EvidenceResult::Unknown,
            "Windows edition, build or architecture unavailable.",
            "Native scan",
        )
    }
}
fn graphics(gpu: Option<&Gpu>, req: &Requirements) -> Evidence {
    let Some(gpu) = gpu else {
        return evidence(
            EvidenceResult::Unknown,
            "Select a detected gaming GPU.",
            "Native scan",
        );
    };
    if !["DirectX 11", "DirectX 12"].contains(&req.graphics_api.as_str()) {
        return evidence(EvidenceResult::Unknown, &format!("{} is not probed. Direct3D feature levels cannot establish support for another graphics API.", req.graphics_api), "Native scanner API coverage");
    }
    let result = match req.graphics_api.as_str() {
        "DirectX 12" => match gpu.directx12 {
            Some(true) => EvidenceResult::Pass,
            Some(false) => EvidenceResult::Fail,
            None => EvidenceResult::Unknown,
        },
        "DirectX 11" if gpu.directx12 == Some(true) => EvidenceResult::Pass,
        _ => EvidenceResult::Unknown,
    };
    if let Some(required) = &req.feature_level {
        let levels = ["11_0", "11_1", "12_0", "12_1", "12_2"];
        let measured = gpu
            .feature_level
            .as_deref()
            .and_then(|s| levels.iter().position(|l| *l == s));
        let requested = levels.iter().position(|l| l == required);
        let fl = match (measured, requested) {
            (Some(a), Some(b)) => {
                if a >= b {
                    EvidenceResult::Pass
                } else {
                    EvidenceResult::Fail
                }
            }
            _ => EvidenceResult::Unknown,
        };
        return evidence(
            aggregate([result, fl].into_iter()),
            &format!(
                "{} required; GPU D3D12 probe: {:?}; feature level {:?}, required {required}.",
                req.graphics_api, gpu.directx12, gpu.feature_level
            ),
            "D3D12CreateDevice per adapter",
        );
    }
    evidence(result,&format!("{} required; D3D12 device support: {:?}; maximum probed D3D12 feature level: {:?}. API version and feature level are distinct.",req.graphics_api,gpu.directx12,gpu.feature_level),"D3D12CreateDevice per adapter; other APIs unverified")
}
fn ssd(volume: Option<&Volume>, required: bool) -> Evidence {
    if !required {
        return evidence(
            EvidenceResult::Pass,
            "SSD is not required at this level.",
            "Game profile",
        );
    }
    match volume.map(|v|&v.media) {
        Some(MediaKind::Ssd)=>evidence(EvidenceResult::Pass,"Backing volume is classified as SSD.","Native storage descriptor via sysinfo"),
        Some(MediaKind::Hdd)=>evidence(EvidenceResult::Fail,"SSD requested; backing storage is HDD.","Native storage descriptor via sysinfo"),
        _=>evidence(EvidenceResult::Unknown,"SSD requested; media type is unavailable. RAID, USB and virtual disks may be ambiguous.","Native storage descriptor via sysinfo"),
    }
}
pub fn assess(
    profile: &GameProfile,
    system: &SystemSnapshot,
    gpu_id: Option<&str>,
    destination: &str,
    volume: Option<Volume>,
) -> Assessment {
    let gpu = gpu_id.and_then(|id| system.gpus.iter().find(|g| g.id == id));
    let mut components = Vec::new();
    let min = &profile.minimum;
    let rec = &profile.recommended;
    let pairs = [
        (
            "CPU",
            compare("cpu", system.cpu.model.as_deref(), &min.cpu),
            compare("cpu", system.cpu.model.as_deref(), &rec.cpu),
        ),
        (
            "GPU",
            compare("gpu", gpu.map(|g| g.model.as_str()), &min.gpu),
            compare("gpu", gpu.map(|g| g.model.as_str()), &rec.gpu),
        ),
        (
            "RAM",
            numeric(system.ram_total_bytes, min.ram_gib, 1073741824.0, "GiB RAM"),
            numeric(system.ram_total_bytes, rec.ram_gib, 1073741824.0, "GiB RAM"),
        ),
        (
            "VRAM",
            numeric(
                gpu.and_then(|g| g.dedicated_bytes),
                min.vram_gib,
                1073741824.0,
                "GiB dedicated VRAM",
            ),
            numeric(
                gpu.and_then(|g| g.dedicated_bytes),
                rec.vram_gib,
                1073741824.0,
                "GiB dedicated VRAM",
            ),
        ),
        ("Graphics API", graphics(gpu, min), graphics(gpu, rec)),
        (
            "Windows",
            windows(&system.os, min),
            windows(&system.os, rec),
        ),
        (
            "Storage",
            numeric(
                volume.as_ref().map(|v| v.free_bytes),
                min.storage_gb,
                1e9,
                "GB free storage",
            ),
            numeric(
                volume.as_ref().map(|v| v.free_bytes),
                rec.storage_gb,
                1e9,
                "GB free storage",
            ),
        ),
        (
            "SSD",
            ssd(volume.as_ref(), min.ssd),
            ssd(volume.as_ref(), rec.ssd),
        ),
    ];
    for (component, minimum, recommended) in pairs {
        components.push(ComponentResult {
            component: component.into(),
            verdict: verdict(minimum.result, recommended.result),
            minimum,
            recommended,
        });
    }
    let overall = verdict(
        aggregate(components.iter().map(|c| c.minimum.result)),
        aggregate(components.iter().map(|c| c.recommended.result)),
    );
    let mut warnings=vec!["Requirements and benchmark approximations do not guarantee FPS, stability or game-specific feature support.".into()];
    if gpu.is_none() {
        warnings.push(
            "No gaming GPU selected. Choose one explicitly, especially on multi-GPU systems."
                .into(),
        );
    }
    if volume.is_none() {
        warnings.push("Installation location could not be mapped to a detected volume.".into());
    }
    if let Some(available) = system.ram_available_bytes {
        if (available as f64) < min.ram_gib * 1073741824.0 {
            warnings.push("Available RAM is below minimum. Close background applications; installed RAM is assessed separately.".into());
        }
    }
    if components
        .iter()
        .any(|c| c.component == "SSD" && c.recommended.result != EvidenceResult::Pass)
    {
        warnings
            .push("SSD is recommended or required, but suitable storage was not confirmed.".into());
    }
    let headroom_bytes = volume
        .as_ref()
        .map(|v| v.free_bytes as i128 - (rec.storage_gb * 1e9).ceil() as i128);
    if headroom_bytes.is_some_and(|b| b < 0) {
        warnings.push("Insufficient installation space on the selected volume.".into());
    }
    Assessment {
        profile: profile.clone(),
        system: system.clone(),
        gpu_id: gpu_id.map(String::from),
        destination: destination.into(),
        volume,
        headroom_bytes,
        verdict: overall,
        components,
        warnings,
        catalog_version: crate::catalog::catalog().version,
    }
}
pub fn recommend(
    report: &Assessment,
    width: u32,
    height: u32,
    preset: Option<&str>,
    observed: Option<f64>,
    target: Option<f64>,
) -> Vec<SettingRecommendation> {
    let default = match report.verdict {
        Verdict::MeetsRecommended => "High",
        Verdict::MeetsMinimum => "Medium",
        Verdict::BelowMinimum => "Very Low",
        Verdict::Unknown => "Low",
    };
    let selected = preset
        .filter(|p| ["Very Low", "Low", "Medium", "High", "Ultra"].contains(p))
        .unwrap_or(default);
    let struggling = matches!((observed,target),(Some(a),Some(b)) if a.is_finite() && b.is_finite() && a>0.0 && b>0.0 && a<b);
    let gpu = report
        .gpu_id
        .as_ref()
        .and_then(|id| report.system.gpus.iter().find(|g| &g.id == id));
    let vram = gpu
        .and_then(|g| g.dedicated_bytes)
        .map(|b| b as f64 / 1073741824.0);
    let textures = match vram {
        Some(n) if n >= 12.0 => "High",
        Some(n) if n >= 8.0 => "Medium",
        Some(n) if n >= 4.0 => "Low",
        _ => "Very Low",
    };
    let mut settings = Vec::new();
    let mut add = |setting: &str, value: String, reason: &str, confidence: &str| {
        settings.push(SettingRecommendation {
            setting: setting.into(),
            value,
            reason: reason.into(),
            confidence: confidence.into(),
        })
    };
    add("Preset",selected.into(),"Conservative starting point based on the weakest evidence. Ultra is an optional experiment, never inferred from requirements.","Low: heuristic");
    add("Resolution",format!("{width} × {height}"),"Use a selected display mode. 1440p renders more pixels than 1080p; compare measured results.","High for selected dimensions; low for performance");
    add("Render scale",if struggling || height>=1440 || report.verdict!=Verdict::MeetsRecommended {"75%"}else{"100%"}.into(),"Lower scaling reduces GPU pixel workload at the cost of clarity. Apply only if the game exposes this control.","Low: heuristic");
    add("Textures",textures.into(),"Based on dedicated VRAM only. Texture budgets vary by game; lower this if stuttering persists.","Medium for capacity; low for game budget");
    for (setting, reason) in [
        ("Shadows", "Reducing shadow quality often lowers GPU cost."),
        (
            "View distance",
            "Reduce this when CPU limits or dense scenes cause slowdowns.",
        ),
        (
            "Effects",
            "Lower effects to reduce GPU load in busy scenes.",
        ),
    ] {
        add(
            setting,
            if struggling { "Low" } else { selected }.into(),
            reason,
            "Low: heuristic",
        );
    }
    add(
        "Anti-aliasing",
        "TAA or low-cost option if available".into(),
        "Avoid expensive supersampling until you have measured performance.",
        "Low: game-dependent",
    );
    add("Upscaling","Check the game's supported options; try Quality mode if available".into(),"FSR availability depends on game integration. DLSS and XeSS variants require separate game and hardware verification. No supported technology is inferred from a GPU name.","Unknown until game support is confirmed");
    if struggling {
        add("Measured tuning","Lower shadows/effects first, then render scale; measure again".into(),"Your observed FPS is below your target. This is a tuning loop, not a predicted FPS calculation.","Medium: based on user measurement");
    }
    settings
}
