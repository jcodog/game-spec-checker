import { useEffect, useRef, useState } from "react";
import {
  Cpu,
  Gamepad2,
  Monitor,
  Settings,
  RefreshCw,
  Plus,
  Upload,
  Search,
} from "lucide-react";
import { api, errorMessage } from "./api";
import {
  duplicateProfile,
  filterProfiles,
  type Catalog,
  type Profile,
  type Recommendations,
  type Report,
  type Snapshot,
} from "./types";
import ProfileEditor from "./ProfileEditor";
import {
  AssessmentView,
  CatalogDetails,
  Dashboard,
  ProfileSource,
} from "./Views";
type Page = "dashboard" | "library" | "check" | "settings";
type Theme = "system" | "dark" | "light";
const navigation = [
  { id: "dashboard", name: "Your system", icon: Cpu },
  { id: "library", name: "Game library", icon: Gamepad2 },
  { id: "check", name: "Compatibility", icon: Monitor },
  { id: "settings", name: "Settings & diagnostics", icon: Settings },
] as const;
export default function App() {
  const [page, setPage] = useState<Page>("dashboard");
  const [system, setSystem] = useState<Snapshot | null>(null);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [libraryDiagnostics, setLibraryDiagnostics] = useState<string[]>([]);
  const [catalog, setCatalog] = useState<Catalog | null>(null);
  const [profileId, setProfileId] = useState("");
  const [gpuId, setGpuId] = useState<string | null>(null);
  const [destination, setDestination] = useState("");
  const [report, setReport] = useState<Report | null>(null);
  const [settings, setSettings] = useState<Recommendations>([]);
  const [editor, setEditor] = useState<Profile | null>(null);
  const [busy, setBusy] = useState<string | null>("Loading your computer");
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [removeId, setRemoveId] = useState<string | null>(null);
  const [theme, setTheme] = useState<Theme>(() => {
    const value = localStorage.getItem("gamespec-theme");
    return value === "dark" || value === "light" ? value : "system";
  });
  const [dimensions, setDimensions] = useState("1920x1080");
  const [display, setDisplay] = useState("");
  const [preset, setPreset] = useState("auto");
  const [observed, setObserved] = useState("");
  const [target, setTarget] = useState("");
  const [catalogPreview, setCatalogPreview] = useState<{
    json: string;
    catalog: Catalog;
  } | null>(null);
  const [trusted, setTrusted] = useState(false);
  const file = useRef<HTMLInputElement>(null);
  const selected = profiles.find((p) => p.id === profileId);
  const filtered = filterProfiles(profiles, search);
  const modes = Array.from(
    new Set([
      "1920x1080",
      "2560x1440",
      ...(system?.displays.map((d) => `${d.width}x${d.height}`) ?? []),
    ]),
  );
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      document.documentElement.dataset.theme =
        theme === "system" ? (media.matches ? "dark" : "light") : theme;
    };
    apply();
    localStorage.setItem("gamespec-theme", theme);
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);
  useEffect(() => {
    let active = true;
    void Promise.allSettled([api.scan(), api.profiles(), api.catalog()])
      .then(([s, p, c]) => {
        if (!active) return;
        if (s.status === "fulfilled") {
          setSystem(s.value);
          setGpuId(
            s.value.gpus.length === 1 ? (s.value.gpus[0]?.id ?? null) : null,
          );
        }
        if (p.status === "fulfilled") {
          setProfiles(p.value.profiles);
          setLibraryDiagnostics(p.value.diagnostics);
          setProfileId(p.value.profiles[0]?.id ?? "");
        }
        if (c.status === "fulfilled") setCatalog(c.value);
        const failures = [s, p, c].flatMap((v) =>
          v.status === "rejected" ? [errorMessage(v.reason)] : [],
        );
        if (failures.length) setError(Array.from(new Set(failures)).join("\n"));
        setBusy(null);
      })
      .catch((e) => {
        if (active) {
          setError(errorMessage(e));
          setBusy(null);
        }
      });
    return () => {
      active = false;
    };
  }, []);
  useEffect(() => {
    if (!report) return;
    let active = true;
    const [width, height] = dimensions.split("x").map(Number);
    setSettings([]);
    const observedValue = observed ? Number(observed) : null;
    const targetValue = target ? Number(target) : null;
    if (
      [observedValue, targetValue].some(
        (v) => v != null && (!Number.isFinite(v) || v <= 0),
      )
    )
      return;
    void api
      .recommendations(
        report,
        width ?? 1920,
        height ?? 1080,
        preset === "auto" ? null : preset,
        observedValue,
        targetValue,
      )
      .then((r) => {
        if (active) setSettings(r);
      })
      .catch((e) => {
        if (active) setError(errorMessage(e));
      });
    return () => {
      active = false;
    };
  }, [report, dimensions, preset, observed, target]);
  async function task(label: string, action: () => Promise<void>) {
    if (busy) return;
    setBusy(label);
    setError(null);
    setNotice(null);
    try {
      await action();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(null);
    }
  }
  async function loadProfiles() {
    const value = await api.profiles();
    setProfiles(value.profiles);
    setLibraryDiagnostics(value.diagnostics);
  }
  async function refresh() {
    await task("Scanning hardware", async () => {
      const s = await api.scan();
      setSystem(s);
      if (!s.gpus.some((g) => g.id === gpuId))
        setGpuId(s.gpus.length === 1 ? (s.gpus[0]?.id ?? null) : null);
      setReport(null);
      setNotice("Hardware scan refreshed. Run a new assessment.");
    });
  }
  async function add() {
    await task("Creating a blank profile", async () => {
      setEditor(await api.blank(crypto.randomUUID()));
    });
  }
  async function save(p: Profile) {
    setBusy("Saving profile");
    try {
      const saved = await api.save(p);
      await loadProfiles();
      setProfileId(saved.id);
      setEditor(null);
      setReport(null);
      setPage("library");
      setNotice("Game profile saved locally.");
    } finally {
      setBusy(null);
    }
  }
  function choose(p: Profile) {
    setProfileId(p.id);
    setReport(null);
    setPage("check");
  }
  async function exportValue(
    value: unknown,
    kind: "profile" | "report" | "diagnostics",
  ) {
    await task("Exporting JSON", async () => {
      if (await api.export(value, kind)) setNotice("JSON exported.");
    });
  }
  return (
    <div className="app">
      <header className="app-header">
        <a
          className="brand"
          href="#"
          onClick={(e) => {
            e.preventDefault();
            setEditor(null);
            setPage("dashboard");
          }}
        >
          <Gamepad2 size={26} />
          <span>
            GameSpec <strong>Checker</strong>
          </span>
        </a>
        <span className="local-label">Offline · Private by default</span>
        <button
          className="secondary compact"
          disabled={!!busy}
          onClick={() => {
            void refresh();
          }}
        >
          <RefreshCw size={16} />
          Rescan
        </button>
      </header>
      <nav aria-label="Main navigation">
        {navigation.map((n) => (
          <button
            key={n.id}
            aria-current={!editor && page === n.id ? "page" : undefined}
            onClick={() => {
              setEditor(null);
              setPage(n.id);
            }}
          >
            <n.icon size={18} />
            {n.name}
          </button>
        ))}
      </nav>
      <main>
        <div className="feedback" aria-live="polite">
          {busy && <p className="loading">{busy}…</p>}
          {notice && <p className="notice">{notice}</p>}
        </div>
        {error && (
          <div role="alert" className="error">
            {error}
            <button
              className="secondary compact"
              onClick={() => setError(null)}
            >
              Dismiss
            </button>
          </div>
        )}
        {editor ? (
          <ProfileEditor
            key={editor.id}
            initial={editor}
            busy={!!busy}
            onSave={save}
            onCancel={() => {
              setEditor(null);
              setPage("library");
            }}
          />
        ) : (
          <>
            {page === "dashboard" &&
              (system ? (
                <Dashboard
                  system={system}
                  onLibrary={() => setPage("library")}
                />
              ) : (
                !busy && (
                  <section className="empty">
                    <h1>Hardware scan unavailable</h1>
                    <p>Review the error and retry the native scan.</p>
                    <button
                      onClick={() => {
                        void refresh();
                      }}
                    >
                      Retry scan
                    </button>
                  </section>
                )
              ))}
            {page === "library" && (
              <>
                <div className="page-heading">
                  <div>
                    <p className="eyebrow">Saved requirements</p>
                    <h1>Your games</h1>
                    <p className="intro">
                      Profiles belong to a specific game and version. Verify the
                      source before relying on them.
                    </p>
                  </div>
                  <div className="actions">
                    <button
                      className="secondary"
                      disabled={!!busy}
                      onClick={() => file.current?.click()}
                    >
                      <Upload size={16} />
                      Import JSON
                    </button>
                    <button
                      disabled={!!busy}
                      onClick={() => {
                        void add();
                      }}
                    >
                      <Plus size={16} />
                      Add game
                    </button>
                  </div>
                </div>
                <label className="search">
                  <Search size={18} />
                  <input
                    aria-label="Search games"
                    type="search"
                    placeholder="Search name, version or platform"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                  />
                </label>
                <input
                  ref={file}
                  hidden
                  type="file"
                  accept=".json,application/json"
                  onChange={(e) => {
                    const f = e.target.files?.[0];
                    e.target.value = "";
                    if (!f) return;
                    void task("Validating imported profile", async () => {
                      if (f.size > 1000000)
                        throw new Error("Profile exceeds 1 MB.");
                      const p = await api.parse(await f.text());
                      setEditor({ ...p, id: crypto.randomUUID() });
                      setNotice(
                        "Imported as a new profile. Review before saving. Existing profiles are preserved.",
                      );
                    });
                  }}
                />
                {libraryDiagnostics.map((d) => (
                  <p className="warning" key={d}>
                    {d}
                  </p>
                ))}
                <div className="game-list">
                  {filtered.map((p) => (
                    <article className="game-row" key={p.id}>
                      <div>
                        <h2>{p.name}</h2>
                        <p>
                          {p.version || "Version not specified"} · {p.platform}
                        </p>
                        <small>
                          <ProfileSource profile={p} />
                        </small>
                        {p.migration_notes.length > 0 && (
                          <span className="tag warning">
                            Legacy requirements need review
                          </span>
                        )}
                      </div>
                      <div className="actions">
                        <button disabled={!!busy} onClick={() => choose(p)}>
                          Assess
                        </button>
                        <button
                          className="secondary"
                          disabled={!!busy}
                          onClick={() => setEditor(structuredClone(p))}
                        >
                          Edit
                        </button>
                        <details className="profile-actions">
                          <summary>More actions</summary>
                          <div className="actions">
                            <button
                              className="secondary"
                              onClick={() =>
                                setEditor(
                                  duplicateProfile(p, crypto.randomUUID()),
                                )
                              }
                            >
                              Duplicate
                            </button>
                            <button
                              className="secondary"
                              onClick={() => {
                                void exportValue(p, "profile");
                              }}
                            >
                              Export JSON
                            </button>
                            <button
                              className="secondary"
                              aria-label={`Delete ${p.name}`}
                              disabled={!!busy}
                              onClick={() => setRemoveId(p.id)}
                            >
                              Remove
                            </button>
                          </div>
                        </details>
                      </div>
                      {removeId === p.id && (
                        <div className="delete-confirm">
                          <p>Remove {p.name} from this computer?</p>
                          <button
                            className="danger"
                            disabled={!!busy}
                            onClick={() => {
                              void task("Removing profile", async () => {
                                await api.remove(p.id);
                                await loadProfiles();
                                if (profileId === p.id) {
                                  setProfileId("");
                                  setReport(null);
                                }
                                setRemoveId(null);
                              });
                            }}
                          >
                            Confirm removal
                          </button>
                          <button
                            className="secondary"
                            onClick={() => setRemoveId(null)}
                          >
                            Keep profile
                          </button>
                        </div>
                      )}
                    </article>
                  ))}
                </div>
                {!filtered.length && (
                  <section className="empty">
                    <h2>
                      {profiles.length
                        ? "No matching games"
                        : "Start with a genuine game specification"}
                    </h2>
                    <p>
                      {profiles.length
                        ? "Try another search."
                        : "Create a blank profile or import requirements JSON."}
                    </p>
                    <button
                      onClick={() => {
                        void add();
                      }}
                    >
                      Create profile
                    </button>
                  </section>
                )}
              </>
            )}
            {page === "check" && (
              <>
                <div className="page-heading">
                  <div>
                    <p className="eyebrow">Compatibility checker</p>
                    <h1>Can this computer run your game?</h1>
                    <p className="intro">
                      Compare requirements, confirm your destination, then
                      choose an initial configuration.
                    </p>
                  </div>
                </div>
                <form
                  className="panel setup"
                  onSubmit={(e) => {
                    e.preventDefault();
                    if (!selected) return;
                    void task("Assessing your system", async () => {
                      const r = await api.check(selected, gpuId, destination);
                      setSystem(r.system);
                      setReport(r);
                    });
                  }}
                >
                  <div className="form-row">
                    <label>
                      Game profile
                      <select
                        required
                        value={profileId}
                        onChange={(e) => {
                          setProfileId(e.target.value);
                          setReport(null);
                        }}
                      >
                        <option value="">Select a saved game</option>
                        {profiles.map((p) => (
                          <option key={p.id} value={p.id}>
                            {p.name} · {p.version || "No version"}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Gaming GPU
                      <select
                        value={gpuId ?? ""}
                        onChange={(e) => {
                          setGpuId(e.target.value || null);
                          setReport(null);
                        }}
                      >
                        <option value="">
                          Not selected, GPU stays unconfirmed
                        </option>
                        {system?.gpus.map((g) => (
                          <option key={g.id} value={g.id}>
                            {g.model}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                  <div className="form-row">
                    <label>
                      Installation drive
                      <select
                        value={
                          system?.volumes.some((v) => v.mount === destination)
                            ? destination
                            : ""
                        }
                        onChange={(e) => {
                          if (e.target.value) {
                            setDestination(e.target.value);
                            setReport(null);
                          }
                        }}
                      >
                        <option value="">
                          Choose a volume or enter a path
                        </option>
                        {system?.volumes.map((v) => (
                          <option key={v.mount} value={v.mount}>
                            {v.mount} · {(v.free_bytes / 1e9).toFixed(0)} GB
                            free · {v.media.toUpperCase()}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Destination path
                      <input
                        required
                        value={destination}
                        placeholder="C:\Games\YourGame"
                        onChange={(e) => {
                          setDestination(e.target.value);
                          setReport(null);
                        }}
                      />
                      <small>
                        The native scanner resolves the actual backing volume,
                        including mounted folders.
                      </small>
                    </label>
                  </div>
                  <div className="form-row">
                    <label>
                      Display
                      <select
                        value={display}
                        onChange={(e) => {
                          setDisplay(e.target.value);
                          const d = system?.displays.find(
                            (_, i) => String(i) === e.target.value,
                          );
                          if (d) setDimensions(`${d.width}x${d.height}`);
                        }}
                      >
                        <option value="">
                          Choose output dimensions manually
                        </option>
                        {system?.displays.map((d, i) => (
                          <option key={`${d.name}-${i}`} value={i}>
                            {d.name} · {d.width} × {d.height} · {d.refresh_hz}{" "}
                            Hz
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Resolution profile
                      <select
                        value={dimensions}
                        onChange={(e) => setDimensions(e.target.value)}
                      >
                        {modes.map((m) => (
                          <option key={m} value={m}>
                            {m.replace("x", " × ")}
                          </option>
                        ))}
                      </select>
                    </label>
                  </div>
                  {selected && (
                    <p>
                      <ProfileSource profile={selected} /> · Source labels are
                      supplied by the author.
                    </p>
                  )}
                  <button disabled={!!busy || !selected || !destination}>
                    Check compatibility
                  </button>
                </form>
                {report && (
                  <>
                    <AssessmentView
                      report={report}
                      onCopy={() => {
                        void task("Copying report", async () => {
                          await navigator.clipboard.writeText(
                            JSON.stringify(report, null, 2),
                          );
                          setNotice("JSON report copied.");
                        });
                      }}
                      onExport={() => {
                        void exportValue(report, "report");
                      }}
                    />
                    <section className="graphics">
                      <div className="page-heading">
                        <div>
                          <p className="eyebrow">
                            Initial graphics configuration
                          </p>
                          <h2>Start here, then measure in the game</h2>
                          <p>
                            These are generic heuristics. No game-specific
                            preset or FPS guarantee is implied.
                          </p>
                        </div>
                      </div>
                      <div className="panel form-row">
                        <label>
                          Starting preset
                          <select
                            value={preset}
                            onChange={(e) => setPreset(e.target.value)}
                          >
                            <option value="auto">
                              Suggested starting point
                            </option>
                            {["Very Low", "Low", "Medium", "High", "Ultra"].map(
                              (p) => (
                                <option key={p}>{p}</option>
                              ),
                            )}
                          </select>
                        </label>
                        <label>
                          Observed FPS
                          <input
                            type="number"
                            min="1"
                            max="10000"
                            placeholder="Optional"
                            value={observed}
                            onChange={(e) => setObserved(e.target.value)}
                          />
                        </label>
                        <label>
                          Target FPS
                          <input
                            type="number"
                            min="1"
                            max="10000"
                            placeholder="Optional"
                            value={target}
                            onChange={(e) => setTarget(e.target.value)}
                          />
                        </label>
                      </div>
                      <div className="recommendations">
                        {settings.map((s) => (
                          <article key={s.setting}>
                            <div>
                              <h3>{s.setting}</h3>
                              <strong>{s.value}</strong>
                            </div>
                            <p>{s.reason}</p>
                            <small>Confidence: {s.confidence}</small>
                          </article>
                        ))}
                      </div>
                    </section>
                  </>
                )}
              </>
            )}
            {page === "settings" && (
              <>
                <div className="page-heading">
                  <div>
                    <p className="eyebrow">Settings & diagnostics</p>
                    <h1>Your data stays on this computer</h1>
                  </div>
                </div>
                <section className="panel">
                  <h2>Appearance</h2>
                  <label>
                    Theme
                    <select
                      value={theme}
                      onChange={(e) =>
                        setTheme(
                          e.target.value === "light"
                            ? "light"
                            : e.target.value === "dark"
                              ? "dark"
                              : "system",
                        )
                      }
                    >
                      <option value="system">Follow Windows</option>
                      <option value="dark">Dark</option>
                      <option value="light">Light</option>
                    </select>
                  </label>
                </section>
                <section className="panel">
                  <h2>Benchmark catalogue</h2>
                  {catalog && <CatalogDetails catalog={catalog} />}
                  <p>
                    Updates use local JSON files. Schema validation checks
                    structure and consistency; it cannot verify a publisher's
                    license claim, methodology or measurements. Review those
                    before installing data.
                  </p>
                  <label className="file-picker">
                    Preview catalogue JSON
                    <input
                      type="file"
                      accept=".json,application/json"
                      disabled={!!busy}
                      onChange={(e) => {
                        const f = e.target.files?.[0];
                        e.target.value = "";
                        if (!f) return;
                        setCatalogPreview(null);
                        setTrusted(false);
                        void task("Validating catalogue", async () => {
                          if (f.size > 10000000)
                            throw new Error("Catalogue exceeds 10 MB.");
                          const json = await f.text();
                          setCatalogPreview({
                            json,
                            catalog: await api.previewCatalog(json),
                          });
                        });
                      }}
                    />
                  </label>
                  {catalogPreview && (
                    <div className="catalog-preview">
                      <h3>Review before applying</h3>
                      <CatalogDetails catalog={catalogPreview.catalog} />
                      <label className="checkbox">
                        <input
                          type="checkbox"
                          checked={trusted}
                          onChange={(e) => setTrusted(e.target.checked)}
                        />
                        I have reviewed the sources, test contexts and
                        redistribution permission.
                      </label>
                      <div className="actions">
                        <button
                          disabled={!!busy || !trusted}
                          onClick={() => {
                            void task("Installing catalogue", async () => {
                              setCatalog(
                                await api.installCatalog(catalogPreview.json),
                              );
                              setCatalogPreview(null);
                              setReport(null);
                              setNotice(
                                "Catalogue updated locally. Run a new assessment.",
                              );
                            });
                          }}
                        >
                          Apply reviewed catalogue
                        </button>
                        <button
                          className="secondary"
                          onClick={() => setCatalogPreview(null)}
                        >
                          Discard preview
                        </button>
                      </div>
                    </div>
                  )}
                  <button
                    className="secondary"
                    disabled={!!busy}
                    onClick={() => {
                      void task("Restoring bundled catalogue", async () => {
                        setCatalog(await api.resetCatalog());
                        setReport(null);
                        setCatalogPreview(null);
                        setNotice("Bundled catalogue restored.");
                      });
                    }}
                  >
                    Restore bundled baseline
                  </button>
                </section>
                <section className="panel">
                  <h2>Scan diagnostics</h2>
                  {system?.diagnostics.length ? (
                    <ul>
                      {system.diagnostics.map((d, i) => (
                        <li key={i}>{d}</li>
                      ))}
                    </ul>
                  ) : (
                    <p>
                      {system
                        ? "No scanner warnings were reported."
                        : "A hardware scan is not available."}
                    </p>
                  )}
                  {libraryDiagnostics.map((d) => (
                    <p key={d}>{d}</p>
                  ))}
                  <p>
                    Diagnostic export includes hardware models, drivers and
                    volume paths. Review it before sharing. Export opens a local
                    save dialog and sends nothing over the network.
                  </p>
                  <button
                    className="secondary"
                    disabled={!!busy}
                    onClick={() => {
                      void exportValue(
                        {
                          schema_version: 2,
                          system,
                          library_diagnostics: libraryDiagnostics,
                          catalog,
                          privacy:
                            "Local export requested by user; no telemetry",
                        },
                        "diagnostics",
                      );
                    }}
                  >
                    Export diagnostic JSON
                  </button>
                </section>
                <section className="panel">
                  <h2>Privacy</h2>
                  <p>
                    No telemetry, accounts, ads, hardware uploads or automatic
                    network refresh. Profiles are saved locally. Source URLs can
                    be copied for independent review.
                  </p>
                </section>
              </>
            )}
          </>
        )}
      </main>
      <footer>
        GameSpec Checker · Compatibility evidence does not predict frame rates.
      </footer>
    </div>
  );
}
