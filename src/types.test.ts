import { describe, expect, it } from "vitest";
import {
  capacityLabel,
  duplicateProfile,
  filterProfiles,
  memory,
  profileSchema,
  reportSchema,
  snapshotSchema,
} from "./types";
import sample from "../data/icarus.json";
import reported from "../crates/gamespec-core/tests/fixtures/reported-system.json";
const profile = profileSchema.parse(sample);
describe("profile editing boundary", () => {
  it("preserves published GB and provenance", () => {
    expect(profile.recommended.vram).toEqual({ value: 8, unit: "gb" });
    expect(profile.provenance.kind).toBe("unverified");
  });
  it("supports incomplete requirements without silently supplying defaults", () => {
    const p = profileSchema.parse({
      ...sample,
      minimum: { ...sample.minimum, cpu: [], ram: null },
    });
    expect(p.minimum.cpu).toEqual([]);
    expect(p.minimum.ram).toBeNull();
  });
  it("rejects malformed imports and units", () => {
    for (const value of [
      { ...sample, schema_version: 1 },
      { ...sample, id: "../bad" },
      {
        ...sample,
        minimum: { ...sample.minimum, ram: { value: -1, unit: "gb" } },
      },
      {
        ...sample,
        minimum: { ...sample.minimum, vram: { value: 8, unit: "shared" } },
      },
      { ...sample, extra: true },
    ])
      expect(profileSchema.safeParse(value).success).toBe(false);
  });
  it("rejects unsafe source URLs", () => {
    expect(
      profileSchema.safeParse({
        ...sample,
        provenance: { ...sample.provenance, source_url: "javascript:alert(1)" },
      }).success,
    ).toBe(false);
  });
  it("accepts Steam sources and normalises pasted URLs without fetching data", () => {
    const url = "https://store.steampowered.com/app/3393110/AION_2/";
    for (const input of [
      url,
      `  ${url}  `,
      url.replace("https://", ""),
      url.replace("https:", "HTTPS:"),
    ]) {
      const p = profileSchema.parse({
        ...sample,
        provenance: {
          ...sample.provenance,
          kind: "community",
          source_url: input,
        },
      });
      expect(p.provenance.source_url).toBe(url);
      expect(p.minimum).toEqual(profile.minimum);
    }
    const tracked = `${url}?l=english&utm_source=reference#requirements`;
    expect(
      profileSchema.parse({
        ...sample,
        provenance: { ...sample.provenance, source_url: tracked },
      }).provenance.source_url,
    ).toBe(tracked);
  });
  it("duplicates explicitly and isolates edits", () => {
    const p = duplicateProfile(
      { ...profile, provenance: { ...profile.provenance, kind: "official" } },
      "copy",
    );
    p.minimum.cpu.push("New CPU");
    expect(profile.minimum.cpu).toHaveLength(2);
    expect(p.id).toBe("copy");
    expect(p.provenance.kind).toBe("user_entered");
    expect(p.notes).toContain("Copied from Icarus");
  });
  it("searches name, version and platform", () => {
    expect(filterProfiles([profile], " ICARUS ")).toHaveLength(1);
    expect(filterProfiles([profile], "windows")).toHaveLength(1);
    expect(filterProfiles([profile], "other")).toHaveLength(0);
  });
});
describe("hardware and reports", () => {
  it("preserves native dedicated bytes without substituting RAM", () => {
    const s = snapshotSchema.parse(reported);
    expect(s.gpus[0]?.dedicated_bytes).toBe(8528596992);
    expect(s.ram_total_bytes).toBe(51539607552);
  });
  it("shows both memory units without rounding to a marketed capacity", () => {
    expect(memory(8528596992)).toBe("8.53 GB / 7.94 GiB");
    expect(memory(null)).toBe("Unavailable");
    expect(capacityLabel({ value: 8, unit: "gib" })).toBe("8 GiB");
    expect(capacityLabel(null)).toBe("Not specified");
  });
  it("rejects invented verdicts and incomplete reports", () => {
    expect(
      reportSchema.safeParse({ verdict: "60fps-guaranteed" }).success,
    ).toBe(false);
  });
});
