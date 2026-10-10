# Continuation and acceptance gaps

This stage replaces the broken data model and comparison boundaries, fixes the
Icarus template bug, preserves units, separates advisory SSD, revises heuristics,
and integrates a redesigned desktop UI and local catalogue workflow.

The following acceptance requirements remain open:

1. Obtain an explicitly reusable, reproducible gaming measurement dataset with
   common CPU/GPU coverage, hardware variants, raw runs/test context, publication
   dates and defensible uncertainty. Permission or independently collected openly
   licensed runs are possible routes. Do not scrape protected benchmark databases,
   invent scores, substitute Blender rendering for games, or treat estimated tiers
   as confirmed measurements. Review data separately before enabling decisions.
2. Populate advertised-memory specification references and probe optional GPU
   capabilities. Preserve API dedicated bytes even when marketed capacities differ.
3. Verify official game-specific graphics guidance and supported technologies.
   Only then recommend enabled upscalers/frame generation or game-specific presets.
4. Extend GUI automation to native save-dialog report/profile/diagnostic exports,
   malformed migration imports, duplicate editing, unavailable destinations and
   catalogue persistence across full restarts.
5. Verify installed release launch, upgrade/profile retention, uninstall and
   missing-WebView2 paths across Windows 10 and 11. CI compilation and creation of
   an NSIS installer are not substitutes for those scenarios.
6. Add junction/mounted-volume and removable-storage integration fixtures on
   Windows. Native resolver behaviour exists; all those environments were not tested.
7. Conduct broader accessibility and real Windows display-scaling validation.
   Viewport/zoom tests cover reflow, not every Windows DPI configuration.

Do not merge or publish this stage as fulfilling all original product acceptance
criteria. The pull request should remain draft until coverage and verification
have been reviewed. See validation.md for evidence actually obtained.
