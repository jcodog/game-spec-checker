use crate::{
    catalog::{evidence, Catalog},
    *,
};

pub fn aggregate(values: impl Iterator<Item = EvidenceResult>) -> EvidenceResult {
    let values: Vec<_> = values.collect();
    if values.contains(&EvidenceResult::Fail) {
        EvidenceResult::Fail
    } else if values.is_empty() || values.contains(&EvidenceResult::Unknown) {
        EvidenceResult::Unknown
    } else {
        EvidenceResult::Pass
    }
}
pub fn verdict(min: EvidenceResult, rec: EvidenceResult) -> Verdict {
    match (min, rec) {
        (EvidenceResult::Fail, _) => Verdict::BelowMinimum,
        (EvidenceResult::Pass, EvidenceResult::Pass) => Verdict::MeetsRecommended,
        (EvidenceResult::Pass, _) => Verdict::MeetsMinimum,
        _ => Verdict::Unknown,
    }
}
pub fn capacity(actual: Option<u64>, required: Option<Capacity>, label: &str) -> Evidence {
    let Some(required) = required else {
        return evidence(
            EvidenceResult::Unknown,
            &format!("No {label} requirement entered. Check the publisher's specification."),
            "Game profile",
        );
    };
    if required.value == 0.0 {
        return evidence(
            EvidenceResult::Pass,
            "Profile explicitly specifies no capacity requirement.",
            "Game profile",
        );
    }
    let Some(actual) = actual else {
        return evidence(
            EvidenceResult::Unknown,
            &format!("Native {label} measurement unavailable."),
            "Native scanner",
        );
    };
    let (low, high) = required.bounds();
    let result = if actual as f64 >= high {
        EvidenceResult::Pass
    } else if (actual as f64) < low {
        EvidenceResult::Fail
    } else {
        EvidenceResult::Unknown
    };
    let mut e=evidence(result,&format!("{:.2} GB / {:.2} GiB reported by the API; {} required. GB = 1,000,000,000 bytes; GiB = 1,073,741,824 bytes. Shared graphics/system memory is not dedicated VRAM.",actual as f64/1e9,actual as f64/1073741824.0,required.label()),"Native API bytes and explicitly selected profile units");
    if required.unit == CapacityUnit::AmbiguousGb {
        e.explanation.push_str(" The published GB unit is ambiguous; confirm its meaning. Results between the decimal and binary thresholds stay uncertain.");
        if result == EvidenceResult::Unknown {
            e.confidence = Confidence::Unknown;
        }
    }
    e
}
fn windows(os: &OperatingSystem, req: &Requirements) -> Evidence {
    if req.windows_versions.is_empty() || req.architecture.is_none() {
        return evidence(
            EvidenceResult::Unknown,
            "Windows versions or architecture are missing from this profile.",
            "Game profile",
        );
    }
    let (Some(edition), Some(build), Some(arch)) = (&os.edition, os.build, &os.architecture) else {
        return evidence(
            EvidenceResult::Unknown,
            "Windows edition, build or architecture was not detected.",
            "Native scanner",
        );
    };
    let version = if build >= 22000 {
        11
    } else if build >= 10240 {
        10
    } else {
        0
    };
    let compatible = edition.contains("Windows")
        && !edition.contains("Server")
        && req.windows_versions.contains(&version)
        && req.architecture.as_ref() == Some(arch);
    evidence(
        if compatible {
            EvidenceResult::Pass
        } else {
            EvidenceResult::Fail
        },
        &format!(
            "{edition}, build {build}, {arch}; accepted Windows {:?}, {:?}.",
            req.windows_versions, req.architecture
        ),
        "Native architecture and WMI Windows edition/build",
    )
}
fn graphics(gpu: Option<&Gpu>, req: &Requirements) -> Evidence {
    let Some(api) = req.graphics_api.as_deref() else {
        return evidence(
            EvidenceResult::Unknown,
            "No graphics API requirement entered.",
            "Game profile",
        );
    };
    let Some(gpu) = gpu else {
        return evidence(
            EvidenceResult::Unknown,
            "Choose a detected gaming GPU.",
            "Native scanner",
        );
    };
    let api_result = match api {
        "DirectX 12" => match gpu.directx12 {
            Some(true) => EvidenceResult::Pass,
            Some(false) => EvidenceResult::Fail,
            None => EvidenceResult::Unknown,
        },
        "DirectX 11" if gpu.directx12 == Some(true) => EvidenceResult::Pass,
        _ => EvidenceResult::Unknown,
    };
    if !["DirectX 11", "DirectX 12"].contains(&api) {
        return evidence(EvidenceResult::Unknown,&format!("{api} is not probed. A Direct3D feature level cannot establish another API's support."),"Native API coverage");
    }
    let result = if let Some(required) = req.feature_level.as_deref() {
        let levels = ["11_0", "11_1", "12_0", "12_1", "12_2"];
        let feature = match (
            gpu.feature_level
                .as_deref()
                .and_then(|v| levels.iter().position(|l| *l == v)),
            levels.iter().position(|l| *l == required),
        ) {
            (Some(a), Some(b)) => {
                if a >= b {
                    EvidenceResult::Pass
                } else {
                    EvidenceResult::Fail
                }
            }
            _ => EvidenceResult::Unknown,
        };
        aggregate([api_result, feature].into_iter())
    } else {
        api_result
    };
    evidence(result,&format!("{api} required. D3D12 device support: {:?}; detected feature level {:?}; required feature level {:?}. Optional ray tracing/upscaling features require separate verification.",gpu.directx12,gpu.feature_level,req.feature_level),"D3D12CreateDevice probe on selected DXGI adapter")
}
fn ssd(volume: Option<&Volume>, policy: SsdPolicy) -> Evidence {
    if policy == SsdPolicy::Unspecified {
        return evidence(
            EvidenceResult::Pass,
            "No SSD condition declared at this level.",
            "Game profile",
        );
    }
    let result = match volume.map(|v| v.media) {
        Some(MediaKind::Ssd) => EvidenceResult::Pass,
        Some(MediaKind::Hdd) => EvidenceResult::Fail,
        _ => EvidenceResult::Unknown,
    };
    evidence(result,&format!("SSD is {:?}; selected media is {:?}. A recommendation is advisory and does not fail hard compatibility.",policy,volume.map(|v|v.media)),"Native storage classification; unknown for some RAID/USB/virtual devices")
}
fn summarize(components: &[ComponentResult], minimum: bool) -> LevelSummary {
    let values: Vec<_> = components
        .iter()
        .filter(|c| {
            if minimum {
                c.minimum_required
            } else {
                c.recommended_required
            }
        })
        .map(|c| {
            if minimum {
                c.minimum.result
            } else {
                c.recommended.result
            }
        })
        .collect();
    LevelSummary {
        result: aggregate(values.iter().copied()),
        passed: values
            .iter()
            .filter(|v| **v == EvidenceResult::Pass)
            .count(),
        failed: values
            .iter()
            .filter(|v| **v == EvidenceResult::Fail)
            .count(),
        unknown: values
            .iter()
            .filter(|v| **v == EvidenceResult::Unknown)
            .count(),
    }
}
pub fn assess_with_catalog(
    profile: &GameProfile,
    system: &SystemSnapshot,
    gpu_id: Option<&str>,
    destination: &str,
    volume: Option<Volume>,
    catalog: &Catalog,
) -> Assessment {
    let gpu = gpu_id.and_then(|id| system.gpus.iter().find(|g| g.id == id));
    let mut components = Vec::new();
    for (component, kind) in [("CPU", "cpu"), ("GPU", "gpu")] {
        let actual = if kind == "cpu" {
            system.cpu.model.as_deref()
        } else {
            gpu.map(|g| g.model.as_str())
        };
        let workload = if kind == "cpu" {
            profile.cpu_workload
        } else {
            profile.gpu_workload
        };
        let min = if kind == "cpu" {
            &profile.minimum.cpu
        } else {
            &profile.minimum.gpu
        };
        let rec = if kind == "cpu" {
            &profile.recommended.cpu
        } else {
            &profile.recommended.gpu
        };
        let minimum = catalog.compare(kind, actual, min, workload);
        let recommended = catalog.compare(kind, actual, rec, workload);
        components.push(ComponentResult {
            component: component.into(),
            verdict: verdict(minimum.result, recommended.result),
            minimum,
            recommended,
            minimum_required: true,
            recommended_required: true,
        });
    }
    for (component, minimum, recommended) in [
        (
            "RAM",
            capacity(system.ram_total_bytes, profile.minimum.ram, "RAM"),
            capacity(system.ram_total_bytes, profile.recommended.ram, "RAM"),
        ),
        (
            "VRAM",
            capacity(
                gpu.and_then(|g| g.dedicated_bytes),
                profile.minimum.vram,
                "dedicated VRAM",
            ),
            capacity(
                gpu.and_then(|g| g.dedicated_bytes),
                profile.recommended.vram,
                "dedicated VRAM",
            ),
        ),
        (
            "Graphics API",
            graphics(gpu, &profile.minimum),
            graphics(gpu, &profile.recommended),
        ),
        (
            "Windows",
            windows(&system.os, &profile.minimum),
            windows(&system.os, &profile.recommended),
        ),
        (
            "Storage",
            capacity(
                volume.as_ref().map(|v| v.free_bytes),
                profile.minimum.storage,
                "installation space",
            ),
            capacity(
                volume.as_ref().map(|v| v.free_bytes),
                profile.recommended.storage,
                "installation space",
            ),
        ),
        (
            "SSD",
            ssd(volume.as_ref(), profile.minimum.ssd),
            ssd(volume.as_ref(), profile.recommended.ssd),
        ),
    ] {
        components.push(ComponentResult {
            component: component.into(),
            verdict: verdict(minimum.result, recommended.result),
            minimum,
            recommended,
            minimum_required: component != "SSD" || profile.minimum.ssd == SsdPolicy::Required,
            recommended_required: component != "SSD"
                || profile.recommended.ssd == SsdPolicy::Required,
        });
    }
    let minimum = summarize(&components, true);
    let recommended = summarize(&components, false);
    let mut warnings=vec!["Compatibility evidence is separate from graphics heuristics. Neither promises FPS or stability.".into()];
    warnings.extend(system.diagnostics.clone());
    warnings.extend(profile.migration_notes.clone());
    if profile.provenance.kind == ProvenanceKind::Unverified
        || profile.provenance.source_url.is_none()
    {
        warnings.push("Requirements have no verified source. Confirm this game's platform, version and published specifications.".into());
    }
    if gpu.is_none() {
        warnings.push("Select the intended gaming GPU; display attachment does not prove which adapter a game uses.".into());
    }
    if let (Some(available), Some(required)) = (system.ram_available_bytes, profile.minimum.ram) {
        if (available as f64) < required.bounds().1 {
            warnings.push("Available RAM is below the entered minimum. Close background applications; installed RAM is assessed separately.".into());
        }
    }
    if let Some(volume) = &volume {
        if volume.removable == Some(true) {
            warnings.push("The selected volume is removable. Check that the game supports this destination and keep it connected.".into());
        }
    }
    if components.iter().any(|c| {
        c.component == "SSD"
            && ((profile.minimum.ssd != SsdPolicy::Unspecified
                && c.minimum.result != EvidenceResult::Pass)
                || (profile.recommended.ssd != SsdPolicy::Unspecified
                    && c.recommended.result != EvidenceResult::Pass))
    }) {
        warnings.push("SSD advice: suitable solid-state storage was not confirmed. Recommended is a warning; required is a hard constraint.".into());
    }
    let required = [profile.minimum.storage, profile.recommended.storage]
        .into_iter()
        .flatten()
        .map(|c| c.bounds().1)
        .reduce(f64::max);
    let headroom_bytes = volume
        .as_ref()
        .zip(required)
        .map(|(v, r)| v.free_bytes as i128 - r.ceil() as i128);
    if components
        .iter()
        .any(|c| c.component == "Storage" && c.minimum.result == EvidenceResult::Fail)
    {
        warnings.push("Do not install on this volume: it lacks the entered required installation space. Choose another location or free space yourself.".into());
    } else if headroom_bytes.is_some_and(|h| h < 0) {
        warnings.push("The recommended installation size exceeds free space. Minimum space may fit; review the publisher's installation options before proceeding.".into());
    }
    if let (Some(headroom), Some(buffer)) = (headroom_bytes, profile.installation_headroom) {
        if headroom < (buffer.bounds().1.ceil() as i128) {
            warnings.push(format!("The optional {} installation buffer is not available. This buffer is advisory and does not change compatibility.",buffer.label()));
        }
    }
    Assessment {
        schema_version: 2,
        profile: profile.clone(),
        system: system.clone(),
        gpu_id: gpu_id.map(String::from),
        destination: destination.into(),
        volume,
        headroom_bytes,
        verdict: verdict(minimum.result, recommended.result),
        minimum,
        recommended,
        components,
        warnings,
        catalog_version: catalog.version.clone(),
    }
}
pub fn assess(
    profile: &GameProfile,
    system: &SystemSnapshot,
    gpu_id: Option<&str>,
    destination: &str,
    volume: Option<Volume>,
) -> Assessment {
    assess_with_catalog(
        profile,
        system,
        gpu_id,
        destination,
        volume,
        &crate::catalog::catalog(),
    )
}
pub fn recommend(
    report: &Assessment,
    width: u32,
    height: u32,
    preset: Option<&str>,
    observed: Option<f64>,
    target: Option<f64>,
) -> Vec<SettingRecommendation> {
    crate::recommendations::recommend(report, width, height, preset, observed, target)
}
