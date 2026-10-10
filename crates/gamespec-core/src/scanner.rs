use crate::*;
use std::path::Path;
use sysinfo::{DiskKind, Disks, System};

pub fn scan() -> SystemSnapshot {
    let mut system = System::new();
    system.refresh_memory();
    system.refresh_cpu_all();
    let cpu = system.cpus().first();
    let mut snapshot = SystemSnapshot {
        cpu: Cpu {
            model: cpu
                .map(|c| c.brand().trim().to_string())
                .filter(|s| !s.is_empty()),
            manufacturer: cpu.map(|c| c.vendor_id().to_string()),
            architecture: Some(System::cpu_arch()),
            physical_cores: System::physical_core_count(),
            logical_threads: (!system.cpus().is_empty()).then_some(system.cpus().len()),
            capabilities: capabilities(),
        },
        ram_total_bytes: (system.total_memory() > 0).then_some(system.total_memory()),
        ram_available_bytes: Some(system.available_memory()),
        os: OperatingSystem {
            edition: System::name(),
            version: System::os_version(),
            build: None,
            architecture: Some(System::cpu_arch()),
        },
        volumes: Disks::new_with_refreshed_list()
            .iter()
            .filter(|d| d.total_space() > 0)
            .map(|d| Volume {
                mount: d.mount_point().to_string_lossy().to_string(),
                name: d.name().to_string_lossy().to_string(),
                total_bytes: d.total_space(),
                free_bytes: d.available_space(),
                file_system: d.file_system().to_string_lossy().to_string(),
                media: match d.kind() {
                    DiskKind::SSD => MediaKind::Ssd,
                    DiskKind::HDD => MediaKind::Hdd,
                    _ => MediaKind::Unknown,
                },
                removable: Some(d.is_removable()),
            })
            .collect(),
        ..Default::default()
    };
    #[cfg(windows)]
    windows_scan::enrich(&mut snapshot);
    #[cfg(not(windows))]
    snapshot.diagnostics.push("GPU, display and Windows detection is implemented only on Windows. Platform extension point: scanner module.".into());
    snapshot
}
fn capabilities() -> Vec<String> {
    let mut flags = Vec::new();
    #[cfg(target_arch = "x86_64")]
    {
        for (supported, name) in [
            (std::is_x86_feature_detected!("sse4.2"), "SSE4.2"),
            (std::is_x86_feature_detected!("avx"), "AVX"),
            (std::is_x86_feature_detected!("avx2"), "AVX2"),
            (std::is_x86_feature_detected!("fma"), "FMA"),
        ] {
            if supported {
                flags.push(name.into());
            }
        }
    }
    flags
}
pub fn resolve_location(path: &str, volumes: &[Volume]) -> Result<Volume, String> {
    if path.trim().is_empty() {
        return Err("Choose an installation volume or an absolute path.".into());
    }
    let path = Path::new(path);
    if !path.is_absolute() {
        return Err("Installation path must be absolute.".into());
    }
    if path.exists() && !path.is_dir() {
        return Err("Installation destination must be a directory, not a file.".into());
    }
    // Resolve the nearest existing parent, including junctions and mount points.
    let existing = path
        .ancestors()
        .find(|p| p.is_dir())
        .ok_or("No existing parent for installation path.")?;
    let canonical = existing
        .canonicalize()
        .map_err(|e| format!("Cannot resolve installation path: {e}"))?;
    #[cfg(windows)]
    {
        let mount = windows_scan::volume_mount(&canonical)?;
        let normalize = |s: &str| {
            s.strip_prefix("\\\\?\\")
                .unwrap_or(s)
                .trim_end_matches(['\\', '/'])
                .to_lowercase()
        };
        if let Some(volume) = volumes
            .iter()
            .find(|v| normalize(&v.mount) == normalize(&mount))
        {
            return Ok(volume.clone());
        }
        Err(format!(
            "The backing volume {mount} was not detected. Rescan after connecting the drive."
        ))
    }
    #[cfg(not(windows))]
    {
        volumes
            .iter()
            .filter(|v| canonical.starts_with(&v.mount))
            .max_by_key(|v| v.mount.len())
            .cloned()
            .ok_or("Backing volume not detected.".into())
    }
}

#[cfg(windows)]
mod windows_scan {
    use super::*;
    use serde::Deserialize;
    use windows::{
        core::PCWSTR,
        Win32::{
            Graphics::{Direct3D::*, Direct3D12::*, Dxgi::*, Gdi::*},
            Storage::FileSystem::GetVolumePathNameW,
            System::SystemInformation::GetPhysicallyInstalledSystemMemory,
        },
    };

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct OsRow {
        caption: Option<String>,
        version: Option<String>,
        build_number: Option<String>,
        #[serde(rename = "OSArchitecture")]
        os_architecture: Option<String>,
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct VideoRow {
        name: Option<String>,
        driver_version: Option<String>,
    }
    fn wide_string(buf: &[u16]) -> String {
        String::from_utf16_lossy(&buf[..buf.iter().position(|x| *x == 0).unwrap_or(buf.len())])
    }

    pub fn enrich(snapshot: &mut SystemSnapshot) {
        let mut installed_kib = 0u64;
        // SAFETY: Win32 writes a single u64 to the valid local output pointer.
        match unsafe { GetPhysicallyInstalledSystemMemory(&mut installed_kib) } {
            Ok(()) if installed_kib > 0 => {
                snapshot.ram_total_bytes = installed_kib.checked_mul(1024)
            }
            Ok(()) => {
                snapshot.ram_total_bytes = None;
                snapshot
                    .diagnostics
                    .push("Installed physical RAM was reported as zero.".into());
            }
            Err(e) => {
                snapshot.ram_total_bytes = None;
                snapshot.diagnostics.push(format!("Installed physical RAM unavailable: {e}. Usable memory is not substituted for installed module capacity."));
            }
        }
        match wmi::WMIConnection::new() {
            Ok(connection) => {
                match connection.raw_query::<OsRow>("SELECT Caption, Version, BuildNumber, OSArchitecture FROM Win32_OperatingSystem") {
                    Ok(rows) => if let Some(row) = rows.into_iter().next() {
                        snapshot.os.edition=row.caption;
                        snapshot.os.version=row.version;
                        snapshot.os.build=row.build_number.and_then(|s|s.parse().ok());
                        if let Some(architecture)=row.os_architecture { if architecture.contains("64") { snapshot.os.architecture=Some("x86_64".into()); } }
                    },
                    Err(e) => snapshot.diagnostics.push(format!("Windows WMI query failed: {e}")),
                }
                match adapters() {
                    Ok(gpus) => snapshot.gpus = gpus,
                    Err(e) => snapshot
                        .diagnostics
                        .push(format!("DXGI enumeration failed: {e}")),
                }
                match connection
                    .raw_query::<VideoRow>("SELECT Name, DriverVersion FROM Win32_VideoController")
                {
                    Ok(rows) => {
                        for gpu in &mut snapshot.gpus {
                            let matches: Vec<_> = rows
                                .iter()
                                .filter(|r| {
                                    r.name.as_deref().is_some_and(|n| {
                                        n.trim().eq_ignore_ascii_case(gpu.model.trim())
                                    })
                                })
                                .collect();
                            if matches.len() == 1 {
                                gpu.driver_version = matches[0].driver_version.clone();
                            }
                        }
                    }
                    Err(e) => snapshot
                        .diagnostics
                        .push(format!("GPU driver WMI query failed: {e}")),
                }
            }
            Err(e) => {
                snapshot.diagnostics.push(format!("WMI unavailable: {e}"));
                match adapters() {
                    Ok(gpus) => snapshot.gpus = gpus,
                    Err(e) => snapshot.diagnostics.push(format!("DXGI unavailable: {e}")),
                }
            }
        }
        snapshot.displays = displays();
        if snapshot.displays.is_empty() {
            snapshot
                .diagnostics
                .push("No active displays detected through EnumDisplaySettingsW.".into());
        }
        for gpu in &snapshot.gpus {
            if gpu.directx12.is_none() {
                snapshot.diagnostics.push(format!("Direct3D 12 probe unavailable for {}. Driver failure is not proof of missing support.",gpu.model));
            }
        }
    }
    fn adapters() -> windows::core::Result<Vec<Gpu>> {
        // SAFETY: COM interface wrappers own their references; API output buffers are initialized.
        unsafe {
            let factory: IDXGIFactory1 = CreateDXGIFactory1()?;
            let mut gpus = Vec::new();
            let mut index = 0;
            loop {
                let adapter = match factory.EnumAdapters1(index) {
                    Ok(a) => a,
                    Err(e) if e.code() == DXGI_ERROR_NOT_FOUND => break,
                    Err(e) => return Err(e),
                };
                index += 1;
                let desc = adapter.GetDesc1()?;
                if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
                    continue;
                }
                let mut active = false;
                let mut output_index = 0;
                while let Ok(output) = adapter.EnumOutputs(output_index) {
                    if let Ok(desc) = output.GetDesc() {
                        active |= desc.AttachedToDesktop.as_bool();
                    }
                    output_index += 1;
                }
                let levels = [
                    (D3D_FEATURE_LEVEL_12_2, "12_2"),
                    (D3D_FEATURE_LEVEL_12_1, "12_1"),
                    (D3D_FEATURE_LEVEL_12_0, "12_0"),
                    (D3D_FEATURE_LEVEL_11_1, "11_1"),
                    (D3D_FEATURE_LEVEL_11_0, "11_0"),
                ];
                let mut feature_level = None;
                let mut dx12 = None;
                for (level, name) in levels {
                    let mut device: Option<ID3D12Device> = None;
                    match D3D12CreateDevice(&adapter, level, &mut device) {
                        Ok(()) => {
                            dx12 = Some(true);
                            feature_level = Some(name.into());
                            break;
                        }
                        Err(e)
                            if level == D3D_FEATURE_LEVEL_11_0
                                && e.code() == DXGI_ERROR_UNSUPPORTED =>
                        {
                            dx12 = Some(false)
                        }
                        Err(_) => {}
                    }
                }
                gpus.push(Gpu {
                    id: format!("{}:{}", desc.AdapterLuid.HighPart, desc.AdapterLuid.LowPart),
                    model: wide_string(&desc.Description),
                    manufacturer: match desc.VendorId {
                        0x1002 => "AMD",
                        0x10de => "NVIDIA",
                        0x8086 => "Intel",
                        _ => "Unknown",
                    }
                    .into(),
                    // DedicatedVideoMemory is deliberately used, never SharedSystemMemory or WMI AdapterRAM.
                    dedicated_bytes: Some(desc.DedicatedVideoMemory as u64),
                    driver_version: None,
                    active,
                    directx12: dx12,
                    feature_level,
                });
            }
            Ok(gpus)
        }
    }
    fn displays() -> Vec<Display> {
        let mut displays = Vec::new();
        let mut index = 0;
        // SAFETY: size fields and writable buffers follow the documented Win32 contracts.
        unsafe {
            loop {
                let mut device = DISPLAY_DEVICEW {
                    cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                    ..Default::default()
                };
                if !EnumDisplayDevicesW(None, index, &mut device, 0).as_bool() {
                    break;
                }
                index += 1;
                if (device.StateFlags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP).0 == 0 {
                    continue;
                }
                let mut mode = DEVMODEW {
                    dmSize: std::mem::size_of::<DEVMODEW>() as u16,
                    ..Default::default()
                };
                if EnumDisplaySettingsW(
                    PCWSTR(device.DeviceName.as_ptr()),
                    ENUM_CURRENT_SETTINGS,
                    &mut mode,
                )
                .as_bool()
                {
                    displays.push(Display {
                        name: wide_string(&device.DeviceName),
                        width: mode.dmPelsWidth,
                        height: mode.dmPelsHeight,
                        refresh_hz: mode.dmDisplayFrequency,
                    });
                }
            }
        }
        displays
    }
    pub fn volume_mount(path: &Path) -> Result<String, String> {
        use std::os::windows::ffi::OsStrExt;
        let input: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut output = vec![0u16; 32768];
        // SAFETY: input is null terminated; output remains valid for the duration of the call.
        unsafe {
            GetVolumePathNameW(PCWSTR(input.as_ptr()), &mut output)
                .map_err(|e| format!("Backing volume query failed: {e}"))?;
        }
        Ok(wide_string(&output))
    }
}
