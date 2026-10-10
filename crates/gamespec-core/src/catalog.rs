use crate::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub schema_version: u32,
    pub version: String,
    pub devices: Vec<Device>,
    pub datasets: Vec<Dataset>,
    pub measurements: Vec<Measurement>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub advertised_memory: Option<Capacity>,
    pub specification_url: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dataset {
    pub id: String,
    pub title: String,
    pub workload: Workload,
    pub metric: String,
    pub test_context: String,
    pub higher_is_better: bool,
    pub methodology: String,
    pub source_url: String,
    pub published_date: String,
    pub license: String,
    pub redistribution_cleared: bool,
    pub decision_eligible: bool,
    pub uncertainty_method: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Measurement {
    pub device_id: String,
    pub dataset_id: String,
    pub low: f64,
    pub high: f64,
    pub samples: u32,
}
#[derive(Debug, Serialize)]
pub struct CatalogSummary {
    pub version: String,
    pub devices: usize,
    pub measurements: usize,
    pub decision_measurements: usize,
    pub datasets: Vec<Dataset>,
    pub diagnostics: Vec<String>,
}
pub fn normalize(model: &str) -> String {
    let lower = model
        .to_lowercase()
        .replace("(r)", "")
        .replace("(tm)", "")
        .replace(['®', '™'], "");
    let stem = lower.split('@').next().unwrap_or(&lower);
    let mut words: Vec<_> = stem.split_whitespace().collect();
    if words.len() > 2
        && words[1] == "gen"
        && words[0].chars().next().is_some_and(|c| c.is_ascii_digit())
    {
        words.drain(0..2);
    }
    while words
        .last()
        .is_some_and(|w| ["cpu", "processor"].contains(w))
    {
        words.pop();
    }
    if words.last().is_some_and(|w| {
        w.ends_with("-core")
            && (w[..w.len() - 5].chars().all(|c| c.is_ascii_digit())
                || ["dual-core", "quad-core", "six-core", "eight-core"].contains(w))
    }) {
        words.pop();
    }
    words.join(" ")
}
impl Catalog {
    pub fn lookup(&self, kind: &str, name: &str) -> Option<&Device> {
        let name = normalize(name);
        self.devices.iter().find(|d| {
            d.kind == kind
                && (normalize(&d.name) == name || d.aliases.iter().any(|a| normalize(a) == name))
        })
    }
    pub fn summary(&self) -> CatalogSummary {
        let eligible: HashSet<_> = self
            .datasets
            .iter()
            .filter(|d| d.decision_eligible && d.redistribution_cleared)
            .map(|d| d.id.as_str())
            .collect();
        CatalogSummary {
            version: self.version.clone(),
            devices: self.devices.len(),
            measurements: self.measurements.len(),
            decision_measurements: self
                .measurements
                .iter()
                .filter(|m| eligible.contains(m.dataset_id.as_str()))
                .count(),
            datasets: self.datasets.clone(),
            diagnostics: if eligible.is_empty() {
                vec!["No decision-eligible gaming measurements in this catalog. Import a trusted, licensed catalog to expand coverage. Identity matches and native capacities still work.".into()]
            } else {
                vec![]
            },
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2
            || self.version.trim().is_empty()
            || self.version.len() > 80
            || self.devices.len() > 10000
            || self.datasets.len() > 500
            || self.measurements.len() > 50000
        {
            return Err("Invalid or oversized catalog manifest.".into());
        }
        let mut ids = HashSet::new();
        let mut aliases = HashSet::new();
        for d in &self.devices {
            if !profiles::valid_id(&d.id)
                || !ids.insert(d.id.as_str())
                || !["cpu", "gpu"].contains(&d.kind.as_str())
                || d.name.trim().is_empty()
                || d.name.len() > 250
                || d.aliases.len() > 50
            {
                return Err("Invalid or duplicate hardware identity.".into());
            }
            let mut local = HashSet::new();
            for name in std::iter::once(&d.name).chain(d.aliases.iter()) {
                if name.trim().is_empty() || name.len() > 250 {
                    return Err("Invalid hardware alias.".into());
                }
                let key = (d.kind.clone(), normalize(name));
                if local.insert(key.clone()) && !aliases.insert(key) {
                    return Err("An alias maps to more than one device. Catalog rejected.".into());
                }
            }
            if let Some(memory) = d.advertised_memory {
                profiles::validate_capacity(memory)?;
                if d.kind != "gpu" || d.specification_url.is_none() {
                    return Err("Advertised GPU capacity requires a specification source.".into());
                }
            }
            if d.specification_url
                .as_deref()
                .is_some_and(|u| !profiles::valid_url(u))
            {
                return Err("Invalid specification URL.".into());
            }
        }
        let mut datasets = HashSet::new();
        for d in &self.datasets {
            if !profiles::valid_id(&d.id)
                || !datasets.insert(d.id.as_str())
                || !profiles::valid_url(&d.source_url)
                || !profiles::valid_date(&d.published_date)
                || d.title.trim().is_empty()
                || d.title.len() > 300
                || d.methodology.trim().is_empty()
                || d.metric.trim().is_empty()
                || d.metric.len() > 200
                || d.test_context.trim().is_empty()
                || d.test_context.len() > 3000
                || d.methodology.len() > 5000
                || d.uncertainty_method.trim().is_empty()
                || d.uncertainty_method.len() > 3000
                || d.license.trim().is_empty()
                || d.license.len() > 200
            {
                return Err("Dataset requires unique ID, source, publication date, methodology, uncertainty and license.".into());
            }
            if d.decision_eligible
                && (!d.redistribution_cleared
                    || ![
                        "CC0-1.0",
                        "CC-BY-4.0",
                        "CC-BY-SA-4.0",
                        "MIT",
                        "explicit-permission",
                    ]
                    .contains(&d.license.as_str()))
            {
                return Err("Decision data must have an explicit reusable license or permission. Source claims still require review.".into());
            }
        }
        let mut keys = HashSet::new();
        for m in &self.measurements {
            if !ids.contains(m.device_id.as_str())
                || !datasets.contains(m.dataset_id.as_str())
                || !keys.insert((&m.device_id, &m.dataset_id))
                || !m.low.is_finite()
                || !m.high.is_finite()
                || m.low <= 0.0
                || m.high < m.low
                || m.high > 1e12
                || m.samples == 0
            {
                return Err(
                    "Invalid interval, duplicate measurement or dangling catalog reference.".into(),
                );
            }
            let device = self
                .devices
                .iter()
                .find(|d| d.id == m.device_id)
                .ok_or("Missing device.")?;
            let dataset = self
                .datasets
                .iter()
                .find(|d| d.id == m.dataset_id)
                .ok_or("Missing dataset.")?;
            let cpu = matches!(
                dataset.workload,
                Workload::CpuGaming | Workload::CpuSingleThread | Workload::CpuMultiThread
            );
            if cpu != (device.kind == "cpu") {
                return Err("Measurement workload does not match hardware kind.".into());
            }
        }
        Ok(())
    }
    pub fn compare(
        &self,
        kind: &str,
        actual: Option<&str>,
        required: &[String],
        workload: Workload,
    ) -> Evidence {
        let Some(actual) = actual.filter(|s| !s.trim().is_empty()) else {
            return evidence(
                EvidenceResult::Unknown,
                "Hardware model was not detected.",
                "Native scan",
            );
        };
        if required.is_empty() {
            return evidence(
                EvidenceResult::Unknown,
                "This profile does not specify a required model. Add genuine requirements.",
                "Game profile",
            );
        }
        let mut alternatives = Vec::new();
        for requirement in required {
            let a = self.lookup(kind, actual);
            let b = self.lookup(kind, requirement);
            if normalize(actual) == normalize(requirement)
                || matches!((a,b),(Some(a),Some(b)) if a.id==b.id)
            {
                return evidence(
                    EvidenceResult::Pass,
                    "Detected model matches a named alternative. This does not predict FPS.",
                    "Exact hardware identity",
                );
            }
            let mut comparisons = Vec::new();
            if let (Some(a), Some(b)) = (a, b) {
                for dataset in self.datasets.iter().filter(|d| {
                    d.workload == workload && d.decision_eligible && d.redistribution_cleared
                }) {
                    let left = self
                        .measurements
                        .iter()
                        .find(|m| m.device_id == a.id && m.dataset_id == dataset.id);
                    let right = self
                        .measurements
                        .iter()
                        .find(|m| m.device_id == b.id && m.dataset_id == dataset.id);
                    if let (Some(left), Some(right)) = (left, right) {
                        let range = if dataset.higher_is_better {
                            [left.low / right.high, left.high / right.low]
                        } else {
                            [right.low / left.high, right.high / left.low]
                        };
                        let result = if range[0] >= 1.0 {
                            EvidenceResult::Pass
                        } else if range[1] < 1.0 {
                            EvidenceResult::Fail
                        } else {
                            EvidenceResult::Unknown
                        };
                        comparisons.push(Evidence {result,confidence:if result==EvidenceResult::Unknown {Confidence::Low} else {Confidence::Medium},explanation:format!("{actual} versus {requirement}: the comparable {:?} performance interval is {:.2} to {:.2} times the requirement. Metric: {}, higher is better: {}. {} Neither this proxy nor a capacity match guarantees game performance.",workload,range[0],range[1],dataset.metric,dataset.higher_is_better,dataset.uncertainty_method),source:format!("{}\n{}\nPublished {}; {}; {}\nTest context: {}",dataset.title,dataset.source_url,dataset.published_date,dataset.license,dataset.methodology,dataset.test_context),ratio_interval:Some(range)});
                    }
                }
            }
            let e = if comparisons.is_empty() {
                evidence(EvidenceResult::Unknown,&format!("No cleared, comparable {workload:?} measurements for {actual} versus {requirement}. Import a trusted catalog containing both models in the same test context; single-thread, multicore, raster, ray tracing and synthetic scales cannot substitute for each other."),"Catalog coverage")
            } else if comparisons[0].result != EvidenceResult::Unknown
                && comparisons
                    .iter()
                    .all(|e| e.result == comparisons[0].result)
            {
                let mut combined = comparisons[0].clone();
                combined.source = comparisons
                    .iter()
                    .map(|e| e.source.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                combined.explanation = comparisons
                    .iter()
                    .map(|e| e.explanation.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                if comparisons.len() > 1 {
                    combined.ratio_interval = None;
                }
                combined
            } else {
                evidence(EvidenceResult::Unknown,"Comparable datasets overlap or disagree. Review the evidence instead of treating a near match as confirmed.",&comparisons.iter().map(|e|e.source.as_str()).collect::<Vec<_>>().join("\n"))
            };
            if e.result == EvidenceResult::Pass {
                return e;
            }
            alternatives.push(e);
        }
        let result = if alternatives
            .iter()
            .all(|e| e.result == EvidenceResult::Fail)
        {
            EvidenceResult::Fail
        } else {
            EvidenceResult::Unknown
        };
        Evidence {
            result,
            confidence: if result == EvidenceResult::Fail {
                Confidence::Medium
            } else {
                Confidence::Unknown
            },
            explanation: alternatives
                .iter()
                .map(|e| e.explanation.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            source: alternatives
                .iter()
                .map(|e| e.source.as_str())
                .collect::<Vec<_>>()
                .join("\n"),
            ratio_interval: None,
        }
    }
}
pub fn evidence(result: EvidenceResult, explanation: &str, source: &str) -> Evidence {
    Evidence {
        result,
        confidence: if result == EvidenceResult::Unknown {
            Confidence::Unknown
        } else {
            Confidence::High
        },
        explanation: explanation.into(),
        source: source.into(),
        ratio_interval: None,
    }
}
pub fn parse(json: &str) -> Result<Catalog, String> {
    if json.len() > 10_000_000 {
        return Err("Catalog exceeds 10 MB.".into());
    }
    let catalog: Catalog = serde_json::from_str(json).map_err(|e| e.to_string())?;
    catalog.validate()?;
    Ok(catalog)
}
pub fn catalog() -> Catalog {
    parse(include_str!("../../../data/hardware-catalog.json")).expect("validated bundled catalog")
}
pub fn load(path: Option<&Path>) -> Result<Catalog, String> {
    match path {
        Some(path) => {
            if fs::metadata(path).map_err(|e| e.to_string())?.len() > 10_000_000 {
                return Err("Catalog exceeds 10 MB.".into());
            }
            parse(&fs::read_to_string(path).map_err(|e| e.to_string())?)
        }
        None => Ok(catalog()),
    }
}
