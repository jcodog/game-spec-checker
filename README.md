# GameSpec Checker

An offline Windows desktop app that scans your hardware and compares it with
game requirements you enter. Rust handles scanning and assessment. Tauri 2 and
React provide the interface. A CLI uses the same Rust core.

GameSpec reports **Meets recommended**, **Meets minimum**, **Below minimum**, or
**Unknown / insufficient evidence**. Every component includes its minimum and
recommended evidence. It does not predict FPS.

## Supported systems

Windows 10/11 x64 with Microsoft Edge WebView2. Normal operation and the
current-user installer do not require administrator permissions. GPU detection
uses DXGI and Direct3D, with direct WMI queries for Windows and driver details.
PowerShell is not used by the application to scan hardware.

The shared core has a platform extension point, but full detection and the
desktop release currently target Windows only. Older Windows versions, Windows
Server and ARM64 are not supported targets.

## Installation

If a release has been published, download the GameSpec Checker Windows setup
executable from this repository's Releases page, or download a build artifact
from the Windows production build workflow. Install for your current user, then
open GameSpec Checker. WebView2 must be present; the installer can download its
bootstrapper when needed. Runtime scanning and assessment work offline.

The build is unsigned. Signing requires a publisher certificate and is not
configured. For a local build, follow Development below. A release workflow
definition is not proof that a release was published.

## Using the application

1. Open Your system and let the native scan finish. Review the CPU, installed
   physical RAM, dedicated VRAM, Direct3D support, volumes and displays.
2. Choose a game from the library or add its minimum and recommended
   requirements. Put CPU/GPU alternatives on separate lines.
3. Select your intended gaming GPU. Multi-GPU systems require an explicit
   choice. The adapter driving a display may differ from the one used by a game.
4. Select a volume or enter an absolute installation path. Windows resolves the
   backing volume through existing parent directories, junctions and mount points.
5. Check compatibility. Expand component rows to inspect the evidence. Review
   storage headroom and SSD warnings.
6. Try the graphics recommendations for 1080p or 1440p, select a connected
   display, and optionally enter observed and target FPS to guide further tuning.

Profiles can be imported/exported as JSON. Import opens the editor for review
before saving. Copy and export buttons include the complete JSON assessment,
hardware snapshot, destination and source references. Export uses a native Save
dialog. The app never installs games, deletes game files or changes partitions.

The editable Icarus sample uses the requirements supplied in the project
request. It is not an automatically updated publisher requirements database.
Verify requirements for your game's version.

## Accuracy and limitations

- Exact model identity or an explicit catalog alias verifies a named model.
  Model numbers, vendor, generations, cores and clocks never rank performance.
- The initial comparative GPU dataset contains six attributed Time Spy Graphics
  scores from [Tweakers' RX 7600 review, 24 May 2023](https://tweakers.net/reviews/11132/amd-radeon-rx-7600-weinig-vernieuwend-amper-sneller.html).
  Comparisons are approximations. A ratio of at least 1.25 passes; a ratio of at
  most 0.75 fails. Intermediate ratios are Unknown. The 25% band is a product
  policy, not a statistical confidence interval.
- RX 7600 versus RTX 2070 is about 1.19 in that dataset, so GameSpec reports
  Unknown rather than failing because the vendors differ or claiming equivalence.
- The initial CPU catalog supports exact identities only. Cross-model CPU
  equivalence remains Unknown. GTX 1050 Ti has an identity entry but no comparable
  score in this test set. Icarus can therefore contain Unknown CPU/GPU checks on
  otherwise capable PCs. The report identifies the missing evidence.
- Missing data, uncatalogued hardware, laptop variants and incompatible datasets
  stay Unknown. Recommended identity does not silently prove a different minimum
  model. A known minimum failure still produces Below minimum.
- Dedicated VRAM comes from DXGI `DedicatedVideoMemory`, never total available
  graphics memory, shared RAM or WMI's 32-bit `AdapterRAM` field.
- Direct3D 12 device support and the maximum probed feature level are measured
  separately. DirectX 11 is confirmed only when the D3D12 probe proves suitable
  support. Vulkan, OpenGL, optional DirectX features and proprietary upscalers
  are not probed in this version. Their requirements may remain Unknown.
- SSD/HDD classification can be unknown for RAID, USB and virtual storage.
  Displays reflect current desktop modes. No EDID model or HDR capability is
  inferred. Driver versions are joined only when WMI and DXGI names match uniquely.
- Storage uses decimal GB; memory uses GiB. Headroom excludes updates, temporary
  installer files and decompression. Free space is a snapshot.
- Graphics recommendations are explained heuristics. They do not prove a game
  supports a setting or upscaler and never promise a specific FPS. Ultra is a
  manually selectable experiment.

The [architecture and evidence policy](docs/architecture.md) documents the
scanner APIs, aggregation rules, failure handling and extension points. The
[offline catalog](data/hardware-catalog.json) contains each source and metric.

## Privacy and local files

No telemetry, advertisements, accounts or automatic hardware uploads. No
hardware information is sent to external services by the application. Selecting
a benchmark link is a user-initiated web action.

Profiles are stored in the current user's Tauri application data directory,
normally `%APPDATA%\io.github.jcodog.gamespec\profiles`. Theme preference is in
the WebView's local storage. Reports include hardware and installation paths,
so review exported files before sharing them. Corrupt profiles produce a visible
error with their path; hardware detection can still complete.

## Development

Install [Rust stable and the Windows C++ Build Tools](https://v2.tauri.app/start/prerequisites/),
WebView2 and Bun 1.3.14. Use the x64 MSVC Rust toolchain. Do not add npm, Yarn or
pnpm lockfiles.

```powershell
bun install --frozen-lockfile
bun run desktop
```

For a production installer and desktop executable:

```powershell
bun run desktop:build
cargo build --release -p gamespec-cli --locked
```

The desktop output is `target/release/gamespec-checker.exe`, the CLI is
`target/release/gamespec.exe`, and the installer is under
`target/release/bundle/nsis/`. A browser preview cannot access native hardware
and explicitly reports that limitation.

## CLI

```powershell
gamespec scan
gamespec scan --output system.json
gamespec check data/icarus.json --path "D:\Games\Icarus" --gpu "ADAPTER_ID"
gamespec report data/icarus.json --path "D:\Games\Icarus" --gpu "ADAPTER_ID" --output report.json
```

Get adapter IDs from `scan`. With one adapter, the CLI selects it automatically.
With multiple adapters, omitting `--gpu` produces Unknown GPU checks. Output is
JSON. Invalid profiles, GPU IDs and unresolved locations exit with code 1.
Below-minimum and unknown assessments are valid reports, not command failures.
From source, use `cargo run -p gamespec-cli --` before the command.

## Tests and checks

```powershell
cargo fmt --all -- --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
bun run lint
bun run typecheck
bun run test
bun run build
```

Core fixtures cover AMD, Intel, NVIDIA, multi-GPU, missing properties and
insufficient storage. Tests exercise comparisons, alternatives, all verdict
combinations, numeric thresholds, Windows/API checks, SSD requirements,
recommendations and persistence. They need no GPU. Frontend tests validate
imported profiles and native IPC shapes.

For real desktop UI verification, build and launch the app with WebView2
debugging explicitly enabled in a development environment:

```powershell
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=9222"
Start-Process -WindowStyle Hidden target/release/gamespec-checker.exe
bun run test:desktop
Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
```

The test connects to the actual Tauri WebView and uses real scanning and IPC.
It creates, edits and removes a temporary profile, checks a report, changes
graphics settings and exercises both themes. Results and a screenshot stay in
ignored `.local/`. Enable debugging only for local development.

Windows CI checks Rust formatting, tests and Clippy, frontend linting/types/tests,
and a production desktop build. The production workflow builds the current-user
NSIS installer, CLI and SHA-256 checksums. Tag builds create a draft GitHub
release; workflow dispatch provides downloadable Actions artifacts.

## Project structure

```text
crates/gamespec-core/  Models, native scanner, profiles, engine and tests
crates/gamespec-cli/   Shared-core command-line interface
src-tauri/            Tauri commands, permissions and installer configuration
src/                  React UI and validated IPC boundary
data/                 Editable Icarus sample and attributed offline catalog
docs/                 Architecture, evidence policy and validation record
scripts/              Fixture generation and real desktop UI verification
.github/workflows/    Windows CI and production artifacts
```

MIT licensed. Benchmark facts retain their attribution; the MIT licence does not
grant rights to third-party articles or charts.
