use gamespec_core::{catalog, engine, profiles, *};

fn fixture(name: &str) -> SystemSnapshot {
    let json = match name {
        "amd" => include_str!("fixtures/amd.json"),
        "nvidia" => include_str!("fixtures/nvidia.json"),
        "intel" => include_str!("fixtures/intel.json"),
        "multi" => include_str!("fixtures/multi_gpu.json"),
        "storage" => include_str!("fixtures/inadequate_storage.json"),
        "missing" => include_str!("fixtures/missing.json"),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_str(json).unwrap()
}
fn report(name: &str, gpu: Option<&str>) -> Assessment {
    let system = fixture(name);
    assess(
        &profiles::sample(),
        &system,
        gpu,
        "C:\\Games\\Icarus",
        system.volumes.first().cloned(),
    )
}
fn component<'a>(report: &'a Assessment, name: &str) -> &'a ComponentResult {
    report
        .components
        .iter()
        .find(|c| c.component == name)
        .unwrap()
}
fn exact_profile(system: &SystemSnapshot) -> GameProfile {
    let mut profile = profiles::sample();
    for req in [&mut profile.minimum, &mut profile.recommended] {
        req.cpu = vec![system.cpu.model.clone().unwrap()];
        req.gpu = vec![system.gpus[0].model.clone()];
    }
    profile
}
#[test]
fn cross_vendor_near_equivalent_is_unknown_not_name_failure() {
    let report = report("amd", Some("amd"));
    let gpu = component(&report, "GPU");
    assert_eq!(gpu.recommended.result, EvidenceResult::Unknown);
    assert!(gpu.recommended.explanation.contains("1.19"));
    assert!(gpu.recommended.source.starts_with("https://tweakers.net"));
}
#[test]
fn attributed_cross_vendor_comparison_can_pass_or_fail() {
    let required = vec!["NVIDIA RTX 2070".into()];
    assert_eq!(
        catalog::compare("gpu", Some("Intel Arc A770"), &required).result,
        EvidenceResult::Pass
    );
    assert_eq!(
        catalog::compare("gpu", Some("NVIDIA RTX 2070"), &["Intel Arc A770".into()]).result,
        EvidenceResult::Fail
    );
}
#[test]
fn unknown_alternative_prevents_false_failure() {
    assert_eq!(
        catalog::compare(
            "gpu",
            Some("RTX 2070"),
            &["Intel Arc A770".into(), "Future GPU".into()]
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn exact_alternative_passes_despite_unknown_alternative() {
    assert_eq!(
        catalog::compare(
            "cpu",
            Some("AMD Ryzen 5 2600 Six-Core Processor"),
            &["Unknown CPU".into(), "AMD Ryzen 5 2600".into()]
        )
        .result,
        EvidenceResult::Pass
    );
}
#[test]
fn similar_names_and_laptop_variants_never_alias() {
    for actual in [
        "NVIDIA GeForce RTX 2070 SUPER",
        "NVIDIA GeForce RTX 2070 Laptop GPU",
        "NVIDIA GeForce RTX 20700",
    ] {
        assert_eq!(
            catalog::compare("gpu", Some(actual), &["RTX 2070".into()]).result,
            EvidenceResult::Unknown
        );
    }
}
#[test]
fn higher_cpu_model_number_is_not_evidence() {
    assert_eq!(
        catalog::compare(
            "cpu",
            Some("Intel Core i9-99999"),
            &["AMD Ryzen 7 3700X".into()]
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn empty_missing_models_are_unknown() {
    assert_eq!(
        catalog::compare("cpu", None, &["CPU".into()]).result,
        EvidenceResult::Unknown
    );
    assert_eq!(
        catalog::compare("gpu", Some(""), &["GPU".into()]).result,
        EvidenceResult::Unknown
    );
    assert_eq!(
        catalog::compare("gpu", Some("GPU"), &[]).result,
        EvidenceResult::Unknown
    );
}
#[test]
fn exact_nvidia_and_intel_aliases_work() {
    assert_eq!(
        component(&report("nvidia", Some("nvidia")), "GPU")
            .recommended
            .result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&report("intel", Some("intel")), "CPU")
            .recommended
            .result,
        EvidenceResult::Pass
    );
}
#[test]
fn missing_data_is_unknown() {
    let report = report("missing", None);
    assert_eq!(report.verdict, Verdict::Unknown);
    for name in [
        "CPU",
        "GPU",
        "RAM",
        "VRAM",
        "Windows",
        "Storage",
        "Graphics API",
    ] {
        assert_eq!(
            component(&report, name).minimum.result,
            EvidenceResult::Unknown
        );
    }
}
#[test]
fn multi_gpu_uses_explicit_selection_and_dedicated_vram_only() {
    let integrated = report("multi", Some("integrated"));
    let discrete = report("multi", Some("discrete"));
    assert_eq!(
        component(&integrated, "VRAM").minimum.result,
        EvidenceResult::Fail
    );
    assert_eq!(
        component(&discrete, "VRAM").recommended.result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&report("multi", None), "VRAM").minimum.result,
        EvidenceResult::Unknown
    );
}
#[test]
fn inadequate_storage_fails_even_when_other_evidence_unknown() {
    let report = report("storage", Some("amd"));
    assert_eq!(report.verdict, Verdict::BelowMinimum);
    assert_eq!(report.headroom_bytes, Some(-50_000_000_000));
    assert_eq!(
        component(&report, "SSD").recommended.result,
        EvidenceResult::Fail
    );
}
#[test]
fn all_numeric_boundaries_are_inclusive() {
    let system = fixture("nvidia");
    let mut profile = exact_profile(&system);
    profile.minimum = profile.recommended.clone();
    profile.minimum.storage_gb = 500.0;
    profile.recommended.storage_gb = 500.0;
    let report = assess(
        &profile,
        &system,
        Some("nvidia"),
        "C:\\",
        system.volumes.first().cloned(),
    );
    assert_eq!(report.verdict, Verdict::MeetsRecommended);
    assert_eq!(report.headroom_bytes, Some(0));
}
#[test]
fn memory_and_vram_shortfalls_are_independent() {
    let mut system = fixture("nvidia");
    system.ram_total_bytes = Some(4 * 1073741824);
    system.gpus[0].dedicated_bytes = None;
    let report = assess(&profiles::sample(), &system, Some("nvidia"), "C:\\", None);
    assert_eq!(
        component(&report, "RAM").minimum.result,
        EvidenceResult::Fail
    );
    assert_eq!(
        component(&report, "VRAM").minimum.result,
        EvidenceResult::Unknown
    );
    assert_eq!(report.verdict, Verdict::BelowMinimum);
}
#[test]
fn available_ram_warns_but_does_not_replace_installed_capacity() {
    let mut system = fixture("amd");
    system.ram_available_bytes = Some(1);
    let report = assess(&profiles::sample(), &system, Some("amd"), "C:\\", None);
    assert_eq!(
        component(&report, "RAM").recommended.result,
        EvidenceResult::Pass
    );
    assert!(report
        .warnings
        .iter()
        .any(|s| s.starts_with("Available RAM")));
}
#[test]
fn api_probe_failures_and_feature_levels_are_independent() {
    let mut system = fixture("amd");
    system.gpus[0].directx12 = None;
    let mut profile = profiles::sample();
    let a = assess(&profile, &system, Some("amd"), "C:\\", None);
    assert_eq!(
        component(&a, "Graphics API").minimum.result,
        EvidenceResult::Unknown
    );
    system.gpus[0].directx12 = Some(false);
    let b = assess(&profile, &system, Some("amd"), "C:\\", None);
    assert_eq!(
        component(&b, "Graphics API").minimum.result,
        EvidenceResult::Fail
    );
    system.gpus[0].directx12 = Some(true);
    profile.minimum.feature_level = Some("12_2".into());
    let c = assess(&profile, &system, Some("amd"), "C:\\", None);
    assert_eq!(
        component(&c, "Graphics API").minimum.result,
        EvidenceResult::Fail
    );
    profile.minimum.graphics_api = "Vulkan".into();
    assert_eq!(
        component(
            &assess(&profile, &system, Some("amd"), "C:\\", None),
            "Graphics API"
        )
        .minimum
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn windows_edition_architecture_and_build_are_checked() {
    let mut system = fixture("amd");
    for (edition, build, arch, result) in [
        (
            "Microsoft Windows 10 Home",
            19045,
            "x86_64",
            EvidenceResult::Pass,
        ),
        ("Microsoft Windows 8", 9600, "x86_64", EvidenceResult::Fail),
        ("Windows Server 2022", 20348, "x86_64", EvidenceResult::Fail),
        ("Windows 11", 26100, "x86", EvidenceResult::Fail),
    ] {
        system.os.edition = Some(edition.into());
        system.os.build = Some(build);
        system.os.architecture = Some(arch.into());
        assert_eq!(
            component(
                &assess(&profiles::sample(), &system, Some("amd"), "C:\\", None),
                "Windows"
            )
            .minimum
            .result,
            result
        );
    }
}
#[test]
fn ssd_requirement_fails_hdd_and_unknown_stays_unknown() {
    let mut system = fixture("storage");
    let mut profile = profiles::sample();
    profile.minimum.ssd = true;
    let a = assess(
        &profile,
        &system,
        Some("amd"),
        "C:\\",
        system.volumes.first().cloned(),
    );
    assert_eq!(component(&a, "SSD").minimum.result, EvidenceResult::Fail);
    system.volumes[0].media = MediaKind::Unknown;
    let b = assess(
        &profile,
        &system,
        Some("amd"),
        "C:\\",
        system.volumes.first().cloned(),
    );
    assert_eq!(component(&b, "SSD").minimum.result, EvidenceResult::Unknown);
}
#[test]
fn verdict_table_covers_all_nine_combinations() {
    use EvidenceResult::*;
    for (m, r, expected) in [
        (Pass, Pass, Verdict::MeetsRecommended),
        (Pass, Fail, Verdict::MeetsMinimum),
        (Pass, Unknown, Verdict::Unknown),
        (Fail, Pass, Verdict::BelowMinimum),
        (Fail, Fail, Verdict::BelowMinimum),
        (Fail, Unknown, Verdict::BelowMinimum),
        (Unknown, Pass, Verdict::Unknown),
        (Unknown, Fail, Verdict::Unknown),
        (Unknown, Unknown, Verdict::Unknown),
    ] {
        assert_eq!(engine::verdict(m, r), expected);
    }
}
#[test]
fn recommendations_use_vram_and_user_measurements_without_fps_prediction() {
    let report = report("amd", Some("amd"));
    let settings = recommend(&report, 2560, 1440, None, Some(30.0), Some(60.0));
    assert_eq!(
        settings
            .iter()
            .find(|s| s.setting == "Textures")
            .unwrap()
            .value,
        "Medium"
    );
    assert!(settings.iter().any(|s| s.setting == "Measured tuning"));
    assert_eq!(
        recommend(&report, 1920, 1080, Some("Ultra"), None, None)[0].value,
        "Ultra"
    );
}
#[test]
fn profiles_validate_import_and_reject_path_traversal_and_bad_values() {
    let sample = profiles::sample();
    profiles::validate(&sample).unwrap();
    assert_eq!(
        profiles::parse(&serde_json::to_string(&sample).unwrap())
            .unwrap()
            .id,
        "icarus"
    );
    for id in ["../bad", "C:\\bad", ".", "", "CON", "com1", "LPT9"] {
        let mut p = sample.clone();
        p.id = id.into();
        assert!(profiles::validate(&p).is_err());
    }
    for n in [f64::NAN, f64::INFINITY, -1.0] {
        let mut p = sample.clone();
        p.minimum.ram_gib = n;
        assert!(profiles::validate(&p).is_err());
    }
    let mut p = sample.clone();
    p.schema_version = 2;
    assert!(profiles::validate(&p).is_err());
    p = sample.clone();
    p.recommended.ram_gib = 1.0;
    assert!(profiles::validate(&p).is_err());
}
#[test]
fn local_persistence_roundtrips_and_overwrites_without_losing_sample() {
    let dir = std::env::temp_dir().join(format!("gamespec-test-{}", std::process::id()));
    let mut p = profiles::sample();
    p.id = "test-game".into();
    profiles::save(&dir, &p).unwrap();
    p.name = "Changed".into();
    profiles::save(&dir, &p).unwrap();
    let loaded = profiles::load(&dir).unwrap();
    assert!(loaded.iter().any(|p| p.id == "icarus"));
    assert_eq!(
        loaded.iter().find(|p| p.id == "test-game").unwrap().name,
        "Changed"
    );
    std::fs::remove_file(dir.join("test-game.json")).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn location_rejects_relative_or_unavailable_paths() {
    assert!(resolve_location("Games\\Icarus", &[]).is_err());
    assert!(resolve_location("", &[]).is_err());
}
