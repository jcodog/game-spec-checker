import { z } from 'zod';

const nullableText = z.string().nullable();
const nullableNumber = z.number().nullable();
export const requirementsSchema = z.object({
  cpu: z.array(z.string().min(1).max(250)).min(1).max(30),
  gpu: z.array(z.string().min(1).max(250)).min(1).max(30),
  ram_gib: z.number().finite().min(0).max(100000),
  vram_gib: z.number().finite().min(0).max(100000),
  windows_versions: z.array(z.union([z.literal(10), z.literal(11)])).min(1),
  architecture: z.enum(['x86_64', 'x86', 'aarch64']),
  graphics_api: z.string().min(1).max(80),
  feature_level: z.enum(['11_0', '11_1', '12_0', '12_1', '12_2']).nullable(),
  storage_gb: z.number().finite().min(0).max(100000),
  ssd: z.boolean(),
}).strict();
export const profileSchema = z.object({
  schema_version: z.literal(1),
  id: z.string().regex(/^[a-zA-Z0-9_-]{1,80}$/).refine(s => !/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$/i.test(s), { message: 'Profile ID is a reserved Windows device name.' }),
  name: z.string().trim().min(1).max(200),
  version: z.string().max(200),
  minimum: requirementsSchema,
  recommended: requirementsSchema,
  notes: z.string().max(20000),
}).strict().refine(p => p.recommended.ram_gib >= p.minimum.ram_gib && p.recommended.vram_gib >= p.minimum.vram_gib && p.recommended.storage_gb >= p.minimum.storage_gb && (!p.minimum.ssd || p.recommended.ssd), { message: 'Recommended capacities and SSD requirements cannot be lower than minimum.' });
export type Profile = z.infer<typeof profileSchema>;
export type Requirements = z.infer<typeof requirementsSchema>;
const gpuSchema = z.object({ id: z.string(), model: z.string(), manufacturer: z.string(), dedicated_bytes: nullableNumber, driver_version: nullableText, active: z.boolean(), directx12: z.boolean().nullable(), feature_level: nullableText });
export const volumeSchema = z.object({ mount: z.string(), name: z.string(), total_bytes: z.number(), free_bytes: z.number(), file_system: z.string(), media: z.enum(['ssd', 'hdd', 'unknown']) });
export const snapshotSchema = z.object({
  cpu: z.object({ model: nullableText, manufacturer: nullableText, architecture: nullableText, physical_cores: nullableNumber, logical_threads: nullableNumber, capabilities: z.array(z.string()) }),
  ram_total_bytes: nullableNumber, ram_available_bytes: nullableNumber,
  gpus: z.array(gpuSchema),
  os: z.object({ edition: nullableText, version: nullableText, build: nullableNumber, architecture: nullableText }),
  volumes: z.array(volumeSchema),
  displays: z.array(z.object({ name: z.string(), width: z.number(), height: z.number(), refresh_hz: z.number() })),
  diagnostics: z.array(z.string()),
});
export const verdictSchema = z.enum(['meets_recommended', 'meets_minimum', 'below_minimum', 'unknown']);
const evidenceSchema = z.object({ result: z.enum(['pass', 'fail', 'unknown']), explanation: z.string(), source: z.string() });
export const reportSchema = z.object({
  profile: profileSchema, system: snapshotSchema, gpu_id: nullableText, destination: z.string(), volume: volumeSchema.nullable(), headroom_bytes: nullableNumber, verdict: verdictSchema,
  components: z.array(z.object({ component: z.string(), verdict: verdictSchema, minimum: evidenceSchema, recommended: evidenceSchema })),
  warnings: z.array(z.string()), catalog_version: z.string(),
});
export const settingsSchema = z.array(z.object({ setting: z.string(), value: z.string(), reason: z.string(), confidence: z.string() }));
export type Snapshot = z.infer<typeof snapshotSchema>;
export type Report = z.infer<typeof reportSchema>;
export type Verdict = z.infer<typeof verdictSchema>;
export type Recommendations = z.infer<typeof settingsSchema>;
