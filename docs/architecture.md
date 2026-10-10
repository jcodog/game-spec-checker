# Architecture and evidence policy

The Rust core owns types, validation, migration, scanning, catalogue comparison,
assessment and recommendations. Tauri and CLI supply a validated profile,
native snapshot, explicit selected GPU, resolved volume and active catalogue.
React validates IPC shapes and renders the same assessment, never a second engine.

## Native detection

Retain sysinfo for CPU topology, usable/available memory and storage classification.
Installed physical RAM uses GetPhysicallyInstalledSystemMemory; a failed call
stays unavailable instead of substituting usable memory. CPU ISA flags are
informative and never performance rankings.

Direct WMI queries read Windows edition/build and drivers. DXGI enumerates
hardware adapters and outputs. DedicatedVideoMemory supplies dedicated VRAM;
shared RAM and AdapterRAM are excluded. D3D12CreateDevice probes each adapter's
feature levels independently. Unsupported baseline is a negative API result;
other API errors are unknown. GDI supplies current active display modes.

Resolve destinations through their nearest existing directory, canonicalise
junctions, then call GetVolumePathNameW. Relative paths, existing files and
unrecognised backing volumes are errors. Volume capacity, free bytes, file system,
media and removability belong to the actual backing volume. Scans run on a
blocking worker. No PowerShell or privileged access is used by the application.

## Profiles and units

Schema 2 uses optional typed Capacity values, independent SSD policies, provenance,
version/platform, source/date and CPU/GPU workloads. Null means missing evidence;
explicit zero means a zero capacity constraint. GB and GiB retain exact meanings.
Ambiguous GB produces an interval from decimal to binary bytes and stays unknown
when a native capacity lies between them. API bytes and marketed capacity are
separate properties. The scanner does not infer marketed memory by rounding.

Blank profiles originate in Rust and contain no requirements. Duplication is an
explicit frontend action with a new ID and user-entered provenance. Import always
opens a new-ID draft and never overwrites an existing profile. Rust remains the
authority for semantic validation, calendar dates and capacity consistency.

Schema 1 reads preserve the previous GiB interpretation and attach a review
warning about units and the former Icarus-copying bug. Writes occur only on save.
File identity must match its filename. Bad files produce diagnostics individually.
Unique temporary siblings and rename prevent partial saved JSON; a Tauri mutex
serialises writes. First-load seeding uses a marker so removed samples stay removed.

## Catalogue contract

Schema 2 separates devices, datasets and measurements. Device aliases are unique
per component kind and preserve variants. Dataset ID identifies a single comparable
test scale/context, not a publisher's entire site. Required metadata includes:

- Workload, metric and higher/lower-is-better direction.
- Publication date, source URL, test context and methodology.
- License/permission declaration and explicit redistribution clearance.
- Documented uncertainty method and decision eligibility.

Each measurement identifies a device/dataset, positive low/high interval and sample
count. These bounds must come from reviewed evidence, not guessed safety margins.
Two devices compare only within the same eligible dataset of the selected workload.
For higher-is-better, the performance ratio is actual.low/required.high through
actual.high/required.low. Lower-is-better inverts the ratio. The interval entirely
at or above 1 passes, entirely below 1 fails, and overlapping 1 stays unknown.
All available comparable contexts must agree to make a confirmed result.
Missing or conflicting evidence remains unknown. CPU gaming, single-thread and
multi-thread are distinct. GPU raster, ray tracing, synthetic and compute are distinct.

The catalogue validator checks structure, alias uniqueness, bounds, references and
metadata. It cannot authenticate claimed measurements or licenses. Local imports
therefore require source review before the desktop applies them. CLI operators
explicitly select reviewed data with --catalog. Invalid CLI data is an error.
Invalid saved desktop data is preserved, a visible diagnostic explains rejection,
and the bundled baseline is explicitly selected instead.

The baseline has 52 identities and no performance measurements. The earlier
synthetic scores were removed, not relabelled as game performance. This means
common cross-model CPU/GPU comparisons are still an outstanding product requirement.
Unit tests use explicitly labelled synthetic algorithm fixtures that are never
bundled into production. See the audit for source/licensing research.

## Verdicts and recommendations

Each component carries minimum/recommended evidence, confidence and mandatory
flags. Any confirmed mandatory failure fails that level. Otherwise any missing
critical evidence makes the level unknown. All mandatory checks must pass for a
pass. SSD Recommended is advisory and excluded; SSD Required participates.

Minimum Pass and Recommended Unknown retains Meets minimum, with recommended
uncertainty visible separately. Minimum failure gives Below minimum. Unknown
minimum stays unconfirmed. A recommended CPU identity does not invent evidence
against a different minimum CPU model.

Graphics recommendations use individual GPU/CPU checks, requirements-relative
VRAM, available RAM, selected pixel count and user measurements. Disk/OS failures
do not set quality. Suggestions are generic trials, not benchmark-based FPS or
verified game presets. Upscalers/frame generation/ray tracing remain disabled
until both hardware features and game support are verified. No such feature
verification or game-specific guidance database exists in this stage.

## UI and command boundaries

Dashboard, library, checker/report and settings are distinct screens. Views and
editor components are separate from application orchestration. Changed assessment
inputs invalidate the previous report. Async recommendation responses are ignored
when superseded. Errors are visible; missing scanner values are labelled.

Exports use a native Save dialog; no unrestricted filesystem command is exposed.
Profiles and catalogues are bounded JSON, and IDs cannot contain path separators
or reserved Windows device names. The frontend CSP allows local IPC and no hardware
network upload. Normal operation requires no administrator rights.

Full Windows detection and packaging target x64. The scanner retains cfg platform
boundaries for future ports. Other platforms report incomplete GPU/display/OS
support rather than fake Windows values.
