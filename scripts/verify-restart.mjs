import { chromium, expect } from "@playwright/test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
const phase = process.argv[2];
assert(
  ["prepare", "verify"].includes(phase),
  "Use prepare before restart, verify afterward.",
);
const browser = await chromium.connectOverCDP(
  process.env.GAMESPEC_CDP_URL ?? "http://127.0.0.1:9223",
);
const page = browser
  .contexts()
  .flatMap((c) => c.pages())
  .find((p) => p.url().includes("tauri.localhost"));
assert(page, "Dedicated desktop instance not found.");
const nav = (name) => page.getByRole("button", { name, exact: true });
try {
  await nav("Your system").click();
  await page
    .getByRole("heading", { name: "Hardware, measured locally", exact: true })
    .waitFor({ timeout: 60000 });
  await nav("Game library").click();
  const row = page
    .locator(".game-row")
    .filter({
      has: page.getByRole("heading", {
        name: "Restart verification",
        exact: true,
      }),
    });
  if (phase === "prepare") {
    await nav("Add game").click();
    await page
      .getByLabel("Game name", { exact: true })
      .fill("Restart verification");
    await page
      .getByLabel("Game version", { exact: true })
      .fill("Persist across a full restart");
    await nav("Save game profile").click();
    await row.waitFor();
    await nav("Settings & diagnostics").click();
    const catalog = {
      ...JSON.parse(readFileSync("data/hardware-catalog.json", "utf8")),
      version: "test-only-restart-baseline",
    };
    await page
      .getByLabel("Preview catalogue JSON", { exact: true })
      .setInputFiles({
        name: "restart-baseline.json",
        mimeType: "application/json",
        buffer: Buffer.from(JSON.stringify(catalog)),
      });
    await page
      .getByRole("heading", { name: "Review before applying", exact: true })
      .waitFor();
    await page.getByRole("checkbox").check();
    await nav("Apply reviewed catalogue").click();
    await page
      .getByText("Catalogue updated locally. Run a new assessment.", {
        exact: true,
      })
      .waitFor();
    console.log(
      "Created a profile and installed an unchanged baseline with a test-only version label. Restart the dedicated process, then run verify.",
    );
  } else {
    await row
      .getByText("Persist across a full restart", { exact: false })
      .waitFor();
    await row.getByText("More actions", { exact: true }).click();
    await row
      .getByRole("button", { name: "Delete Restart verification", exact: true })
      .click();
    await row
      .getByRole("button", { name: "Confirm removal", exact: true })
      .click();
    await row.waitFor({ state: "detached" });
    await nav("Settings & diagnostics").click();
    await expect(
      page.getByText(/Catalogue test-only-restart-baseline/),
    ).toBeVisible();
    await nav("Restore bundled baseline").click();
    await page
      .getByText("Bundled catalogue restored.", { exact: true })
      .waitFor();
    console.log(
      "Full application restart preserved the profile and reviewed catalogue. Temporary data cleaned up.",
    );
  }
} finally {
  await browser.close();
}
