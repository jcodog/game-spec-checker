import { invoke, isTauri } from "@tauri-apps/api/core";
import { z } from "zod";
import {
  catalogSchema,
  draftSchema,
  librarySchema,
  profileSchema,
  reportSchema,
  settingsSchema,
  snapshotSchema,
  type Profile,
  type Report,
} from "./types";
async function call<T>(
  command: string,
  args: Record<string, unknown>,
  schema: z.ZodType<T>,
): Promise<T> {
  if (!isTauri())
    throw new Error(
      "Native scanning requires the desktop application. Run bun run desktop or launch the Windows application.",
    );
  const value: unknown = await invoke(command, args);
  return schema.parse(value);
}
export const api = {
  scan: () => call("scan_system", {}, snapshotSchema),
  profiles: () => call("list_profiles", {}, librarySchema),
  blank: (id: string) => call("blank_profile", { id }, draftSchema),
  parse: (json: string) => call("parse_profile", { json }, profileSchema),
  save: (profile: Profile) =>
    call(
      "save_profile",
      { json: JSON.stringify(profileSchema.parse(profile)) },
      profileSchema,
    ),
  remove: (id: string) => call("remove_profile", { id }, z.null()),
  check: (profile: Profile, gpuId: string | null, destination: string) =>
    call(
      "check_profile",
      {
        json: JSON.stringify(profileSchema.parse(profile)),
        gpuId,
        destination,
      },
      reportSchema,
    ),
  recommendations: (
    report: Report,
    width: number,
    height: number,
    preset: string | null,
    observed: number | null,
    target: number | null,
  ) =>
    call(
      "graphics_settings",
      { report, width, height, preset, observed, target },
      settingsSchema,
    ),
  catalog: () => call("catalog_info", {}, catalogSchema),
  previewCatalog: (json: string) =>
    call("preview_catalog", { json }, catalogSchema),
  installCatalog: (json: string) =>
    call("install_catalog", { json }, catalogSchema),
  resetCatalog: () => call("reset_catalog", {}, catalogSchema),
  export: (value: unknown, kind: "profile" | "report" | "diagnostics") =>
    call("export_document", { json: JSON.stringify(value), kind }, z.boolean()),
};
export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
