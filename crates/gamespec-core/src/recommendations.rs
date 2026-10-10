use crate::*;

pub fn recommend(
    report: &Assessment,
    width: u32,
    height: u32,
    preset: Option<&str>,
    observed: Option<f64>,
    target: Option<f64>,
) -> Vec<SettingRecommendation> {
    let result = |component: &str, rec: bool| {
        report
            .components
            .iter()
            .find(|c| c.component == component)
            .map(|c| {
                if rec {
                    c.recommended.result
                } else {
                    c.minimum.result
                }
            })
            .unwrap_or(EvidenceResult::Unknown)
    };
    let gpu_min = result("GPU", false);
    let gpu_rec = result("GPU", true);
    let cpu_min = result("CPU", false);
    let pixels = u64::from(width) * u64::from(height);
    let struggling = matches!((observed,target),(Some(a),Some(b)) if a.is_finite() && b.is_finite() && a>0.0 && b>0.0 && a<b);
    let initial = if gpu_rec == EvidenceResult::Pass && pixels <= 1920 * 1080 {
        "High"
    } else if gpu_min == EvidenceResult::Pass {
        "Medium"
    } else {
        "Low"
    };
    let manual = preset.filter(|p| ["Very Low", "Low", "Medium", "High", "Ultra"].contains(p));
    let selected = manual.unwrap_or(if struggling { "Low" } else { initial });
    let uncertain = gpu_min == EvidenceResult::Unknown || cpu_min == EvidenceResult::Unknown;
    let confidence = if uncertain {
        "Low: missing performance evidence; measure in the game"
    } else {
        "Medium: requirement-relative heuristic, not a game benchmark"
    };
    let mut settings = Vec::new();
    let mut add = |setting: &str, value: String, reason: String, certainty: &str| {
        settings.push(SettingRecommendation {
            setting: setting.into(),
            value,
            reason,
            confidence: certainty.into(),
        })
    };
    add(
        "Starting preset",
        selected.into(),
        if manual.is_some() {
            "Manual trial selected by you. It is not inferred as supported or optimal.".into()
        } else {
            format!("GPU minimum is {gpu_min:?}; recommended is {gpu_rec:?}. Resolution is {width} × {height}. Storage/OS failures do not set graphical quality. Generic starting point; no verified game-specific preset is available.")
        },
        confidence,
    );
    add("Display resolution",format!("{width} × {height}"),"Selected by you. Use a connected display mode; changing render resolution is different from changing output resolution.".into(),"High for selected dimensions; performance unmeasured");
    let scaling = if cpu_min == EvidenceResult::Fail {
        "100% initially"
    } else if (struggling || pixels > 1920 * 1080) && gpu_rec != EvidenceResult::Pass {
        "Try 75% after measuring"
    } else {
        "100% initially"
    };
    add(
        "Render scaling",
        scaling.into(),
        if cpu_min == EvidenceResult::Fail {
            "CPU evidence is below minimum. Reducing pixel count cannot repair a CPU requirement; test view distance and CPU-heavy settings first.".into()
        } else {
            "Lower scale reduces GPU pixel work and image clarity. Only change it if the game exposes this option and measurements suggest a GPU limit.".into()
        },
        confidence,
    );
    let textures = match (result("VRAM", false), result("VRAM", true)) {
        (_, EvidenceResult::Pass) => "Medium to High",
        (EvidenceResult::Pass, _) => "Low to Medium",
        (EvidenceResult::Fail, _) => "Very Low",
        _ => "Low until memory budget is measured",
    };
    add("Textures",textures.into(),"Compared with this game's entered dedicated VRAM requirements using their original units. Shared RAM is excluded. Watch VRAM usage/stuttering; the game-specific texture budget is not known.".into(),"Medium for capacity evidence; low for texture budget");
    add("View distance",if cpu_min==EvidenceResult::Fail || struggling {"Low"}else{"Medium"}.into(),format!("CPU minimum evidence is {cpu_min:?}. View distance, crowds and simulation can increase CPU work; observe CPU/GPU utilization before changing resolution."),confidence);
    for setting in ["Shadows", "Effects"] {
        add(setting,if struggling || gpu_min!=EvidenceResult::Pass {"Low"}else{"Medium"}.into(),"Generic GPU-cost controls to reduce first when measured performance is below target. Availability and cost vary by game.".into(),confidence);
    }
    add("Anti-aliasing","Low-cost native option initially".into(),"Use the game's documented options. Avoid supersampling before measuring. No particular AA implementation is assumed to exist.".into(),"Low: game integration unverified");
    add("Ray tracing / upscaling / frame generation","Leave disabled until verified".into(),"Raster, ray-tracing and compute workloads are distinct. GPU feature level alone does not verify DLSS, FSR, XeSS or game integration. No enabled technology is recommended without both checks.".into(),"Unknown: game and hardware feature verification required");
    if report
        .components
        .iter()
        .any(|c| c.component == "Storage" && c.minimum.result == EvidenceResult::Fail)
    {
        add("Installation","Choose a volume with enough space before playing".into(),"The chosen volume fails the entered minimum storage requirement. Graphics settings cannot solve insufficient disk space.".into(),"High: measured space versus entered requirement");
    }
    if report
        .volume
        .as_ref()
        .is_some_and(|v| v.media == MediaKind::Hdd)
    {
        add("Asset streaming","Monitor loading and streaming stutter".into(),"This installation uses HDD storage. Follow any verified SSD requirement/advice; a faster disk is not a guaranteed average-FPS increase.".into(),"Medium for media; low for game behavior");
    }
    if let (Some(available), Some(required)) = (
        report.system.ram_available_bytes,
        report.profile.minimum.ram,
    ) {
        if (available as f64) < required.bounds().1 {
            add("Background applications","Close memory-heavy applications and retest".into(),"Available RAM is below the entered minimum. Installed capacity alone does not describe current memory pressure.".into(),"High for current measurement");
        }
    }
    if let Some(goal) = target.filter(|v| v.is_finite() && *v > 0.0) {
        add("Measured tuning",if struggling {"Change one setting, then repeat the same scene"}else{"Record FPS and utilization in a repeatable scene"}.into(),format!("Your target is {goal:.0} FPS; observed result is {:?}. A target does not create a frame-rate prediction. Capture averages, lows and utilization; reduce CPU-heavy settings for CPU limits, pixel/GPU settings for GPU limits.",observed),if observed.is_some(){"Medium: based on your measurement"}else{"Low: measured results needed"});
    }
    settings
}
