import { useState } from "react";
import { api, errorMessage } from "./api";
import {
  capacitySchema,
  normaliseSourceUrl,
  profileSchema,
  requirementsSchema,
  type Capacity,
  type Profile,
  type Requirements,
} from "./types";
export function CapacityField({
  label,
  value,
  onChange,
}: {
  label: string;
  value: Capacity | null;
  onChange: (v: Capacity | null) => void;
}) {
  return (
    <label>
      {label}
      <span className="quantity">
        <input
          aria-label={`${label} value`}
          type="number"
          min="0"
          max="100000"
          step="any"
          placeholder="Not specified"
          value={value?.value ?? ""}
          onChange={(e) =>
            onChange(
              e.target.value === ""
                ? null
                : { value: e.target.valueAsNumber, unit: value?.unit ?? "gb" },
            )
          }
        />
        <select
          aria-label={`${label} unit`}
          value={value?.unit ?? "gb"}
          onChange={(e) =>
            onChange(
              value
                ? {
                    ...value,
                    unit: capacitySchema.shape.unit.parse(e.target.value),
                  }
                : null,
            )
          }
          disabled={!value}
        >
          <option value="gb">GB</option>
          <option value="gib">GiB</option>
          <option value="ambiguous_gb">GB, ambiguous</option>
        </select>
      </span>
    </label>
  );
}
function TierFields({
  title,
  value,
  onChange,
}: {
  title: string;
  value: Requirements;
  onChange: (v: Requirements) => void;
}) {
  const update = <K extends keyof Requirements>(
    key: K,
    next: Requirements[K],
  ) => onChange({ ...value, [key]: next });
  return (
    <fieldset>
      <legend>{title}</legend>
      <label>
        CPU alternatives
        <textarea
          rows={3}
          value={value.cpu.join("\n")}
          onChange={(e) => update("cpu", e.target.value.split("\n"))}
        />
        <small>
          One full model per line. Any supported alternative is sufficient.
        </small>
      </label>
      <label>
        GPU alternatives
        <textarea
          rows={3}
          value={value.gpu.join("\n")}
          onChange={(e) => update("gpu", e.target.value.split("\n"))}
        />
      </label>
      <CapacityField
        label="RAM"
        value={value.ram}
        onChange={(v) => update("ram", v)}
      />
      <CapacityField
        label="Dedicated VRAM"
        value={value.vram}
        onChange={(v) => update("vram", v)}
      />
      <CapacityField
        label="Installation space"
        value={value.storage}
        onChange={(v) => update("storage", v)}
      />
      <div className="form-row">
        <label>
          Windows versions
          <select
            value={value.windows_versions.join(",")}
            onChange={(e) =>
              update(
                "windows_versions",
                e.target.value
                  ? e.target.value.split(",").map((v) => Number(v) as 10 | 11)
                  : [],
              )
            }
          >
            <option value="">Not specified</option>
            <option value="10,11">10 or 11</option>
            <option value="10">10</option>
            <option value="11">11</option>
          </select>
        </label>
        <label>
          Architecture
          <select
            value={value.architecture ?? ""}
            onChange={(e) =>
              update(
                "architecture",
                requirementsSchema.shape.architecture.parse(
                  e.target.value || null,
                ),
              )
            }
          >
            <option value="">Not specified</option>
            <option value="x86_64">x64</option>
            <option value="x86">x86</option>
            <option value="aarch64">ARM64</option>
          </select>
        </label>
      </div>
      <label>
        Graphics API
        <input
          list="graphics-apis"
          value={value.graphics_api ?? ""}
          onChange={(e) => update("graphics_api", e.target.value || null)}
          placeholder="Not specified"
        />
      </label>
      <label>
        Required feature level
        <select
          value={value.feature_level ?? ""}
          onChange={(e) =>
            update(
              "feature_level",
              requirementsSchema.shape.feature_level.parse(
                e.target.value || null,
              ),
            )
          }
        >
          <option value="">Not specified</option>
          {["11_0", "11_1", "12_0", "12_1", "12_2"].map((v) => (
            <option key={v}>{v}</option>
          ))}
        </select>
      </label>
      <label>
        SSD policy
        <select
          value={value.ssd}
          onChange={(e) =>
            update("ssd", requirementsSchema.shape.ssd.parse(e.target.value))
          }
        >
          <option value="unspecified">Not specified</option>
          <option value="recommended">Recommended, advisory</option>
          <option value="required">Required, mandatory</option>
        </select>
      </label>
    </fieldset>
  );
}
export default function ProfileEditor({
  initial,
  busy,
  onSave,
  onCancel,
}: {
  initial: Profile;
  busy: boolean;
  onSave: (p: Profile) => Promise<void>;
  onCancel: () => void;
}) {
  const [profile, setProfile] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  const update = <K extends keyof Profile>(key: K, value: Profile[K]) =>
    setProfile((p) => ({ ...p, [key]: value }));
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        setError(null);
        const clean = (r: Requirements) => ({
          ...r,
          cpu: r.cpu.map((s) => s.trim()).filter(Boolean),
          gpu: r.gpu.map((s) => s.trim()).filter(Boolean),
        });
        void (async () => {
          try {
            const p = await api.parse(
              JSON.stringify(
                profileSchema.parse({
                  ...profile,
                  minimum: clean(profile.minimum),
                  recommended: clean(profile.recommended),
                }),
              ),
            );
            await onSave(p);
          } catch (e) {
            setError(errorMessage(e));
          }
        })();
      }}
    >
      <div className="page-heading">
        <div>
          <p className="eyebrow">Game requirements</p>
          <h1>
            {initial.name ? `Edit ${initial.name}` : "Create a game profile"}
          </h1>
        </div>
        <button type="button" className="secondary" onClick={onCancel}>
          Back to library
        </button>
      </div>
      <p className="intro">
        Enter this game's published requirements. Blank fields stay uncertain.
        GB uses decimal bytes; GiB uses binary bytes. Choose ambiguous GB if the
        source does not establish which it means.
      </p>
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      {profile.migration_notes.map((n) => (
        <p className="warning" key={n}>
          {n}
        </p>
      ))}
      <section className="panel">
        <div className="form-row">
          <label>
            Game name
            <input
              required
              maxLength={200}
              value={profile.name}
              onChange={(e) => update("name", e.target.value)}
            />
          </label>
          <label>
            Game version
            <input
              maxLength={200}
              value={profile.version}
              onChange={(e) => update("version", e.target.value)}
            />
          </label>
          <label>
            Platform
            <input
              required
              maxLength={200}
              value={profile.platform}
              onChange={(e) => update("platform", e.target.value)}
            />
          </label>
        </div>
        <div className="form-row">
          <label>
            Requirements provenance
            <select
              value={profile.provenance.kind}
              onChange={(e) =>
                update("provenance", {
                  ...profile.provenance,
                  kind: profileSchema.shape.provenance.shape.kind.parse(
                    e.target.value,
                  ),
                })
              }
            >
              <option value="user_entered">User entered</option>
              <option value="unverified">Unverified</option>
              <option value="community">Community source</option>
              <option value="official">Official source, labelled by you</option>
            </select>
          </label>
          <label>
            Requirements source URL
            <input
              type="url"
              placeholder="https://store.steampowered.com/app/…"
              aria-describedby="source-url-help"
              maxLength={2048}
              value={profile.provenance.source_url ?? ""}
              onBlur={(e) =>
                update("provenance", {
                  ...profile.provenance,
                  source_url: normaliseSourceUrl(e.target.value) || null,
                })
              }
              onChange={(e) =>
                update("provenance", {
                  ...profile.provenance,
                  source_url: e.target.value || null,
                })
              }
            />
          </label>
          <label>
            Date verified
            <input
              type="date"
              value={profile.provenance.verified_date ?? ""}
              onChange={(e) =>
                update("provenance", {
                  ...profile.provenance,
                  verified_date: e.target.value || null,
                })
              }
            />
          </label>
        </div>
        <small id="source-url-help">
          Steam store and publisher URLs are accepted as references. Enter the
          minimum and recommended specifications below; this field does not
          download them. Use Community for community-posted requirements, or
          Official when you have checked the publisher's requirements and
          recorded the verification date. Labels are recorded by you, not
          authenticated by GameSpec.
        </small>
      </section>
      <div className="two-column">
        <TierFields
          title="Minimum"
          value={profile.minimum}
          onChange={(v) => update("minimum", v)}
        />
        <TierFields
          title="Recommended"
          value={profile.recommended}
          onChange={(v) => update("recommended", v)}
        />
      </div>
      <section className="panel">
        <h2>Assessment context</h2>
        <div className="form-row">
          <label>
            CPU evidence workload
            <select
              value={profile.cpu_workload}
              onChange={(e) =>
                update(
                  "cpu_workload",
                  profileSchema.shape.cpu_workload.parse(e.target.value),
                )
              }
            >
              <option value="cpu_gaming">Gaming</option>
              <option value="cpu_single_thread">Single thread</option>
              <option value="cpu_multi_thread">Multiple threads</option>
            </select>
          </label>
          <label>
            GPU evidence workload
            <select
              value={profile.gpu_workload}
              onChange={(e) =>
                update(
                  "gpu_workload",
                  profileSchema.shape.gpu_workload.parse(e.target.value),
                )
              }
            >
              <option value="gpu_raster">Game rasterisation</option>
              <option value="gpu_ray_tracing">Ray tracing</option>
              <option value="gpu_synthetic">Synthetic graphics</option>
              <option value="gpu_compute">Compute</option>
            </select>
          </label>
          <CapacityField
            label="Optional installation headroom"
            value={profile.installation_headroom}
            onChange={(v) => update("installation_headroom", v)}
          />
        </div>
        <small>
          Only measurements of the selected workload in the same test context
          are comparable. Changing this selection cannot make synthetic scores
          prove gaming performance.
        </small>
        <label>
          Additional notes
          <textarea
            rows={3}
            maxLength={20000}
            value={profile.notes}
            onChange={(e) => update("notes", e.target.value)}
          />
        </label>
      </section>
      <datalist id="graphics-apis">
        <option value="DirectX 12" />
        <option value="DirectX 11" />
        <option value="Vulkan" />
      </datalist>
      <div className="actions">
        <button disabled={busy} type="submit">
          {busy ? "Saving…" : "Save game profile"}
        </button>
        <button type="button" className="secondary" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </form>
  );
}
