import { useState } from "react";
import type { Catalog, Evidence, Profile, Report, Snapshot } from "./types";
import { capacityLabel, memory } from "./types";
export function Bytes({ value }: { value: number | null | undefined }) {
  return <>{memory(value)}</>;
}
const resultLabels = {
  pass: "Meets requirement",
  fail: "Below requirement",
  unknown: "Needs evidence",
};
export function Result({
  result,
  advisory = false,
}: {
  result: Evidence["result"];
  advisory?: boolean;
}) {
  return (
    <span className={`result ${result}`}>
      {advisory
        ? result === "pass"
          ? "Advice satisfied"
          : "Advisory"
        : resultLabels[result]}
    </span>
  );
}
export function Source({ source }: { source: string }) {
  const [message, setMessage] = useState<string | null>(null);
  return (
    <div className="source">
      {source.split("\n").map((line, i) =>
        /^https?:\/\/\S+$/.test(line) ? (
          <span key={i}>
            {line}{" "}
            <button
              type="button"
              className="secondary compact"
              aria-label={`Copy source URL ${line}`}
              onClick={() => {
                void navigator.clipboard
                  .writeText(line)
                  .then(() => setMessage("Source URL copied."))
                  .catch(() =>
                    setMessage(
                      "Could not access the clipboard. Select and copy the URL manually.",
                    ),
                  );
              }}
            >
              Copy URL
            </button>
          </span>
        ) : (
          <span key={i}>{line}</span>
        ),
      )}
      {message && <span role="status">{message}</span>}
    </div>
  );
}
export function Dashboard({
  system,
  onLibrary,
}: {
  system: Snapshot;
  onLibrary: () => void;
}) {
  return (
    <>
      <div className="page-heading">
        <div>
          <p className="eyebrow">Your computer</p>
          <h1>Hardware, measured locally</h1>
          <p className="intro">
            Choose a game to assess this hardware against its requirements.
          </p>
        </div>
        <button onClick={onLibrary}>Choose a game</button>
      </div>
      <section className="hardware-summary">
        <article>
          <span>Processor</span>
          <h2>{system.cpu.model ?? "CPU unavailable"}</h2>
          <p>
            {system.cpu.physical_cores ?? "?"} cores ·{" "}
            {system.cpu.logical_threads ?? "?"} threads ·{" "}
            {system.cpu.architecture ?? "Unknown architecture"}
          </p>
          <details>
            <summary>Processor capabilities</summary>
            <p>{system.cpu.capabilities.join(", ") || "Not detected"}</p>
            <p>{system.cpu.manufacturer ?? "Manufacturer unavailable"}</p>
          </details>
        </article>
        <article>
          <span>System memory</span>
          <h2>
            <Bytes value={system.ram_total_bytes} />
          </h2>
          <p>
            <Bytes value={system.ram_available_bytes} /> available now
          </p>
          <small>
            Installed physical RAM. This is separate from dedicated GPU memory.
          </small>
        </article>
      </section>
      <h2>Graphics adapters</h2>
      {system.gpus.length === 0 && (
        <p className="warning">No adapter detected. Review scan diagnostics.</p>
      )}
      <div className="two-column">
        {system.gpus.map((g) => (
          <article className="panel" key={g.id}>
            <div className="section-title">
              <h3>{g.model}</h3>
              <span className="tag">
                {g.active ? "Display attached" : "No attached display detected"}
              </span>
            </div>
            <p className="large">
              <Bytes value={g.dedicated_bytes} />
            </p>
            <p>
              API-reported dedicated memory. Marketed capacity is not inferred
              or rounded up.
            </p>
            <dl>
              <dt>DirectX 12 device</dt>
              <dd>
                {g.directx12 == null
                  ? "Not confirmed"
                  : g.directx12
                    ? "Supported"
                    : "Not supported"}
              </dd>
              <dt>Feature level</dt>
              <dd>{g.feature_level ?? "Unavailable"}</dd>
              <dt>Driver</dt>
              <dd>{g.driver_version ?? "Unavailable"}</dd>
            </dl>
            <small>
              Select the adapter you intend the game to use. Display attachment
              alone does not select it.
            </small>
          </article>
        ))}
      </div>
      <div className="two-column">
        <section className="panel">
          <h2>Windows</h2>
          <p>{system.os.edition ?? "Unavailable"}</p>
          <p>
            Build {system.os.build ?? "?"} · {system.os.architecture ?? "?"}
          </p>
        </section>
        <section className="panel">
          <h2>Connected displays</h2>
          {system.displays.map((d, i) => (
            <p key={`${d.name}-${i}`}>
              {d.name}
              <br />
              <strong>
                {d.width} × {d.height}
              </strong>{" "}
              · {d.refresh_hz} Hz
            </p>
          ))}
          {!system.displays.length && <p>Display information unavailable.</p>}
        </section>
      </div>
      <section className="panel">
        <h2>Installation volumes</h2>
        <div className="volume-list">
          {system.volumes.map((v) => (
            <article key={v.mount}>
              <strong>
                {v.mount} {v.name}
              </strong>
              <span>
                {v.media.toUpperCase()} · {v.file_system}
                {v.removable ? " · Removable" : ""}
              </span>
              <span>
                {(v.free_bytes / 1e9).toFixed(1)} GB free of{" "}
                {(v.total_bytes / 1e9).toFixed(1)} GB
              </span>
              <meter
                aria-label={`Free space on ${v.mount}`}
                min={0}
                max={v.total_bytes}
                value={v.free_bytes}
              />
            </article>
          ))}
        </div>
      </section>
    </>
  );
}
export function ProfileSource({ profile }: { profile: Profile }) {
  return (
    <span>
      {profile.provenance.kind.replaceAll("_", " ")}
      {profile.provenance.kind === "official" ? ", user-labelled" : ""} ·{" "}
      {profile.provenance.verified_date ?? "No verification date"}
    </span>
  );
}
export function AssessmentView({
  report,
  onCopy,
  onExport,
}: {
  report: Report;
  onCopy: () => void;
  onExport: () => void;
}) {
  const failures = report.components.filter(
    (c) => c.minimum_required && c.minimum.result === "fail",
  );
  const uncertain = report.components.filter(
    (c) => c.minimum_required && c.minimum.result === "unknown",
  );
  const labels = {
    meets_recommended: "Meets recommended",
    meets_minimum: "Meets minimum",
    below_minimum: "Below minimum",
    unknown: "Compatibility is not yet confirmed",
  };
  return (
    <section className="assessment" aria-label="Assessment result">
      <div className="page-heading">
        <div>
          <p className="eyebrow">Assessment result</p>
          <h2>{labels[report.verdict]}</h2>
          <p>
            {report.profile.name} ·{" "}
            {report.profile.version || "Version not specified"}
          </p>
        </div>
        <div className="actions">
          <button className="secondary" onClick={onCopy}>
            Copy report
          </button>
          <button className="secondary" onClick={onExport}>
            Export report
          </button>
        </div>
      </div>
      <div className="two-column level-summaries">
        {(["minimum", "recommended"] as const).map((level) => (
          <article className={`panel ${report[level].result}`} key={level}>
            <h3>
              {level === "minimum"
                ? "Minimum requirements"
                : "Recommended requirements"}
            </h3>
            <Result result={report[level].result} />
            <p>
              {report[level].passed} confirmed · {report[level].failed} below ·{" "}
              {report[level].unknown} uncertain
            </p>
          </article>
        ))}
      </div>
      <div className="panel">
        <h3>What needs your attention</h3>
        {failures.length ? (
          <p>
            Confirmed minimum limits:{" "}
            <strong>{failures.map((c) => c.component).join(", ")}</strong>.
            Address these before installing or playing.
          </p>
        ) : (
          <p>No confirmed minimum shortfall was found.</p>
        )}
        {uncertain.length > 0 && (
          <p>
            Still unconfirmed:{" "}
            <strong>{uncertain.map((c) => c.component).join(", ")}</strong>.
            Missing evidence is separate from a confirmed failure.
          </p>
        )}
        <p>
          <ProfileSource profile={report.profile} />. Labels are supplied by the
          profile author.
        </p>
      </div>
      <div className="component-list">
        {report.components.map((c) => (
          <article className="component" key={c.component}>
            <div className="component-heading">
              <h3>{c.component}</h3>
              <div>
                <span>Minimum</span>
                <Result
                  result={c.minimum.result}
                  advisory={!c.minimum_required}
                />
              </div>
              <div>
                <span>Recommended</span>
                <Result
                  result={c.recommended.result}
                  advisory={!c.recommended_required}
                />
              </div>
            </div>
            {(c.minimum.result !== "pass" ||
              c.recommended.result !== "pass") && (
              <p>
                <strong>
                  {c.minimum.result !== "pass" ? "Minimum" : "Recommended"}:
                </strong>{" "}
                {["CPU", "GPU"].includes(c.component) &&
                (c.minimum.result !== "pass" ? c.minimum : c.recommended)
                  .result === "unknown"
                  ? "The available performance evidence cannot confirm this requirement. Expand the evidence to see which comparisons are missing or uncertain."
                  : c.minimum.result !== "pass"
                    ? c.minimum.explanation
                    : c.recommended.explanation}
              </p>
            )}
            <details>
              <summary>Explanation and technical evidence</summary>
              <div className="two-column">
                {(["minimum", "recommended"] as const).map((level) => (
                  <section key={level}>
                    <h4>
                      {level === "minimum"
                        ? "Minimum evidence"
                        : "Recommended evidence"}
                    </h4>
                    <p>{c[level].explanation}</p>
                    <small>
                      Confidence: {c[level].confidence}
                      {!c[`${level}_required`]
                        ? " · Advisory, excluded from mandatory verdict"
                        : ""}
                    </small>
                    <Source source={c[level].source} />
                  </section>
                ))}
              </div>
            </details>
          </article>
        ))}
      </div>
      <section className="panel">
        <h3>Installation location</h3>
        <p className="path">{report.destination}</p>
        {report.volume ? (
          <>
            <p>
              Backing volume {report.volume.mount} ·{" "}
              {report.volume.media.toUpperCase()} ·{" "}
              {report.volume.removable
                ? "Removable"
                : "Fixed or unconfirmed removability"}
            </p>
            <dl>
              <dt>Free / total</dt>
              <dd>
                {(report.volume.free_bytes / 1e9).toFixed(1)} /{" "}
                {(report.volume.total_bytes / 1e9).toFixed(1)} GB
              </dd>
              <dt>Required space</dt>
              <dd>
                {capacityLabel(report.profile.minimum.storage)} minimum ·{" "}
                {capacityLabel(report.profile.recommended.storage)} recommended
              </dd>
              <dt>Remaining after largest entered installation size</dt>
              <dd>
                {report.headroom_bytes == null
                  ? "Unconfirmed"
                  : `${(report.headroom_bytes / 1e9).toFixed(1)} GB`}
              </dd>
              <dt>Optional buffer</dt>
              <dd>{capacityLabel(report.profile.installation_headroom)}</dd>
            </dl>
          </>
        ) : (
          <p>Backing volume unconfirmed.</p>
        )}
      </section>
      <details className="panel">
        <summary>
          Warnings, assumptions and scan diagnostics ({report.warnings.length})
        </summary>
        <ul>
          {report.warnings.map((w, i) => (
            <li key={i}>{w}</li>
          ))}
        </ul>
      </details>
    </section>
  );
}
export function CatalogDetails({ catalog }: { catalog: Catalog }) {
  return (
    <>
      <p>
        Catalogue {catalog.version} · {catalog.devices} hardware identities ·{" "}
        {catalog.decision_measurements} decision-eligible measurements
      </p>
      {catalog.diagnostics.map((d) => (
        <p className="warning" key={d}>
          {d}
        </p>
      ))}
      {catalog.datasets.map((d) => (
        <details key={d.id}>
          <summary>
            {d.title} · {d.workload} ·{" "}
            {d.decision_eligible ? "Decision eligible" : "Reference only"}
          </summary>
          <p>{d.methodology}</p>
          <p>
            Metric: {d.metric} · Higher is better: {String(d.higher_is_better)}
          </p>
          <p>Test context: {d.test_context}</p>
          <p>{d.uncertainty_method}</p>
          <p>
            {d.published_date} · {d.license} · Redistribution{" "}
            {d.redistribution_cleared ? "claimed cleared" : "not cleared"}
          </p>
          <Source source={d.source_url} />
        </details>
      ))}
    </>
  );
}
