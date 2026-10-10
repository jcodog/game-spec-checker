# Validation record for the overhaul

Branch: feat/evidence-and-profile-overhaul. This document records completed checks,
not intended checks. The original main workflows passing does not validate this branch.

## Completed locally

- 39 Rust core regression tests passed, without requiring GPU hardware.
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

The redesigned packaged application, installer and full GUI flow are pending
Windows CI artifacts and subsequent actual execution. Their verification results
must be added here only after those checks run. Screenshots of an old application
must not be presented as screenshots of this redesign.
