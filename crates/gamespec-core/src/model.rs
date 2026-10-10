use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cpu {
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub architecture: Option<String>,
    pub physical_cores: Option<usize>,
    pub logical_threads: Option<usize>,
    pub capabilities: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Gpu {
    pub id: String,
    pub model: String,
    pub manufacturer: String,
    pub dedicated_bytes: Option<u64>,
    pub driver_version: Option<String>,
    pub active: bool,
    pub directx12: Option<bool>,
    pub feature_level: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Ssd,
    Hdd,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub mount: String,
    pub name: String,
    pub total_bytes: u64,
    pub free_bytes: u64,
    pub file_system: String,
    pub media: MediaKind,
    #[serde(default)]
    pub removable: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub refresh_hz: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OperatingSystem {
    pub edition: Option<String>,
    pub version: Option<String>,
    pub build: Option<u32>,
    pub architecture: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemSnapshot {
    pub cpu: Cpu,
    pub ram_total_bytes: Option<u64>,
    pub ram_available_bytes: Option<u64>,
    pub gpus: Vec<Gpu>,
    pub os: OperatingSystem,
    pub volumes: Vec<Volume>,
    pub displays: Vec<Display>,
    pub diagnostics: Vec<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CapacityUnit {
    Gb,
    Gib,
    AmbiguousGb,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Capacity {
    pub value: f64,
    pub unit: CapacityUnit,
}
impl Capacity {
    pub fn bounds(self) -> (f64, f64) {
        match self.unit {
            CapacityUnit::Gb => (self.value * 1e9, self.value * 1e9),
            CapacityUnit::Gib => (self.value * 1073741824.0, self.value * 1073741824.0),
            CapacityUnit::AmbiguousGb => (self.value * 1e9, self.value * 1073741824.0),
        }
    }
    pub fn label(self) -> String {
        format!(
            "{} {}",
            self.value,
            match self.unit {
                CapacityUnit::Gb => "GB",
                CapacityUnit::Gib => "GiB",
                CapacityUnit::AmbiguousGb => "GB (unit ambiguous)",
            }
        )
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SsdPolicy {
    Unspecified,
    Recommended,
    Required,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Workload {
    CpuGaming,
    CpuSingleThread,
    CpuMultiThread,
    GpuRaster,
    GpuRayTracing,
    GpuSynthetic,
    GpuCompute,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    pub cpu: Vec<String>,
    pub gpu: Vec<String>,
    pub ram: Option<Capacity>,
    pub vram: Option<Capacity>,
    pub windows_versions: Vec<u32>,
    pub architecture: Option<String>,
    pub graphics_api: Option<String>,
    pub feature_level: Option<String>,
    pub storage: Option<Capacity>,
    pub ssd: SsdPolicy,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceKind {
    Official,
    Community,
    UserEntered,
    Unverified,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub kind: ProvenanceKind,
    pub source_url: Option<String>,
    pub verified_date: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub provenance: Provenance,
    pub cpu_workload: Workload,
    pub gpu_workload: Workload,
    pub minimum: Requirements,
    pub recommended: Requirements,
    pub installation_headroom: Option<Capacity>,
    pub notes: String,
    pub migration_notes: Vec<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    MeetsRecommended,
    MeetsMinimum,
    BelowMinimum,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceResult {
    Pass,
    Fail,
    Unknown,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub result: EvidenceResult,
    pub confidence: Confidence,
    pub explanation: String,
    pub source: String,
    pub ratio_interval: Option<[f64; 2]>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentResult {
    pub component: String,
    pub verdict: Verdict,
    pub minimum: Evidence,
    pub recommended: Evidence,
    pub minimum_required: bool,
    pub recommended_required: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelSummary {
    pub result: EvidenceResult,
    pub passed: usize,
    pub failed: usize,
    pub unknown: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub schema_version: u32,
    pub profile: GameProfile,
    pub system: SystemSnapshot,
    pub gpu_id: Option<String>,
    pub destination: String,
    pub volume: Option<Volume>,
    pub headroom_bytes: Option<i128>,
    pub verdict: Verdict,
    pub minimum: LevelSummary,
    pub recommended: LevelSummary,
    pub components: Vec<ComponentResult>,
    pub warnings: Vec<String>,
    pub catalog_version: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingRecommendation {
    pub setting: String,
    pub value: String,
    pub reason: String,
    pub confidence: String,
}
