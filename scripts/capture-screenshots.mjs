import { chromium } from "@playwright/test";
import assert from "node:assert/strict";
import { mkdirSync } from "node:fs";

// Publish only game requirements screens. Hardware, diagnostics and local paths stay private.
const browser = await chromium.connectOverCDP(
  process.env.GAMESPEC_CDP_URL ?? "http://127.0.0.1:9222",
);
const page = browser
  .contexts()
  .flatMap((c) => c.pages())
  .find(
    (p) =>
      p.url().includes("tauri.localhost") || p.url().startsWith("tauri://"),
  );
assert(
  page,
  "Launch the desktop app with the local WebView2 debugging port first.",
);
mkdirSync("docs/screenshots", { recursive: true });
for (const theme of ["dark", "light"]) {
  await page
    .getByRole("button", { name: "Settings & diagnostics", exact: true })
    .click();
  await page
    .getByRole("combobox", { name: "Theme", exact: true })
    .selectOption(theme);
  await page.getByRole("button", { name: "Game library", exact: true }).click();
  await page
    .locator(".game-row")
    .filter({ has: page.getByRole("heading", { name: "Icarus", exact: true }) })
    .getByRole("button", { name: "Edit", exact: true })
    .click();
  await page
    .getByRole("heading", { name: "Edit Icarus", exact: true })
    .waitFor();
  const dismiss = page.getByRole("button", {
    name: "Dismiss notification",
    exact: true,
  });
  if (await dismiss.count()) await dismiss.click();
  await page.evaluate(() => globalThis.scrollTo(0, 0));
  await page.screenshot({ path: `docs/screenshots/requirements-${theme}.png` });
}
await page
  .getByRole("button", { name: "Back to library", exact: true })
  .click();
await browser.close();
console.log(
  "Captured actual desktop requirements editor in dark and light themes.",
);
