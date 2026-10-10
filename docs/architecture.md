# Architecture

`gamespec-core` owns the serializable model, profile validation, hardware scanning,
offline catalog, compatibility engine and graphics recommendation rules.
`gamespec-cli` and the Tauri command layer call the same functions. React validates
IPC responses and profile imports with Zod, then renders native results.

## Detection

- sysinfo uses native platform implementations for CPU topology, available memory
  and volume information. Windows storage classification uses the storage
  descriptor. A missing or ambiguous classification stays Unknown.
- Installed RAM uses `GetPhysicallyInstalledSystemMemory`. Usable memory is not
  substituted when that call fails, since firmware reservation can make usable
  capacity smaller than the physically installed modules.
- CPU capabilities use Rust's runtime x86 feature detection. They are informative,
  not a performance ranking.
- WMI reads Windows edition, build and driver versions. WMI is queried directly
  through Rust, without launching PowerShell or WMIC.
- DXGI enumerates hardware adapters and attached outputs. Dedicated VRAM comes
  from `DXGI_ADAPTER_DESC1::DedicatedVideoMemory`, never shared memory or the
  truncated WMI `AdapterRAM` field.
- Direct3D 12 devices are probed on each DXGI adapter from feature level 12_2 down
  to 11_0. Only an explicit unsupported result at the baseline becomes a negative
  API result. Other errors remain Unknown. This is D3D12 support, not proof of
  every optional DirectX feature or every game's runtime dependencies.
- GDI enumerates active desktop display devices and their current modes.
- Installation paths resolve their nearest existing ancestor, canonicalize
  junctions, then call `GetVolumePathNameW`. A missing backing volume is an error,
  never a guess based on a drive-letter prefix.

Scan commands run on a blocking background worker. The UI exposes missing values
and diagnostics. It requires explicit GPU selection on multi-GPU systems.

## Evidence and aggregation

Model identification uses exact strings and explicit aliases. It preserves
variant suffixes, including SUPER, XT and laptop variants. No substring matching
or inferred generation ordering is allowed.

The initial GPU dataset contains a few factual Time Spy Graphics measurements
from Tweakers' 24 May 2023 RX 7600 review. Scores are used only within that test
set. The ratio must be at least 1.25 to pass or at most 0.75 to fail. Intermediate
ratios are Unknown. The band is an application policy, not a statistical
confidence interval. Synthetic performance is an approximation of game
performance and says nothing about ray tracing or proprietary API features.

RX 7600 versus RTX 2070 has a ratio of about 1.19, so it is Unknown. The GTX 1050 Ti
and initial CPUs have identity entries only. Cross-model CPU equivalence and
comparisons involving that GTX are Unknown until suitable evidence is added.

An alternative requirement passes when any alternative passes. It fails only
when all alternatives have evidence of failure. A missing alternative leaves
the result Unknown unless another alternative passes.

Each component retains independent minimum and recommended evidence. A known
minimum failure produces Below minimum even if other components are unknown.
All minimum and recommended checks must pass to claim Meets recommended.
Minimum passes with a known recommended failure produce Meets minimum.
Otherwise the headline stays Unknown, with confirmed minimum checks still shown.
Recommended equivalence never silently proves a different minimum model.

RAM and VRAM use GiB, storage uses decimal GB. Installed RAM is assessed;
available RAM can add a warning. Headroom subtracts the recommended storage
requirement from free space and excludes updates and installer temporary files.

## Boundaries and persistence

Rust validates profiles on import, save and assessment. IDs cannot contain path
separators. Unknown schema versions and fields are rejected. Numbers must be
finite and bounded. Recommended capacities cannot be lower than minimum.

Tauri stores profiles under the current user's application data directory.
Writes use a temporary sibling and rename, serialized with a store mutex. The
Icarus sample is available even before any files exist. Corrupt saved profiles
produce a visible error with their path rather than silently disappearing.
Export destinations come from a native Save dialog, not an unrestricted frontend
filesystem API. Profile imports use an explicitly selected local file.

The application has no network code for hardware data, telemetry or profile
lookups. CSP limits frontend connections to Tauri IPC. Benchmark links open only
when selected by the user. Reports include hardware and installation paths.

## Platform extension

The model and engine are independent of Windows. Windows-specific scanning is
behind `cfg(windows)`. Other platforms return basic sysinfo measurements and
explicitly report unimplemented GPU, display and OS support. The shipped
desktop product and workflows currently target Windows x64 only.
