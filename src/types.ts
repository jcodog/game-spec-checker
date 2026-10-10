import { z } from "zod";
const text = z.string().nullable();
const number = z.number().finite().nullable();
export function normaliseSourceUrl(value: string): string {
  const trimmed = value.trim();
  return /^store\.steampowered\.com\//i.test(trimmed)
    ? `https://${trimmed}`
    : trimmed;
}
export const capacitySchema = z
  .object({
    value: z.number().finite().min(0).max(100000),
    unit: z.enum(["gb", "gib", "ambiguous_gb"]),
  })
  .strict();
export const requirementsSchema = z
  .object({
    cpu: z.array(z.string().min(1).max(250)).max(30),
    gpu: z.array(z.string().min(1).max(250)).max(30),
    ram: capacitySchema.nullable(),
    vram: capacitySchema.nullable(),
    storage: capacitySchema.nullable(),
    windows_versions: z.array(z.union([z.literal(10), z.literal(11)])).max(2),
    architecture: z.enum(["x86_64", "x86", "aarch64"]).nullable(),
    graphics_api: z.string().min(1).max(80).nullable(),
    feature_level: z.enum(["11_0", "11_1", "12_0", "12_1", "12_2"]).nullable(),
    ssd: z.enum(["unspecified", "recommended", "required"]),
  })
  .strict();
export const provenanceSchema = z
  .object({
    kind: z.enum(["official", "community", "user_entered", "unverified"]),
    source_url: z
      .string()
      .transform(normaliseSourceUrl)
      .pipe(
        z
          .string()
          .url()
          .max(2048)
          .refine(
            (u) => /^https?:\/\//i.test(u),
            "Enter an HTTP or HTTPS source URL, such as a Steam store page.",
          ),
      )
      .transform((u) => new URL(u).href)
      .nullable(),
    verified_date: z
      .string()
      .regex(/^\d{4}-\d{2}-\d{2}$/)
      .nullable(),
  })
  .strict();
export const profileSchema = z
  .object({
    schema_version: z.literal(2),
    id: z
      .string()
      .regex(/^[a-zA-Z0-9_-]{1,80}$/)
      .refine((s) => !/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$/i.test(s)),
    name: z.string().trim().min(1).max(200),
    version: z.string().max(200),
    platform: z.string().trim().min(1).max(200),
    provenance: provenanceSchema,
    cpu_workload: z.enum([
      "cpu_gaming",
      "cpu_single_thread",
      "cpu_multi_thread",
    ]),
    gpu_workload: z.enum([
      "gpu_raster",
      "gpu_ray_tracing",
      "gpu_synthetic",
      "gpu_compute",
    ]),
    minimum: requirementsSchema,
    recommended: requirementsSchema,
    installation_headroom: capacitySchema.nullable(),
    notes: z.string().max(20000),
    migration_notes: z.array(z.string().max(2000)).max(20),
  })
  .strict();
export type Profile = z.infer<typeof profileSchema>;
export type Requirements = z.infer<typeof requirementsSchema>;
export type Capacity = z.infer<typeof capacitySchema>;
export const volumeSchema = z.object({
  mount: z.string(),
  name: z.string(),
  total_bytes: z.number(),
  free_bytes: z.number(),
  file_system: z.string(),
  media: z.enum(["ssd", "hdd", "unknown"]),
  removable: z.boolean().nullable().optional(),
});
export const snapshotSchema = z.object({
  cpu: z.object({
    model: text,
    manufacturer: text,
    architecture: text,
    physical_cores: number,
    logical_threads: number,
    capabilities: z.array(z.string()),
  }),
  ram_total_bytes: number,
  ram_available_bytes: number,
  gpus: z.array(
    z.object({
      id: z.string(),
      model: z.string(),
      manufacturer: z.string(),
      dedicated_bytes: number,
      driver_version: text,
      active: z.boolean(),
      directx12: z.boolean().nullable(),
      feature_level: text,
    }),
  ),
  os: z.object({
    edition: text,
    version: text,
    build: number,
    architecture: text,
  }),
  volumes: z.array(volumeSchema),
  displays: z.array(
    z.object({
      name: z.string(),
      width: z.number(),
      height: z.number(),
      refresh_hz: z.number(),
    }),
  ),
  diagnostics: z.array(z.string()),
});
export const verdictSchema = z.enum([
  "meets_recommended",
  "meets_minimum",
  "below_minimum",
  "unknown",
]);
const resultSchema = z.enum(["pass", "fail", "unknown"]);
const summarySchema = z.object({
  result: resultSchema,
  passed: z.number(),
  failed: z.number(),
  unknown: z.number(),
});
const evidenceSchema = z.object({
  result: resultSchema,
  confidence: z.enum(["high", "medium", "low", "unknown"]),
  explanation: z.string(),
  source: z.string(),
  ratio_interval: z.tuple([z.number(), z.number()]).nullable(),
});
export const reportSchema = z.object({
  schema_version: z.literal(2),
  profile: profileSchema,
  system: snapshotSchema,
  gpu_id: text,
  destination: z.string(),
  volume: volumeSchema.nullable(),
  headroom_bytes: number,
  verdict: verdictSchema,
  minimum: summarySchema,
  recommended: summarySchema,
  components: z.array(
    z.object({
      component: z.string(),
      verdict: verdictSchema,
      minimum: evidenceSchema,
      recommended: evidenceSchema,
      minimum_required: z.boolean(),
      recommended_required: z.boolean(),
    }),
  ),
  warnings: z.array(z.string()),
  catalog_version: z.string(),
});
export const settingsSchema = z.array(
  z.object({
    setting: z.string(),
    value: z.string(),
    reason: z.string(),
    confidence: z.string(),
  }),
);
export const catalogSchema = z.object({
  version: z.string(),
  devices: z.number(),
  measurements: z.number(),
  decision_measurements: z.number(),
  diagnostics: z.array(z.string()),
  datasets: z.array(
    z.object({
      id: z.string(),
      title: z.string(),
      workload: z.string(),
      metric: z.string(),
      test_context: z.string(),
      higher_is_better: z.boolean(),
      methodology: z.string(),
      source_url: z.string(),
      published_date: z.string(),
      license: z.string(),
      redistribution_cleared: z.boolean(),
      decision_eligible: z.boolean(),
      uncertainty_method: z.string(),
    }),
  ),
});
export const librarySchema = z.object({
  profiles: z.array(profileSchema),
  diagnostics: z.array(z.string()),
});
// A blank draft intentionally has no name; it is validated when saved, not created.
export const draftSchema = profileSchema.extend({ name: z.string() });
export type Snapshot = z.infer<typeof snapshotSchema>;
export type Report = z.infer<typeof reportSchema>;
export type Verdict = z.infer<typeof verdictSchema>;
export type Recommendations = z.infer<typeof settingsSchema>;
export type Catalog = z.infer<typeof catalogSchema>;
export type Evidence = z.infer<typeof evidenceSchema>;
export function capacityLabel(value: Capacity | null): string {
  return value
    ? `${value.value} ${value.unit === "gib" ? "GiB" : value.unit === "ambiguous_gb" ? "GB (ambiguous)" : "GB"}`
    : "Not specified";
}
export function memory(bytes: number | null | undefined): string {
  return bytes == null
    ? "Unavailable"
    : `${(bytes / 1e9).toFixed(2)} GB / ${(bytes / 1073741824).toFixed(2)} GiB`;
}
export function duplicateProfile(p: Profile, id: string): Profile {
  return {
    ...structuredClone(p),
    id,
    name: `${p.name} copy`,
    provenance: { ...p.provenance, kind: "user_entered" },
    notes: `Copied from ${p.name}. Verify these requirements before use.\n${p.notes}`,
  };
}
export function filterProfiles(profiles: Profile[], query: string): Profile[] {
  const q = query.trim().toLocaleLowerCase();
  return profiles.filter((p) =>
    `${p.name} ${p.version} ${p.platform}`.toLocaleLowerCase().includes(q),
  );
}
