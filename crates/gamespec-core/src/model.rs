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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirements {
    pub cpu: Vec<String>,
    pub gpu: Vec<String>,
    pub ram_gib: f64,
    pub vram_gib: f64,
    pub windows_versions: Vec<u32>,
    pub architecture: String,
    pub graphics_api: String,
    pub feature_level: Option<String>,
    pub storage_gb: f64,
    pub ssd: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameProfile {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub minimum: Requirements,
    pub recommended: Requirements,
    pub notes: String,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub result: EvidenceResult,
    pub explanation: String,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentResult {
    pub component: String,
    pub verdict: Verdict,
    pub minimum: Evidence,
    pub recommended: Evidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assessment {
    pub profile: GameProfile,
    pub system: SystemSnapshot,
    pub gpu_id: Option<String>,
    pub destination: String,
    pub volume: Option<Volume>,
    pub headroom_bytes: Option<i128>,
    pub verdict: Verdict,
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
