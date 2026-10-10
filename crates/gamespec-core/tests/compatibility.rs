use gamespec_core::{
    catalog::{self, Catalog, Dataset, Measurement},
    engine, profiles, *,
};
fn fixture(name: &str) -> SystemSnapshot {
    let json = match name {
        "amd" => include_str!("fixtures/amd.json"),
        "nvidia" => include_str!("fixtures/nvidia.json"),
        "intel" => include_str!("fixtures/intel.json"),
        "multi" => include_str!("fixtures/multi_gpu.json"),
        "storage" => include_str!("fixtures/inadequate_storage.json"),
        "missing" => include_str!("fixtures/missing.json"),
        "reported" => include_str!("fixtures/reported-system.json"),
        _ => panic!("unknown fixture"),
    };
    serde_json::from_str(json).unwrap()
}
fn report(name: &str, gpu: Option<&str>) -> Assessment {
    let s = fixture(name);
    assess(
        &profiles::sample(),
        &s,
        gpu,
        "C:\\Games",
        s.volumes.first().cloned(),
    )
}
fn component<'a>(r: &'a Assessment, name: &str) -> &'a ComponentResult {
    r.components.iter().find(|c| c.component == name).unwrap()
}
fn exact(s: &SystemSnapshot) -> GameProfile {
    let mut p = profiles::sample();
    for r in [&mut p.minimum, &mut p.recommended] {
        r.cpu = vec![s.cpu.model.clone().unwrap()];
        r.gpu = vec![s.gpus[0].model.clone()];
    }
    p
}
// Deliberately synthetic intervals exercise the algorithm. These are not hardware benchmarks.
fn measured(
    kind: &str,
    actual: &str,
    required: &str,
    workload: Workload,
    a: (f64, f64),
    b: (f64, f64),
) -> Catalog {
    let mut c = catalog::catalog();
    let aid = c.lookup(kind, actual).unwrap().id.clone();
    let bid = c.lookup(kind, required).unwrap().id.clone();
    c.datasets.push(Dataset {
        id: "test-only".into(),
        title: "Synthetic test intervals, not measured hardware".into(),
        workload,
        metric: "Test-only score".into(),
        test_context: "Identical synthetic unit-test context".into(),
        higher_is_better: true,
        methodology: "Deterministic test-only workload and context".into(),
        source_url: "https://example.org/test-fixture".into(),
        published_date: "2026-10-10".into(),
        license: "CC0-1.0".into(),
        redistribution_cleared: true,
        decision_eligible: true,
        uncertainty_method: "Test bounds supplied explicitly".into(),
    });
    for (id, bounds) in [(aid, a), (bid, b)] {
        c.measurements.push(Measurement {
            device_id: id,
            dataset_id: "test-only".into(),
            low: bounds.0,
            high: bounds.1,
            samples: 3,
        });
    }
    c.validate().unwrap();
    c
}
#[test]
fn bundled_catalog_is_valid_and_has_no_uncleared_decision_scores() {
    let c = catalog::catalog();
    c.validate().unwrap();
    assert!(c.devices.len() >= 50);
    assert_eq!(c.summary().decision_measurements, 0);
    assert!(c.measurements.is_empty());
}
#[test]
fn normalization_handles_real_windows_cpu_names() {
    for name in [
        "AMD Ryzen 5 3600 6-Core Processor",
        "AMD Ryzen 5 3600 Six-Core Processor",
    ] {
        assert_eq!(catalog::normalize(name), "amd ryzen 5 3600");
    }
    assert_eq!(
        catalog::normalize("11th Gen Intel(R) Core(TM) i7-11700 @ 2.50GHz"),
        "intel core i7-11700"
    );
}
#[test]
fn variant_names_are_not_erased() {
    let c = catalog::catalog();
    for name in ["RTX 2070 SUPER", "RTX 2070 Laptop GPU", "RTX 20700"] {
        assert_eq!(
            c.compare("gpu", Some(name), &["RTX 2070".into()], Workload::GpuRaster)
                .result,
            EvidenceResult::Unknown
        );
    }
}
#[test]
fn unknown_model_numbers_never_rank() {
    assert_eq!(
        catalog::catalog()
            .compare(
                "cpu",
                Some("Intel Core i9-99999"),
                &["AMD Ryzen 5 2600".into()],
                Workload::CpuGaming
            )
            .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn exact_alternative_is_sufficient() {
    assert_eq!(
        catalog::catalog()
            .compare(
                "cpu",
                Some("AMD Ryzen 5 2600 Six-Core Processor"),
                &["Unknown CPU".into(), "AMD Ryzen 5 2600".into()],
                Workload::CpuGaming
            )
            .result,
        EvidenceResult::Pass
    );
}
#[test]
fn no_evidence_remains_unknown() {
    for kind in ["cpu", "gpu"] {
        assert_eq!(
            catalog::catalog()
                .compare(kind, None, &[], Workload::CpuGaming)
                .result,
            EvidenceResult::Unknown
        );
    }
}
#[test]
fn cross_vendor_cpu_intervals_work() {
    let c = measured(
        "cpu",
        "AMD Ryzen 5 3600",
        "Intel Core i5-10500",
        Workload::CpuGaming,
        (110., 120.),
        (90., 100.),
    );
    assert_eq!(
        c.compare(
            "cpu",
            Some("AMD Ryzen 5 3600"),
            &["Intel Core i5-10500".into()],
            Workload::CpuGaming
        )
        .result,
        EvidenceResult::Pass
    );
    assert_eq!(
        c.compare(
            "cpu",
            Some("Intel Core i5-10500"),
            &["AMD Ryzen 5 3600".into()],
            Workload::CpuGaming
        )
        .result,
        EvidenceResult::Fail
    );
}
#[test]
fn cross_vendor_gpu_intervals_work() {
    let c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (110., 120.),
        (90., 100.),
    );
    let e = c.compare(
        "gpu",
        Some("AMD Radeon RX 7600"),
        &["NVIDIA RTX 2070".into()],
        Workload::GpuRaster,
    );
    assert_eq!(e.result, EvidenceResult::Pass);
    assert_eq!(e.confidence, Confidence::Medium);
    assert!(e.source.contains("2026-10-10"));
}
#[test]
fn overlapping_intervals_remain_unknown() {
    let c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (90., 110.),
        (95., 105.),
    );
    assert_eq!(
        c.compare(
            "gpu",
            Some("AMD Radeon RX 7600"),
            &["RTX 2070".into()],
            Workload::GpuRaster
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn workloads_cannot_substitute() {
    for workload in [
        Workload::GpuSynthetic,
        Workload::GpuRayTracing,
        Workload::GpuCompute,
    ] {
        let c = measured(
            "gpu",
            "AMD Radeon RX 7600",
            "NVIDIA GeForce RTX 2070",
            workload,
            (110., 120.),
            (90., 100.),
        );
        assert_eq!(
            c.compare(
                "gpu",
                Some("AMD Radeon RX 7600"),
                &["RTX 2070".into()],
                Workload::GpuRaster
            )
            .result,
            EvidenceResult::Unknown
        );
    }
    let c = measured(
        "cpu",
        "AMD Ryzen 5 3600",
        "Intel Core i5-10500",
        Workload::CpuMultiThread,
        (110., 120.),
        (90., 100.),
    );
    assert_eq!(
        c.compare(
            "cpu",
            Some("AMD Ryzen 5 3600"),
            &["Intel Core i5-10500".into()],
            Workload::CpuSingleThread
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn scales_cannot_mix() {
    let mut c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (110., 120.),
        (90., 100.),
    );
    let mut d = c.datasets[0].clone();
    d.id = "other-scale".into();
    c.datasets.push(d);
    c.measurements[1].dataset_id = "other-scale".into();
    c.validate().unwrap();
    assert_eq!(
        c.compare(
            "gpu",
            Some("AMD Radeon RX 7600"),
            &["RTX 2070".into()],
            Workload::GpuRaster
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn unknown_alternative_prevents_false_failure() {
    let c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (70., 80.),
        (90., 100.),
    );
    assert_eq!(
        c.compare(
            "gpu",
            Some("AMD Radeon RX 7600"),
            &["RTX 2070".into(), "Future GPU".into()],
            Workload::GpuRaster
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn cleared_license_is_required() {
    let mut c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (110., 120.),
        (90., 100.),
    );
    c.datasets[0].redistribution_cleared = false;
    assert!(c.validate().is_err());
    c.datasets[0].decision_eligible = false;
    c.validate().unwrap();
    assert_eq!(
        c.compare(
            "gpu",
            Some("AMD Radeon RX 7600"),
            &["RTX 2070".into()],
            Workload::GpuRaster
        )
        .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn catalog_import_rejects_ambiguity_and_invalid_intervals() {
    let mut c = catalog::catalog();
    let duplicate = c.devices[0].name.clone();
    c.devices[1].aliases.push(duplicate);
    assert!(c.validate().is_err());
    let mut c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (110., 120.),
        (90., 100.),
    );
    c.measurements[0].low = -1.;
    assert!(c.validate().is_err());
    assert!(catalog::parse("{}").is_err());
}
#[test]
fn gb_gib_and_ambiguous_units_are_preserved() {
    let bytes = 8_528_596_992;
    for (unit, result) in [
        (CapacityUnit::Gb, EvidenceResult::Pass),
        (CapacityUnit::Gib, EvidenceResult::Fail),
        (CapacityUnit::AmbiguousGb, EvidenceResult::Unknown),
    ] {
        assert_eq!(
            engine::capacity(Some(bytes), Some(Capacity { value: 8., unit }), "VRAM").result,
            result
        );
    }
    assert_eq!(
        engine::capacity(
            Some(8_000_000_000),
            Some(Capacity {
                value: 8.,
                unit: CapacityUnit::Gb
            }),
            "VRAM"
        )
        .result,
        EvidenceResult::Pass
    );
}
#[test]
fn borderline_bytes_do_not_round_up() {
    assert_eq!(
        engine::capacity(
            Some(7_999_999_999),
            Some(Capacity {
                value: 8.,
                unit: CapacityUnit::Gb
            }),
            "VRAM"
        )
        .result,
        EvidenceResult::Fail
    );
    assert_eq!(
        engine::capacity(None, None, "RAM").result,
        EvidenceResult::Unknown
    );
}
#[test]
fn reported_system_regression() {
    let r = report("reported", Some("amd"));
    assert_eq!(r.system.ram_total_bytes, Some(51539607552));
    assert_eq!(
        component(&r, "VRAM").recommended.result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&r, "RAM").recommended.result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&r, "Graphics API").recommended.result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&r, "Windows").recommended.result,
        EvidenceResult::Pass
    );
    assert_eq!(component(&r, "CPU").minimum.result, EvidenceResult::Unknown);
    assert_eq!(
        component(&r, "GPU").recommended.result,
        EvidenceResult::Unknown
    );
}
#[test]
fn missing_data_is_independently_unknown() {
    let r = report("missing", None);
    assert_eq!(r.verdict, Verdict::Unknown);
    for name in [
        "CPU",
        "GPU",
        "RAM",
        "VRAM",
        "Windows",
        "Storage",
        "Graphics API",
    ] {
        assert_eq!(component(&r, name).minimum.result, EvidenceResult::Unknown);
    }
}
#[test]
fn multi_gpu_never_uses_shared_ram_as_vram() {
    assert_eq!(
        component(&report("multi", Some("integrated")), "VRAM")
            .minimum
            .result,
        EvidenceResult::Fail
    );
    assert_eq!(
        component(&report("multi", Some("discrete")), "VRAM")
            .recommended
            .result,
        EvidenceResult::Pass
    );
    assert_eq!(
        component(&report("multi", None), "VRAM").minimum.result,
        EvidenceResult::Unknown
    );
}
#[test]
fn inadequate_storage_is_a_confirmed_failure() {
    let r = report("storage", Some("amd"));
    assert_eq!(r.verdict, Verdict::BelowMinimum);
    assert_eq!(r.headroom_bytes, Some(-50_000_000_000));
    assert!(r.warnings.iter().any(|s| s.starts_with("Do not install")));
}
#[test]
fn soft_ssd_warning_does_not_fail_recommended() {
    let mut s = fixture("nvidia");
    s.volumes[0].media = MediaKind::Hdd;
    let p = exact(&s);
    let r = assess(&p, &s, Some("nvidia"), "C:\\", s.volumes.first().cloned());
    assert_eq!(r.recommended.result, EvidenceResult::Pass);
    assert_eq!(
        component(&r, "SSD").recommended.result,
        EvidenceResult::Fail
    );
    assert!(!component(&r, "SSD").recommended_required);
}
#[test]
fn required_ssd_fails_hdd_and_unknown_is_unknown() {
    let mut s = fixture("amd");
    let mut p = profiles::sample();
    p.minimum.ssd = SsdPolicy::Required;
    p.recommended.ssd = SsdPolicy::Required;
    for (media, expected) in [
        (MediaKind::Hdd, EvidenceResult::Fail),
        (MediaKind::Unknown, EvidenceResult::Unknown),
        (MediaKind::Ssd, EvidenceResult::Pass),
    ] {
        s.volumes[0].media = media;
        let r = assess(&p, &s, Some("amd"), "C:\\", s.volumes.first().cloned());
        assert_eq!(component(&r, "SSD").minimum.result, expected);
    }
}
#[test]
fn headroom_is_optional_and_advisory() {
    let mut s = fixture("nvidia");
    s.volumes[0].free_bytes = 100_000_000_000;
    let mut p = exact(&s);
    p.installation_headroom = Some(Capacity {
        value: 20.,
        unit: CapacityUnit::Gb,
    });
    let r = assess(&p, &s, Some("nvidia"), "C:\\", s.volumes.first().cloned());
    assert_eq!(r.verdict, Verdict::MeetsRecommended);
    assert_eq!(r.headroom_bytes, Some(0));
    assert!(r.warnings.iter().any(|s| s.contains("optional 20 GB")));
}
#[test]
fn available_ram_warns_without_replacing_installed_ram() {
    let mut s = fixture("amd");
    s.ram_available_bytes = Some(1);
    let r = assess(&profiles::sample(), &s, Some("amd"), "C:\\", None);
    assert_eq!(
        component(&r, "RAM").recommended.result,
        EvidenceResult::Pass
    );
    assert!(r.warnings.iter().any(|s| s.starts_with("Available RAM")));
}
#[test]
fn feature_levels_api_failures_and_other_apis() {
    let mut s = fixture("amd");
    let mut p = profiles::sample();
    for (support, expected) in [
        (None, EvidenceResult::Unknown),
        (Some(false), EvidenceResult::Fail),
        (Some(true), EvidenceResult::Pass),
    ] {
        s.gpus[0].directx12 = support;
        assert_eq!(
            component(&assess(&p, &s, Some("amd"), "C:\\", None), "Graphics API")
                .minimum
                .result,
            expected
        );
    }
    p.minimum.feature_level = Some("12_2".into());
    assert_eq!(
        component(&assess(&p, &s, Some("amd"), "C:\\", None), "Graphics API")
            .minimum
            .result,
        EvidenceResult::Fail
    );
    p.minimum.graphics_api = Some("Vulkan".into());
    assert_eq!(
        component(&assess(&p, &s, Some("amd"), "C:\\", None), "Graphics API")
            .minimum
            .result,
        EvidenceResult::Unknown
    );
}
#[test]
fn windows_edition_architecture_and_build() {
    let mut s = fixture("amd");
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
        s.os.edition = Some(edition.into());
        s.os.build = Some(build);
        s.os.architecture = Some(arch.into());
        assert_eq!(
            component(
                &assess(&profiles::sample(), &s, Some("amd"), "C:\\", None),
                "Windows"
            )
            .minimum
            .result,
            result
        );
    }
}
#[test]
fn verdict_table_preserves_confirmed_minimum() {
    use EvidenceResult::*;
    for (m, r, v) in [
        (Pass, Pass, Verdict::MeetsRecommended),
        (Pass, Fail, Verdict::MeetsMinimum),
        (Pass, Unknown, Verdict::MeetsMinimum),
        (Fail, Pass, Verdict::BelowMinimum),
        (Fail, Fail, Verdict::BelowMinimum),
        (Fail, Unknown, Verdict::BelowMinimum),
        (Unknown, Pass, Verdict::Unknown),
        (Unknown, Fail, Verdict::Unknown),
        (Unknown, Unknown, Verdict::Unknown),
    ] {
        assert_eq!(engine::verdict(m, r), v);
    }
}
#[test]
fn missing_critical_requirement_is_not_a_pass() {
    let s = fixture("nvidia");
    let mut p = exact(&s);
    p.minimum.ram = None;
    let r = assess(&p, &s, Some("nvidia"), "C:\\", s.volumes.first().cloned());
    assert_eq!(r.minimum.result, EvidenceResult::Unknown);
    assert_eq!(component(&r, "VRAM").minimum.result, EvidenceResult::Pass);
}
#[test]
fn graphics_quality_is_independent_of_storage_failure() {
    let s = fixture("nvidia");
    let p = exact(&s);
    let a = assess(&p, &s, Some("nvidia"), "C:\\", s.volumes.first().cloned());
    let mut bad = s.volumes[0].clone();
    bad.free_bytes = 0;
    let b = assess(&p, &s, Some("nvidia"), "C:\\", Some(bad));
    assert_eq!(
        recommend(&a, 1920, 1080, None, None, None)[0].value,
        recommend(&b, 1920, 1080, None, None, None)[0].value
    );
}
#[test]
fn graphics_are_heuristics_and_accept_measured_tuning() {
    let r = report("reported", Some("amd"));
    let settings = recommend(&r, 2560, 1440, None, Some(30.), Some(60.));
    assert!(settings[0].confidence.starts_with("Low"));
    assert!(settings.iter().any(|s| s.setting == "Measured tuning"));
    assert!(settings
        .iter()
        .any(|s| s.setting == "Textures" && s.value == "Medium to High"));
    for preset in ["Very Low", "Low", "Medium", "High", "Ultra"] {
        assert_eq!(
            recommend(&r, 1920, 1080, Some(preset), None, None)[0].value,
            preset
        );
    }
}
#[test]
fn blank_profiles_never_inherit_icarus() {
    let p = profiles::new_profile("new-game".into());
    assert!(p.name.is_empty());
    assert!(p.minimum.cpu.is_empty());
    assert!(p.recommended.gpu.is_empty());
    assert!(p.minimum.ram.is_none());
    assert!(p.recommended.storage.is_none());
    assert!(p.provenance.source_url.is_none());
}
#[test]
fn legacy_migration_preserves_units_and_warns_about_templates() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy-profile.json")).unwrap();
    value["name"] = "AION 2".into();
    let p = profiles::parse(&value.to_string()).unwrap();
    assert_eq!(p.schema_version, 2);
    assert_eq!(p.name, "AION 2");
    assert_eq!(p.recommended.vram.unwrap().unit, CapacityUnit::Gib);
    assert_eq!(p.recommended.ssd, SsdPolicy::Recommended);
    assert!(p.migration_notes[0].contains("copied Icarus"));
}
#[test]
fn imports_reject_malformed_unknown_fields_and_bad_capacities() {
    assert!(profiles::parse("{}").is_err());
    let mut v = serde_json::to_value(profiles::sample()).unwrap();
    v["extra"] = true.into();
    assert!(profiles::parse(&v.to_string()).is_err());
    for n in [f64::NAN, f64::INFINITY, -1.] {
        let mut p = profiles::sample();
        p.minimum.ram = Some(Capacity {
            value: n,
            unit: CapacityUnit::Gb,
        });
        assert!(profiles::validate(&p).is_err());
    }
    for id in ["../bad", "C:\\bad", "CON", "com1", "LPT9", ""] {
        let mut p = profiles::sample();
        p.id = id.into();
        assert!(profiles::validate(&p).is_err());
    }
}
#[test]
fn provenance_requires_a_source_and_real_date() {
    let mut p = profiles::sample();
    p.provenance.kind = ProvenanceKind::Official;
    assert!(profiles::validate(&p).is_err());
    p.provenance.source_url = Some("https://example.org/game".into());
    p.provenance.verified_date = Some("2025-02-29".into());
    assert!(profiles::validate(&p).is_err());
    p.provenance.verified_date = Some("2024-02-29".into());
    profiles::validate(&p).unwrap();
}
#[test]
fn persistence_roundtrips_isolates_corrupt_files_and_honors_deletion() {
    let dir = std::env::temp_dir().join(format!("gamespec-v2-test-{}", std::process::id()));
    let mut p = profiles::sample();
    p.id = "test-game".into();
    profiles::save(&dir, &p).unwrap();
    p.name = "Changed".into();
    profiles::save(&dir, &p).unwrap();
    std::fs::write(dir.join("bad.json"), b"invalid").unwrap();
    let library = profiles::load(&dir).unwrap();
    assert_eq!(
        library
            .profiles
            .iter()
            .find(|p| p.id == "test-game")
            .unwrap()
            .name,
        "Changed"
    );
    assert_eq!(library.diagnostics.len(), 1);
    std::fs::remove_file(dir.join("icarus.json")).unwrap();
    assert!(!profiles::load(&dir)
        .unwrap()
        .profiles
        .iter()
        .any(|p| p.id == "icarus"));
    for entry in std::fs::read_dir(&dir).unwrap() {
        std::fs::remove_file(entry.unwrap().path()).unwrap();
    }
    std::fs::remove_dir(dir).unwrap();
}
#[test]
fn location_rejects_relative_and_unavailable_paths() {
    assert!(resolve_location("Games\\Icarus", &[]).is_err());
    assert!(resolve_location("", &[]).is_err());
}
#[test]
fn lower_is_better_metrics_are_explicit() {
    let mut c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (70., 80.),
        (90., 100.),
    );
    c.datasets[0].higher_is_better = false;
    c.datasets[0].metric = "Test-only latency".into();
    assert_eq!(
        c.compare(
            "gpu",
            Some("AMD Radeon RX 7600"),
            &["RTX 2070".into()],
            Workload::GpuRaster
        )
        .result,
        EvidenceResult::Pass
    );
}
#[test]
fn conflicting_datasets_cannot_confirm_equivalence() {
    let mut c = measured(
        "gpu",
        "AMD Radeon RX 7600",
        "NVIDIA GeForce RTX 2070",
        Workload::GpuRaster,
        (110., 120.),
        (90., 100.),
    );
    let mut d = c.datasets[0].clone();
    d.id = "conflicting-context".into();
    c.datasets.push(d);
    let additional: Vec<_> = c
        .measurements
        .iter()
        .map(|m| Measurement {
            dataset_id: "conflicting-context".into(),
            low: if m.low > 100. { 70. } else { 90. },
            high: if m.low > 100. { 80. } else { 100. },
            ..m.clone()
        })
        .collect();
    c.measurements.extend(additional);
    c.validate().unwrap();
    let result = c.compare(
        "gpu",
        Some("AMD Radeon RX 7600"),
        &["RTX 2070".into()],
        Workload::GpuRaster,
    );
    assert_eq!(result.result, EvidenceResult::Unknown);
    assert!(result.explanation.contains("disagree"));
}
#[test]
fn installation_destination_cannot_be_an_existing_file() {
    let path = std::env::current_dir().unwrap().join("Cargo.toml");
    assert!(resolve_location(path.to_str().unwrap(), &[])
        .unwrap_err()
        .contains("directory"));
}
