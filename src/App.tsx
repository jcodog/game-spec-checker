import { useEffect, useRef, useState } from 'react';
import { Activity, ArrowRight, Check, ChevronRight, Clipboard, Cpu, Download, FolderOpen, Gamepad2, HardDrive, LayoutDashboard, LoaderCircle, MemoryStick, Monitor, Moon, Plus, RefreshCw, Settings, ShieldCheck, Sun, Trash2, Upload, X } from 'lucide-react';
import { api, errorMessage } from './api';
import { profileSchema, type Profile, type Recommendations, type Report, type Snapshot, type Verdict } from './types';
import ProfileEditor from './ProfileEditor';
import sample from '../data/icarus.json';

type Page = 'dashboard' | 'library' | 'report' | 'settings';
type Theme = 'dark' | 'light' | 'system';
const verdictLabels: Record<Verdict, string> = { meets_recommended: 'Meets recommended', meets_minimum: 'Meets minimum', below_minimum: 'Below minimum', unknown: 'Unknown / insufficient evidence' };
function memory(bytes: number | null | undefined): string { return bytes == null ? 'Unknown' : `${(bytes / 1073741824).toFixed(1)} GiB`; }
function storage(bytes: number): string { return `${(bytes / 1e9).toFixed(1)} GB`; }
function Status({ verdict }: { verdict: Verdict }) { return <span className={`status ${verdict}`}><span />{verdictLabels[verdict]}</span>; }
function Info({ label, children }: { label: string; children: React.ReactNode }) { return <div className="info"><span>{label}</span><strong>{children}</strong></div>; }
function EvidenceSource({ source }: { source: string }) { return <>{source.split('\n').map((s, i) => /^https:\/\//.test(s) ? <a key={i} href={s} target="_blank" rel="noreferrer">Benchmark source</a> : <small key={i}>{s}</small>)}</>; }

export default function App() {
  const [page, setPage] = useState<Page>('dashboard');
  const [system, setSystem] = useState<Snapshot | null>(null);
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [profileId, setProfileId] = useState('icarus');
  const [gpuId, setGpuId] = useState<string | null>(null);
  const [destination, setDestination] = useState('');
  const [report, setReport] = useState<Report | null>(null);
  const [recommendations, setRecommendations] = useState<Recommendations>([]);
  const [editor, setEditor] = useState<Profile | null>(null);
  const [busy, setBusy] = useState<string | null>('Detecting hardware');
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [theme, setTheme] = useState<Theme>(() => { const saved = localStorage.getItem('gamespec-theme'); return saved === 'dark' || saved === 'light' ? saved : 'system'; });
  const [dimensions, setDimensions] = useState('1920x1080');
  const [displayName, setDisplayName] = useState('');
  const [preset, setPreset] = useState('auto');
  const [observed, setObserved] = useState('');
  const [target, setTarget] = useState('60');
  const fileInput = useRef<HTMLInputElement>(null);
  const selectedProfile = profiles.find(p => p.id === profileId) ?? profiles[0];
  const gpu = system?.gpus.find(g => g.id === gpuId);
  const volume = system?.volumes.find(v => v.mount === destination);
  const modes = Array.from(new Set(['1920x1080', '2560x1440', ...(system?.displays.map(d => `${d.width}x${d.height}`) ?? [])]));

  useEffect(() => {
    const media = window.matchMedia('(prefers-color-scheme: dark)');
    const apply = () => { document.documentElement.dataset.theme = theme === 'system' ? (media.matches ? 'dark' : 'light') : theme; };
    apply(); localStorage.setItem('gamespec-theme', theme); media.addEventListener('change', apply);
    return () => media.removeEventListener('change', apply);
  }, [theme]);
  useEffect(() => {
    let active = true;
    void Promise.allSettled([api.scan(), api.profiles()]).then(([scanResult, profilesResult]) => {
      if (!active) return;
      if (scanResult.status === 'fulfilled') {
        const scan = scanResult.value;
        setSystem(scan);
        setGpuId(scan.gpus.length === 1 ? scan.gpus[0]?.id ?? null : null);
        setDestination(scan.volumes[0]?.mount ?? '');
      }
      if (profilesResult.status === 'fulfilled') setProfiles(profilesResult.value);
      const failures = [scanResult, profilesResult].flatMap(result => result.status === 'rejected' ? [errorMessage(result.reason)] : []);
      if (failures.length) setError(Array.from(new Set(failures)).join('\n'));
    }).catch(e => { if (active) setError(errorMessage(e)); }).finally(() => { if (active) setBusy(null); });
    return () => { active = false; };
  }, []);
  useEffect(() => {
    if (!report) return;
    let active = true;
    const [width, height] = dimensions.split('x').map(Number);
    setRecommendations([]);
    void api.recommendations(report, width ?? 1920, height ?? 1080, preset === 'auto' ? null : preset, observed ? Number(observed) : null, target ? Number(target) : null)
      .then(result => { if (active) setRecommendations(result); }).catch(e => { if (active) setError(errorMessage(e)); });
    return () => { active = false; };
  }, [report, dimensions, preset, observed, target]);
  async function task(label: string, operation: () => Promise<void>) {
    setBusy(label); setError(null); setNotice(null);
    try { await operation(); } catch (e) { setError(errorMessage(e)); } finally { setBusy(null); }
  }
  async function refresh() {
    await task('Detecting hardware', async () => {
      const scan = await api.scan(); setSystem(scan);
      if (!scan.gpus.some(g => g.id === gpuId)) setGpuId(scan.gpus.length === 1 ? scan.gpus[0]?.id ?? null : null);
      setReport(null); setNotice('Hardware refreshed. Run a new assessment to use these measurements.');
    });
  }
  async function check() {
    if (!selectedProfile) return;
    await task('Assessing your system', async () => {
      const result = await api.check(selectedProfile, gpuId, destination);
      setReport(result); setSystem(result.system); setPage('report');
    });
  }
  async function save(profile: Profile) {
    setBusy('Saving profile');
    try { const saved = await api.save(profile); setProfiles(await api.profiles()); setProfileId(saved.id); setEditor(null); setPage('library'); setReport(null); setNotice('Game profile saved on this computer.'); }
    finally { setBusy(null); }
  }
  function addGame() { const profile = profileSchema.parse(sample); setEditor({ ...profile, id: crypto.randomUUID(), name: '', version: '', notes: '' }); }
  function navigate(next: Page) { setEditor(null); setPage(next); }
  const selection = <section className="panel setup-panel">
    <div className="section-heading compact"><h2>Set up your assessment</h2><span className="mono">01 / INPUTS</span></div>
    <div className="setup-grid">
      <label>Game profile<select value={selectedProfile?.id ?? ''} onChange={e => { setProfileId(e.target.value); setReport(null); }} disabled={!profiles.length || !!busy}>{profiles.map(p => <option value={p.id} key={p.id}>{p.name}</option>)}</select></label>
      <label>Gaming GPU<select value={gpuId ?? ''} onChange={e => { setGpuId(e.target.value || null); setReport(null); }} disabled={!system || !!busy}><option value="">Select an adapter</option>{system?.gpus.map(g => <option value={g.id} key={g.id}>{g.model}{g.active ? ' · display attached' : ''}</option>)}</select></label>
      <label>Installation volume<select value={volume?.mount ?? ''} onChange={e => { setDestination(e.target.value); setReport(null); }} disabled={!system || !!busy}><option value="">Custom destination path</option>{system?.volumes.map(v => <option key={v.mount} value={v.mount}>{v.mount} · {storage(v.free_bytes)} free · {v.media.toUpperCase()}</option>)}</select></label>
    </div>
    <div className="destination-row"><label>Destination path<input value={destination} onChange={e => { setDestination(e.target.value); setReport(null); }} placeholder="C:\Games\Icarus" disabled={!!busy} /><small>Existing parent directories, junctions and volume mount points are resolved by Windows.</small></label><button onClick={() => void check()} disabled={!selectedProfile || !system || !destination || !!busy}>Check compatibility<ArrowRight size={17} /></button></div>
    {volume && <div className="drive-summary"><span>{storage(volume.total_bytes)} total</span><span>{storage(volume.free_bytes)} available</span><span>{selectedProfile?.recommended.storage_gb ?? 0} GB required</span><span>{(volume.free_bytes / 1e9 - (selectedProfile?.recommended.storage_gb ?? 0)).toFixed(1)} GB headroom</span><span>{volume.file_system} · {volume.media.toUpperCase()}</span></div>}
  </section>;

  return <div className="app-shell">
    <aside className="sidebar">
      <div className="brand"><div className="brand-mark"><Gamepad2 size={23} /></div><div>GameSpec<span>CHECKER</span></div></div>
      <p className="sidebar-label">WORKSPACE</p>
      <nav aria-label="Main navigation">{([{ id: 'dashboard', label: 'Your system', icon: LayoutDashboard }, { id: 'library', label: 'Game library', icon: Gamepad2 }, { id: 'report', label: 'Compatibility', icon: Activity }, { id: 'settings', label: 'Settings & diagnostics', icon: Settings }] satisfies { id: Page; label: string; icon: typeof Settings }[]).map(item => <button key={item.id} className={page === item.id && !editor ? 'active' : ''} onClick={() => navigate(item.id)}><item.icon size={18} />{item.label}{item.id === 'library' && <span className="nav-count">{profiles.length}</span>}</button>)}</nav>
      <div className="sidebar-bottom"><ShieldCheck size={19} /><strong>Your hardware. Your data.</strong><p>Offline by design.<br />No accounts. No telemetry.</p><span className="mono">WINDOWS / v0.1.0</span></div>
    </aside>
    <div className="main-shell">
      <header className="topbar"><div><span>Workspace</span><ChevronRight size={14} /><strong>{editor ? 'Requirements editor' : page === 'dashboard' ? 'Your system' : page === 'library' ? 'Game library' : page === 'report' ? 'Compatibility' : 'Settings & diagnostics'}</strong></div><div><span className="local-label"><span />Local analysis</span><button className="icon-button" aria-label={theme === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'} onClick={() => setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark')}>{document.documentElement.dataset.theme === 'dark' ? <Sun size={18} /> : <Moon size={18} />}</button></div></header>
      <main>
        {busy && <div className="notice loading" role="status"><LoaderCircle className="spin" size={17} />{busy}…</div>}
        {error && <div className="notice error" role="alert"><span>{error}</span><button className="icon-button" aria-label="Dismiss error" onClick={() => setError(null)}><X size={17} /></button></div>}
        {notice && <div className="notice" role="status"><Check size={17} />{notice}<button className="icon-button" aria-label="Dismiss notification" onClick={() => setNotice(null)}><X size={17} /></button></div>}
        {editor ? <ProfileEditor key={editor.id} initial={editor} busy={!!busy} onSave={save} onCancel={() => { setEditor(null); setPage('library'); }} /> : <>
          {page === 'dashboard' && <>
            <div className="section-heading"><div><p className="eyebrow">Hardware overview</p><h1>Know what your PC can handle.</h1><p className="muted">Real hardware. Clear requirements. Evidence you can inspect.</p></div><button className="secondary" disabled={!!busy} onClick={() => void refresh()}><RefreshCw size={16} />Rescan system</button></div>
            <div className="overview-strip"><div><span className="mono">SYSTEM STATUS</span><strong>{system ? 'Hardware detected' : busy ? 'Scanning your computer' : 'Detection unavailable'}</strong></div><span>{system?.os.edition ?? 'Windows desktop required'}<br /><small>Build {system?.os.build ?? 'unknown'} · {system?.os.architecture ?? 'unknown architecture'}</small></span><span className="outline-tag"><ShieldCheck size={15} />Private & local</span></div>
            <div className="hardware-grid">
              <article className="hardware-card"><div className="card-title"><Cpu size={20} /><span>PROCESSOR</span></div><h2>{system?.cpu.model ?? 'Unknown processor'}</h2><div className="card-stats"><Info label="Physical cores">{system?.cpu.physical_cores ?? '?'}</Info><Info label="Logical threads">{system?.cpu.logical_threads ?? '?'}</Info><Info label="Architecture">{system?.cpu.architecture ?? '?'}</Info></div><p className="card-foot">{system?.cpu.manufacturer ?? 'Manufacturer unknown'} · {system?.cpu.capabilities.join(' / ') || 'Capabilities unavailable'}</p></article>
              <article className="hardware-card"><div className="card-title"><Monitor size={20} /><span>GRAPHICS</span></div><h2>{gpu?.model ?? (system?.gpus.length ? 'Choose your gaming GPU' : 'GPU detection unavailable')}</h2><div className="card-stats"><Info label="Dedicated VRAM">{memory(gpu?.dedicated_bytes)}</Info><Info label="D3D12 support">{gpu?.directx12 == null ? 'Unknown' : gpu.directx12 ? 'Supported' : 'Unsupported'}</Info><Info label="Feature level">{gpu?.feature_level ?? '?'}</Info></div><p className="card-foot">{gpu?.driver_version ? `Driver ${gpu.driver_version}` : `${system?.gpus.length ?? 0} adapters detected. Select one below.`}</p></article>
              <article className="hardware-card"><div className="card-title"><MemoryStick size={20} /><span>MEMORY</span></div><h2>{memory(system?.ram_total_bytes)} <small>installed</small></h2><div className="memory-track"><span style={{ width: system?.ram_total_bytes ? `${Math.min(100, ((system.ram_total_bytes - (system.ram_available_bytes ?? 0)) / system.ram_total_bytes) * 100)}%` : '0%' }} /></div><div className="card-stats"><Info label="Available now">{memory(system?.ram_available_bytes)}</Info><Info label="Source">Physical memory</Info></div><p className="card-foot">Available memory changes with running applications.</p></article>
              <article className="hardware-card"><div className="card-title"><HardDrive size={20} /><span>STORAGE & DISPLAYS</span></div><h2>{system?.volumes.length ?? 0} volumes <small>/ {system?.displays.length ?? 0} displays</small></h2><div className="compact-list">{system?.volumes.slice(0, 3).map(v => <div key={v.mount}><strong>{v.mount}</strong><span>{storage(v.free_bytes)} free</span><small>{v.media.toUpperCase()}</small></div>)}</div><p className="card-foot">{system?.displays.map(d => `${d.width}×${d.height} @ ${d.refresh_hz} Hz`).join(' · ') || 'Display modes unavailable'}</p></article>
            </div>
            {selection}
            <div className="bottom-note"><ShieldCheck size={17} /><p>Performance comparisons use a small, attributed offline catalog. Missing evidence produces Unknown. Recommended requirements never guarantee a frame rate.</p></div>
          </>}
          {page === 'library' && <>
            <div className="section-heading"><div><p className="eyebrow">Saved profiles</p><h1>Your game library</h1><p className="muted">Write requirements once. Reuse them whenever your hardware changes.</p></div><div className="actions"><button className="secondary" onClick={() => fileInput.current?.click()} disabled={!!busy}><Upload size={16} />Import JSON</button><button onClick={addGame} disabled={!!busy}><Plus size={16} />Add game</button></div></div>
            <input ref={fileInput} type="file" accept=".json,application/json" hidden onChange={e => { const file = e.target.files?.[0]; e.target.value = ''; if (!file) return; void task('Importing profile', async () => { if (file.size > 1000000) throw new Error('Profile exceeds 1 MB.'); const text = await file.text(); const raw: unknown = JSON.parse(text); const profile = profileSchema.parse(raw); setEditor(profile); setNotice('Imported profile opened for review. Save it to add it to your library.'); }); }} />
            {profiles.length === 0 && <div className="empty-state"><FolderOpen size={36} /><h2>No profiles loaded</h2><p>Retry hardware detection or add a profile in the desktop app.</p><button onClick={addGame}><Plus size={16} />Add game</button></div>}
            <div className="game-list">{profiles.map(p => <article className="game-row" key={p.id}><div className="game-symbol"><Gamepad2 size={28} /></div><div className="game-info"><h2>{p.name}</h2><p>{p.version || 'Version not specified'}</p><div className="game-specs"><span>{p.minimum.ram_gib} GiB RAM</span><span>{p.minimum.vram_gib} GiB VRAM</span><span>{p.minimum.storage_gb} GB storage</span>{p.recommended.ssd && <span>SSD recommended</span>}</div></div><div className="game-actions"><button className="secondary" onClick={() => { setProfileId(p.id); setReport(null); setPage('report'); }}>Assess<ArrowRight size={15} /></button><button className="quiet" onClick={() => setEditor(p)}>Edit</button><button className="quiet" onClick={() => setEditor({ ...p, id: crypto.randomUUID(), name: `${p.name} copy` })}>Duplicate</button><button className="icon-button" aria-label={`Export ${p.name}`} disabled={!!busy} onClick={() => void task('Exporting profile', async () => { if (await api.export(p, 'profile')) setNotice('Profile exported.'); })}><Download size={17} /></button>{p.id !== 'icarus' && <button className="icon-button danger" aria-label={`Delete ${p.name}`} disabled={!!busy} onClick={() => void task('Removing profile', async () => { await api.remove(p.id); setProfiles(await api.profiles()); setReport(null); setNotice('Profile removed.'); })}><Trash2 size={17} /></button>}</div></article>)}</div>
            <div className="panel library-note"><h2>The Icarus profile is editable</h2><p>It contains the minimum and recommended requirements supplied for this project. Confirm them against the game's publisher before relying on them for a particular version.</p></div>
          </>}
          {page === 'report' && <>
            <div className="section-heading"><div><p className="eyebrow">Compatibility analysis</p><h1>{report ? `${report.profile.name} / your PC` : 'Will it run?'}</h1><p className="muted">Minimum and recommended requirements, assessed independently.</p></div>{report && <div className="actions"><button className="secondary" onClick={() => void task('Copying report', async () => { await navigator.clipboard.writeText(JSON.stringify(report, null, 2)); setNotice('JSON report copied.'); })}><Clipboard size={16} />Copy report</button><button onClick={() => void task('Exporting report', async () => { if (await api.export(report, 'report')) setNotice('Report exported.'); })}><Download size={16} />Export JSON</button></div>}</div>
            {selection}
            {report ? <>
              <div className={`report-summary ${report.verdict}`}><div><p className="eyebrow">Assessment result</p><h2>{verdictLabels[report.verdict]}</h2><p>{report.components.filter(c => c.minimum.result === 'pass').length} of {report.components.length} minimum checks confirmed. Unknown checks require more evidence.</p></div><Activity size={42} /></div>
              <div className="report-table" role="region" aria-label="Component compatibility results"><div className="report-table-head"><span>COMPONENT</span><span>MINIMUM</span><span>RECOMMENDED</span><span>ASSESSMENT</span></div>{report.components.map(c => <details key={c.component}><summary><strong>{c.component}</strong><span className={`evidence-result ${c.minimum.result}`}>{c.minimum.result}</span><span className={`evidence-result ${c.recommended.result}`}>{c.recommended.result}</span><Status verdict={c.verdict} /><ChevronRight size={16} /></summary><div className="evidence-grid">{[{ name: 'Minimum evidence', value: c.minimum }, { name: 'Recommended evidence', value: c.recommended }].map(e => <div key={e.name}><h3>{e.name}</h3><p>{e.value.explanation}</p><EvidenceSource source={e.value.source} /></div>)}</div></details>)}</div>
              <div className="panel"><h2>Installation location</h2><p className="mono">{report.destination}</p><div className="card-stats"><Info label="Backing volume">{report.volume?.mount ?? 'Unknown'}</Info><Info label="Available space">{report.volume ? storage(report.volume.free_bytes) : 'Unknown'}</Info><Info label="Required">{report.profile.recommended.storage_gb} GB</Info><Info label="Estimated headroom">{report.headroom_bytes == null ? 'Unknown' : storage(report.headroom_bytes)}</Info><Info label="Media">{report.volume?.media.toUpperCase() ?? 'Unknown'}</Info></div><small>Headroom excludes updates, decompression and temporary installer files.</small></div>
              <div className="warnings"><h2>What to keep in mind</h2><ul>{report.warnings.map(w => <li key={w}>{w}</li>)}</ul></div>
              <div className="section-heading"><div><p className="eyebrow">Starting configuration</p><h2>Graphics recommendations</h2><p className="muted">Heuristics to try, then measure in the game. No FPS prediction.</p></div></div>
              <div className="panel graphics-inputs"><label>Connected display<select value={displayName} onChange={e => { setDisplayName(e.target.value); const display = system?.displays.find(d => d.name === e.target.value); if (display) setDimensions(`${display.width}x${display.height}`); }}><option value="">Choose a display or use a resolution profile</option>{system?.displays.map(d => <option key={d.name} value={d.name}>{d.name} · {d.width}×{d.height} @ {d.refresh_hz} Hz</option>)}</select></label><div className="setup-grid"><label>Resolution profile<select value={dimensions} onChange={e => setDimensions(e.target.value)}>{modes.map(mode => <option key={mode} value={mode}>{mode.replace('x', ' × ')}{system?.displays.some(d => `${d.width}x${d.height}` === mode) ? ' · detected display mode' : ' · 1080p/1440p profile'}</option>)}</select></label><label>Starting preset<select value={preset} onChange={e => setPreset(e.target.value)}><option value="auto">Automatic, based on assessment</option>{['Very Low', 'Low', 'Medium', 'High', 'Ultra'].map(p => <option key={p}>{p}</option>)}</select></label><div className="form-row"><label>Observed FPS<input type="number" min="1" max="1000" value={observed} onChange={e => setObserved(e.target.value)} placeholder="Optional" /></label><label>Target FPS<input type="number" min="1" max="1000" value={target} onChange={e => setTarget(e.target.value)} /></label></div></div></div>
              <div className="recommendations">{recommendations.map(s => <article key={s.setting}><div><h3>{s.setting}</h3><strong>{s.value}</strong></div><p>{s.reason}</p><small>{s.confidence}</small></article>)}</div>
              <p className="muted mono">Catalog {report.catalog_version} · Report contains hardware and installation paths. Share only with people you choose.</p>
            </> : <div className="empty-state"><Activity size={36} /><h2>Your assessment will appear here</h2><p>Choose a profile, GPU and installation location, then check compatibility.</p></div>}
          </>}
          {page === 'settings' && <>
            <div className="section-heading"><div><p className="eyebrow">Preferences & diagnostics</p><h1>Keep your analysis transparent.</h1></div></div>
            <div className="two-column"><section className="panel"><h2>Appearance</h2><label>Theme<select value={theme} onChange={e => { const value = e.target.value; if (value === 'dark' || value === 'light' || value === 'system') setTheme(value); }}><option value="system">Follow Windows theme</option><option value="dark">Dark</option><option value="light">Light</option></select></label><p className="muted">Stored locally. Motion follows your system's reduced-motion preference.</p></section><section className="panel"><h2>Privacy</h2><p>Hardware scanning and comparisons run on this PC. Game profiles are saved in the current user's application data directory.</p><p>No telemetry, accounts, ads or automatic hardware uploads. JSON reports include your hardware and paths.</p></section></div>
            <section className="panel"><div className="section-heading compact"><h2>Scanner diagnostics</h2><button className="secondary" disabled={!!busy} onClick={() => void refresh()}><RefreshCw size={16} />Rescan</button></div>{system ? system.diagnostics.length ? <ul>{system.diagnostics.map((d, i) => <li key={i}>{d}</li>)}</ul> : <p>No scanner errors reported. Missing properties can still appear as Unknown.</p> : <p>Scanner has not completed. Open the desktop application and retry.</p>}<details><summary>Detected adapters and displays</summary><pre>{JSON.stringify({ gpus: system?.gpus, displays: system?.displays }, null, 2)}</pre></details></section>
            <section className="panel"><h2>Comparison limits</h2><p>The initial CPU catalog verifies named models only. GPU comparisons use attributed Time Spy Graphics scores from one test set with a 25% uncertainty band. Near-equivalent hardware, missing models, laptop variants and different datasets produce Unknown.</p><p>DirectX 12 device support is probed on each adapter. It does not confirm every optional graphics feature, ray tracing capability or game API. SSD classification can be unavailable for RAID and virtual storage.</p></section>
          </>}
        </>}
      </main>
      <footer><span>GameSpec Checker</span><span>Evidence before assumptions.</span></footer>
    </div>
  </div>;
}
