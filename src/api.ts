import { invoke, isTauri } from '@tauri-apps/api/core';
import { z } from 'zod';
import { profileSchema, reportSchema, settingsSchema, snapshotSchema, type Profile, type Report } from './types';

async function call<T>(command: string, args: Record<string, unknown>, schema: z.ZodType<T>): Promise<T> {
  if (!isTauri()) throw new Error('Open GameSpec Checker as a desktop app. The browser preview cannot inspect native hardware. Run bun run desktop.');
  const result: unknown = await invoke(command, args);
  return schema.parse(result);
}
export const api = {
  scan: () => call('scan_system', {}, snapshotSchema),
  profiles: () => call('list_profiles', {}, z.array(profileSchema)),
  save: (profile: Profile) => call('save_profile', { json: JSON.stringify(profileSchema.parse(profile)) }, profileSchema),
  remove: (id: string) => call('remove_profile', { id }, z.null()),
  check: (profile: Profile, gpuId: string | null, destination: string) => call('check_profile', { json: JSON.stringify(profileSchema.parse(profile)), gpuId, destination }, reportSchema),
  recommendations: (report: Report, width: number, height: number, preset: string | null, observed: number | null, target: number | null) => call('graphics_settings', { report, width, height, preset, observed, target }, settingsSchema),
  export: (value: Profile | Report, kind: 'profile' | 'report') => call('export_document', { json: JSON.stringify(value), kind }, z.boolean()),
};
export function errorMessage(error: unknown): string { return error instanceof Error ? error.message : String(error); }
