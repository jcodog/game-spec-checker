import { chromium, expect } from "@playwright/test";
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
// Run against a dedicated instance with GAMESPEC_DATA_DIR pointing to an empty test directory.
// Uses real Tauri commands and the real native scanner. No production hardware mocks.
const browser = await chromium.connectOverCDP(
  process.env.GAMESPEC_CDP_URL ?? "http://127.0.0.1:9223",
);
const page = browser
  .contexts()
  .flatMap((c) => c.pages())
  .find((p) => /tauri\.localhost|^tauri:\/\//.test(p.url()));
assert(page, "Dedicated GameSpec WebView was not found.");
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
const nav = (name) => page.getByRole("button", { name, exact: true });
const row = (name) =>
  page
    .locator(".game-row")
    .filter({ has: page.getByRole("heading", { name, exact: true }) });
async function remove(name) {
  const r = row(name);
  await r.getByText("More actions", { exact: true }).click();
  await r.getByRole("button", { name: `Delete ${name}`, exact: true }).click();
  await r.getByRole("button", { name: "Confirm removal", exact: true }).click();
  await r.waitFor({ state: "detached" });
}
try {
  await page
    .getByRole("heading", { name: "Hardware, measured locally", exact: true })
    .waitFor({ timeout: 60000 });
  assert.equal(await page.getByRole("alert").count(), 0);
  await nav("Game library").click();
  await nav("Add game").click();
  const min = page.getByRole("group", { name: "Minimum", exact: true });
  assert.equal(
    await min.getByLabel("CPU alternatives", { exact: true }).inputValue(),
    "",
  );
  assert.equal(
    await min.getByLabel("RAM value", { exact: true }).inputValue(),
    "",
  );
  await page
    .getByLabel("Game name", { exact: true })
    .fill("Desktop verification game");
  await page
    .getByLabel("Game version", { exact: true })
    .fill("Verification only");
  await nav("Save game profile").click();
  await row("Desktop verification game").waitFor();
  await row("Desktop verification game")
    .getByRole("button", { name: "Edit", exact: true })
    .click();
  await page
    .getByLabel("Game version", { exact: true })
    .fill("Edited and persisted");
  await nav("Save game profile").click();
  await page.reload();
  await page
    .getByRole("heading", { name: "Hardware, measured locally", exact: true })
    .waitFor({ timeout: 60000 });
  await nav("Game library").click();
  await row("Desktop verification game")
    .getByText("Edited and persisted", { exact: false })
    .waitFor();
  await page.getByLabel("Search games", { exact: true }).fill("does not exist");
  await page
    .getByRole("heading", { name: "No matching games", exact: true })
    .waitFor();
  await page.getByLabel("Search games", { exact: true }).fill("");
  await row("Desktop verification game")
    .getByRole("button", { name: "Assess", exact: true })
    .click();
  const gpu = page.getByLabel("Gaming GPU", { exact: true });
  const ids = await gpu
    .locator("option")
    .evaluateAll((o) => o.map((v) => v.value).filter(Boolean));
  if (ids.length) await gpu.selectOption(ids[0]);
  const drives = page.getByLabel("Installation drive", { exact: true });
  const mounts = await drives
    .locator("option")
    .evaluateAll((o) => o.map((v) => v.value).filter(Boolean));
  assert(mounts.length, "Real scan must find a volume");
  await drives.selectOption(mounts[0]);
  await nav("Check compatibility").click();
  await page
    .getByRole("region", { name: "Assessment result", exact: true })
    .waitFor({ timeout: 60000 });
  assert.equal(await page.locator(".component-list .component").count(), 8);
  await page.locator(".component details summary").first().click();
  await page
    .locator(".component details[open]")
    .getByRole("heading", { name: "Minimum evidence", exact: true })
    .waitFor();
  await nav("Copy report").click();
  await page.getByText("JSON report copied.", { exact: true }).waitFor();
  await page
    .getByLabel("Resolution profile", { exact: true })
    .selectOption("2560x1440");
  await page
    .getByLabel("Starting preset", { exact: true })
    .selectOption("Ultra");
  await page
    .locator(".recommendations")
    .getByText("Ultra", { exact: true })
    .waitFor();
  await page.getByLabel("Observed FPS", { exact: true }).fill("30");
  await page.getByLabel("Target FPS", { exact: true }).fill("60");
  await page
    .getByRole("heading", { name: "Measured tuning", exact: true })
    .waitFor();
  await nav("Settings & diagnostics").click();
  await expect(
    page.getByText(/0 decision-eligible measurements/),
  ).toBeVisible();
  await page.getByLabel("Theme", { exact: true }).selectOption("light");
  assert.equal(await page.locator("html").getAttribute("data-theme"), "light");
  await page.getByLabel("Theme", { exact: true }).selectOption("dark");
  await page
    .getByLabel("Preview catalogue JSON", { exact: true })
    .setInputFiles({
      name: "invalid.json",
      mimeType: "application/json",
      buffer: Buffer.from("{}"),
    });
  await page.getByRole("alert").waitFor();
  await nav("Dismiss").click();
  const catalog = readFileSync("data/hardware-catalog.json");
  await page
    .getByLabel("Preview catalogue JSON", { exact: true })
    .setInputFiles({
      name: "baseline.json",
      mimeType: "application/json",
      buffer: catalog,
    });
  await page
    .getByRole("heading", { name: "Review before applying", exact: true })
    .waitFor();
  await expect(nav("Apply reviewed catalogue")).toBeDisabled();
  await page.getByRole("checkbox").check();
  await nav("Apply reviewed catalogue").click();
  await page
    .getByText("Catalogue updated locally. Run a new assessment.", {
      exact: true,
    })
    .waitFor();
  await nav("Restore bundled baseline").click();
  await nav("Game library").click();
  await remove("Desktop verification game");
  const imported = {
    ...JSON.parse(readFileSync("data/icarus.json", "utf8")),
    id: "icarus",
    name: "Desktop import verification",
  };
  await page.locator("input[hidden][type=file]").setInputFiles({
    name: "import.json",
    mimeType: "application/json",
    buffer: Buffer.from(JSON.stringify(imported)),
  });
  await page.getByLabel("Game name", { exact: true }).waitFor();
  assert.equal(
    await page.getByLabel("Game name", { exact: true }).inputValue(),
    imported.name,
  );
  await nav("Save game profile").click();
  await row("Icarus").waitFor();
  await remove(imported.name);
  assert.deepEqual(errors, []);
  mkdirSync(".local", { recursive: true });
  for (const width of [880, 1280]) {
    await page.setViewportSize({ width, height: 850 });
    assert(
      await page.evaluate(
        () =>
          globalThis.document.documentElement.scrollWidth <=
          globalThis.innerWidth,
      ),
      `Horizontal overflow at ${width}px`,
    );
    await page.screenshot({
      path: `.local/library-${width}.png`,
      fullPage: true,
    });
  }
  await page.evaluate(() => {
    globalThis.document.body.style.zoom = "1.5";
  });
  assert(
    await page.evaluate(
      () =>
        globalThis.document.documentElement.scrollWidth <=
        globalThis.innerWidth,
    ),
    "Overflow at 150% CSS zoom",
  );
  await page.evaluate(() => {
    globalThis.document.body.style.zoom = "";
  });
  writeFileSync(
    ".local/desktop-verification.json",
    JSON.stringify(
      {
        realTauriIpc: true,
        blankProfile: true,
        persistenceAcrossReload: true,
        search: true,
        jsonImportPreservesExisting: true,
        catalogueValidationAndReview: true,
        copyReport: true,
        componentCount: 8,
        measuredTuning: true,
        themes: true,
        widths: [880, 1280],
        cssZoom: 1.5,
        errors,
      },
      null,
      2,
    ),
  );
  console.log("Real desktop verification passed.");
} finally {
  await browser.close();
}
