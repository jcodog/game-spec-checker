use crate::{Evidence, EvidenceResult};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub version: String,
    pub method: String,
    pub entries: Vec<Entry>,
}
#[derive(Debug, Deserialize)]
pub struct Entry {
    pub kind: String,
    pub name: String,
    pub aliases: Vec<String>,
    pub dataset: Option<String>,
    pub score: Option<f64>,
    pub source: String,
    pub label: String,
}
pub fn catalog() -> Catalog {
    serde_json::from_str(include_str!("../../../data/hardware-catalog.json"))
        .expect("bundled catalog must be valid")
}
fn normalized(name: &str) -> String {
    name.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn lookup<'a>(catalog: &'a Catalog, kind: &str, name: &str) -> Option<&'a Entry> {
    let name = normalized(name);
    catalog.entries.iter().find(|e| {
        e.kind == kind
            && (normalized(&e.name) == name || e.aliases.iter().any(|a| normalized(a) == name))
    })
}
pub fn compare(kind: &str, actual: Option<&str>, alternatives: &[String]) -> Evidence {
    let db = catalog();
    let Some(actual) = actual.filter(|s| !s.trim().is_empty()) else {
        return evidence(
            EvidenceResult::Unknown,
            "Hardware model unavailable.",
            "Native scan",
        );
    };
    if alternatives.is_empty() {
        return evidence(
            EvidenceResult::Unknown,
            "No required model was specified.",
            "Game profile",
        );
    }
    let mut outcomes = Vec::new();
    for required in alternatives {
        let a = lookup(&db, kind, actual);
        let b = lookup(&db, kind, required);
        if normalized(actual) == normalized(required)
            || matches!((a,b), (Some(a),Some(b)) if a.name == b.name)
        {
            return evidence(
                EvidenceResult::Pass,
                "Exact model identity. This verifies the named requirement, not FPS.",
                "Model identity",
            );
        }
        let e = match (a,b) {
            (Some(a),Some(b)) if a.dataset.is_some() && a.dataset == b.dataset => {
                match (a.score,b.score) {
                    (Some(x),Some(y)) if x > 0.0 && y > 0.0 => {
                        let ratio = x/y;
                        let result = if ratio >= 1.25 { EvidenceResult::Pass }
                            else if ratio <= 0.75 { EvidenceResult::Fail }
                            else { EvidenceResult::Unknown };
                        Evidence { result, explanation: format!("{} versus {}: synthetic score ratio {:.2}. Conservative 25% uncertainty band; approximate raster performance only. Game behavior may differ. {}", a.name,b.name,ratio,a.label), source:a.source.clone() }
                    },
                    _ => evidence(EvidenceResult::Unknown,"Comparable scores unavailable.","Catalog"),
                }
            },
            _ => evidence(EvidenceResult::Unknown, &format!("No comparable evidence for {actual} versus {required}. Names, clocks and generation numbers are not performance evidence."), "Offline hardware catalog"),
        };
        if e.result == EvidenceResult::Pass {
            return e;
        }
        outcomes.push(e);
    }
    let result = if outcomes.iter().all(|e| e.result == EvidenceResult::Fail) {
        EvidenceResult::Fail
    } else {
        EvidenceResult::Unknown
    };
    Evidence {
        result,
        explanation: outcomes
            .iter()
            .map(|e| e.explanation.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        source: outcomes
            .iter()
            .map(|e| e.source.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
    }
}
pub fn evidence(result: EvidenceResult, explanation: &str, source: &str) -> Evidence {
    Evidence {
        result,
        explanation: explanation.into(),
        source: source.into(),
    }
}
