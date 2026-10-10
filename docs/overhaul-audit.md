# Engineering audit, 10 October 2026

Baseline: main at 6559084. All tracked Rust, frontend, configuration, fixtures,
scripts and documentation were reviewed. The real CLI report and desktop
screenshot reproduce the reported 48 GiB/RX 7600 system and AION 2 assessment.
Both original Windows workflows passed; no published release existed.

## Priority findings

1. Add game clones Icarus requirements. This is a deterministic creation bug,
   not evidence that persistence mixes game IDs. Duplicating is legitimate only
   as an explicit action. Legacy copied profiles need review, never silent repair.
2. Memory is encoded as ram_gib/vram_gib. Published GB cannot be preserved.
   DXGI reports 8,528,596,992 bytes, or 8.53 GB / 7.94 GiB. The old 8 GiB
   threshold manufactures a failure for a requirement entered as 8 GB.
3. Catalog architecture conflates identities and performance. No CPU scores,
   six GPU synthetic scores, no data update path, no structured workloads,
   dates, licenses, confidence, conflicting evidence or uncertainty intervals.
4. SSD recommended is aggregated as required. Minimum-confirmed information
   is lost when recommended evidence is unknown. Missing profile fields cannot
   be represented, so fabricated defaults are encouraged.
5. Recommendations map the overall verdict to presets. Disk space and Windows
   failures affect graphical quality. Texture thresholds also misclassify 7.94
   GiB; CPU/GPU evidence, game budget and available RAM are largely ignored.
6. One large App owns navigation, persistence, editor and assessment state.
   There is no search, provenance, migration, diagnostic export or catalog UI.
   Small text, repeated setup forms and verbose output bury the result.
7. CI exercises no GUI flow. The real packaged executable launches and scans,
   but earlier desktop verification did not finish. Installer creation is not
   installer verification. Local MSVC C++ PDB DLL failure remains a build issue.
8. One corrupt profile aborts the whole library. Fixed temporary save names can
   collide across processes. Import may overwrite an existing ID without review.

## Implementation boundaries

Retain the supported native scanner, Rust core, CLI and Tauri stack. Replace the
profile schema, comparison/catalog model, aggregation, recommendations and UI.
Separate identification from benchmark comparisons and from physical capacity.

Two designs were considered: adding optional fields to schema 1, or introducing
schema 2 with typed quantities, provenance, optional constraints and explicit
migration. Schema 2 avoids permanently preserving ambiguous ram_gib fields and
permits a blank new profile without unrelated requirements. Legacy reads preserve
their GiB interpretation and attach a review warning; writing occurs only on save.

Assessment takes an explicit validated Catalog, SystemSnapshot, GameProfile and
resolved Volume. Both desktop and CLI use it. Workload-specific measurements
compare only within one dataset/test context. Missing or overlapping evidence
stays Unknown. Soft SSD advice never changes hard compatibility.

Catalog updates are selected local JSON files, validated and reviewed before
installation. There is no automatic scraping, network refresh or hardware upload.

## Benchmark research

- Blender Open Data explicitly provides CC0 downloadable data:
  https://opendata.blender.org/download/. Its Cycles rendering scores are not
  raster game performance or single-thread CPU evidence. Do not substitute them.
- BottleneckPC provides CC BY 4.0 data at https://bottleneckpc.com/data and
  describes its methodology at https://bottleneckpc.com/how-it-works. The publisher
  calls its scores comparative estimates, not lab measurements. Raw runs, per-model
  uncertainty and exact calibration are absent. It cannot substantiate a confirmed
  game compatibility result under the requested measurement standard.
- PassMark software licensing does not establish redistribution rights for its
  benchmark database. OpenBenchmarking's software GPL likewise does not establish
  result-data licensing. No database extraction was authorized or implemented.
- Existing Tweakers scores were removed. A redistribution permission was
  not established. They were synthetic evidence, not a cleared,
  reproducible gaming baseline. They must not silently become raster measurements.

Expanding common-hardware gaming coverage still requires an explicitly reusable
measurement dataset or permission. Catalog infrastructure and isolated tests can
be completed without inventing those measurements. This is a delivery limitation,
not a reason to turn Unknown into Pass or claim the full product is finished.

