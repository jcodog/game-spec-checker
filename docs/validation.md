# Validation record for the overhaul

Branch: feat/evidence-and-profile-overhaul. This document records completed checks,
not intended checks. The original main workflows passing does not validate this branch.

## Completed locally

- 40 Rust core regression tests passed, without requiring GPU hardware.
- Core and CLI Clippy passed with warnings denied after the final test additions.
- Frontend lint and TypeScript checks passed.
- Nine frontend boundary/state-helper tests passed.
- Frontend production build passed. Rollup reports harmless annotation warnings
  inside the installed Zod package; errors were not suppressed.
- A shared-core CLI build succeeded.

Core cases include aliases/variants, cross-vendor interval algorithm, separate
workloads/scales, conflicting/overlapping evidence, alternatives, licensing guard,
GB/GiB/ambiguous thresholds, provided-system regression, integrated/multi-GPU,
API/Windows checks, storage and SSD policies, headroom, blank creation, migration,
provenance, malformed JSON and corrupt-file isolation/persistence.

Synthetic comparison intervals exist only inside test functions and are explicitly
labelled. They prove comparison behaviour, not real hardware equivalence. The
reported-system fixture preserves the user's CPU/GPU/RAM/VRAM/build values while
using artificial adapter IDs, driver text, volumes, display and available RAM.
It is not a complete raw hardware export or benchmark dataset.

## Environment limitation

Local full Tauri compilation fails in the installed Visual C++ dependency build:
vswhom-sys C1356 cannot find mspdbcore.dll. Core/CLI build with the available Rust
LLVM linker and Windows libraries. No antivirus protections were disabled.

## Windows CI and packaging

For commit 96c94b0, both workflows passed:

- Windows CI: https://github.com/jcodog/game-spec-checker/actions/runs/38054177333
- Windows production build: https://github.com/jcodog/game-spec-checker/actions/runs/38054179134
- Pull-request CI: https://github.com/jcodog/game-spec-checker/actions/runs/38054457325

They ran full-workspace Rust tests/Clippy, frontend checks, and production desktop
builds. The production workflow created the current-user NSIS installer, CLI and
SHA-256 checksums. Downloaded installer/CLI hashes matched the checksum manifest.

The NSIS installer completed successfully into an isolated workspace directory.
A preflight confirmed no existing GameSpec installation registration would be
replaced. The installed executable launched outside the development environment
using existing WebView2. Bundled profiles/catalogue and frontend resources worked.
No administrator elevation was used by the installer or normal application.
The Codex command runner used its normal approved host execution.

## Actual desktop execution

The installed executable passed scripts/verify-desktop.mjs using real Tauri IPC
and the real scanner. No hardware values were mocked or replaced. Verified:

- A new profile has empty CPU/RAM requirements; no Icarus inheritance.
- Creation, editing, persistence through reload, search and confirmed removal.
- JSON import receives a new ID and preserves the original Icarus profile.
- A native assessment renders eight independent component rows and expandable evidence.
- Copy report, resolution/manual preset changes and measured-FPS tuning.
- Invalid catalogue rejection, preview, required review acknowledgement, installation/reset.
- Dark/light themes and no browser page errors.
- No horizontal overflow at 880 and 1280 CSS pixels or 150% CSS zoom.

scripts/verify-restart.mjs additionally confirmed that a profile and reviewed
catalogue survive closing and relaunching the complete installed application.
The test changed only catalogue version metadata, not scores or hardware values.
Temporary test profiles/catalogue were removed afterward.

Actual screenshots of the installed editor in both themes were reviewed and
committed under docs/screenshots. Hardware/dashboard and native diagnostic
screenshots remain private in ignored .local. These screenshots show commit
96c94b0, not the old main application.

The first interaction attempts failed because getByLabel exact matched full HTML
label text including options/help text, and because an interrupted editor draft
was still open. The harness now starts from the dashboard and uses the observed
accessible combobox names. Blank-profile and real-volume assertions were preserved.

A subsequent identity-data pass adds explicit conservative short aliases and its
regression test. It changes no UI/assessment code and does not create cross-model
performance evidence. Latest-branch CI status is recorded in the pull request.

## Remaining verification gaps

Native Save-dialog exports were not automated. Installer upgrades/uninstall,
missing-WebView2 installation, clean Windows 10/11 VMs and real Windows DPI changes
are not verified. CSS zoom tests demonstrate reflow, not OS DPI behaviour.
See continuation.md. Passing these checks does not supply missing gaming benchmarks
or make the full product acceptance criteria complete.
