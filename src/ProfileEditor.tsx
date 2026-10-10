import { useState } from 'react';
import { ArrowLeft, Save } from 'lucide-react';
import { profileSchema, requirementsSchema, type Profile, type Requirements } from './types';
import { errorMessage } from './api';

function TierFields({ title, value, onChange }: { title: string; value: Requirements; onChange: (v: Requirements) => void }) {
  const update = <K extends keyof Requirements>(key: K, next: Requirements[K]) => onChange({ ...value, [key]: next });
  return <fieldset className="tier-fields">
    <legend>{title}</legend>
    <label>CPU models or alternatives<textarea required value={value.cpu.join('\n')} onChange={e => update('cpu', e.target.value.split('\n'))} rows={3} /><small>One complete model per line. Uncatalogued models remain Unknown.</small></label>
    <label>GPU models or alternatives<textarea required value={value.gpu.join('\n')} onChange={e => update('gpu', e.target.value.split('\n'))} rows={3} /></label>
    <div className="form-row">
      <label>RAM · GiB<input type="number" min="0" max="100000" step="0.5" required value={value.ram_gib} onChange={e => update('ram_gib', e.target.valueAsNumber)} /></label>
      <label>Dedicated VRAM · GiB<input type="number" min="0" max="100000" step="0.5" required value={value.vram_gib} onChange={e => update('vram_gib', e.target.valueAsNumber)} /></label>
    </div>
    <label>Storage · GB<input type="number" min="0" max="100000" step="0.1" required value={value.storage_gb} onChange={e => update('storage_gb', e.target.valueAsNumber)} /></label>
    <label>Windows versions<select value={value.windows_versions.join(',')} onChange={e => update('windows_versions', e.target.value === '10' ? [10] : e.target.value === '11' ? [11] : [10, 11])}><option value="10,11">Windows 10 or 11</option><option value="10">Windows 10</option><option value="11">Windows 11</option></select></label>
    <label>Architecture<select value={value.architecture} onChange={e => update('architecture', requirementsSchema.shape.architecture.parse(e.target.value))}><option value="x86_64">64-bit x86</option><option value="x86">32-bit x86</option><option value="aarch64">ARM64</option></select></label>
    <label>Graphics API<input required value={value.graphics_api} onChange={e => update('graphics_api', e.target.value)} list="graphics-apis" /><small>DirectX 12 is probed. Other APIs may remain Unknown.</small></label>
    <label>Required feature level<select value={value.feature_level ?? ''} onChange={e => update('feature_level', requirementsSchema.shape.feature_level.parse(e.target.value || null))}><option value="">Not specified</option>{['11_0', '11_1', '12_0', '12_1', '12_2'].map(v => <option key={v}>{v}</option>)}</select></label>
    <label className="checkbox"><input type="checkbox" checked={value.ssd} onChange={e => update('ssd', e.target.checked)} />SSD {title === 'Minimum' ? 'required' : 'recommended'}</label>
  </fieldset>;
}

export default function ProfileEditor({ initial, busy, onSave, onCancel }: { initial: Profile; busy: boolean; onSave: (p: Profile) => Promise<void>; onCancel: () => void }) {
  const [profile, setProfile] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  return <form onSubmit={e => {
    e.preventDefault(); setError(null);
    try {
      const clean = { ...profile, minimum: { ...profile.minimum, cpu: profile.minimum.cpu.map(s => s.trim()).filter(Boolean), gpu: profile.minimum.gpu.map(s => s.trim()).filter(Boolean) }, recommended: { ...profile.recommended, cpu: profile.recommended.cpu.map(s => s.trim()).filter(Boolean), gpu: profile.recommended.gpu.map(s => s.trim()).filter(Boolean) } };
      void onSave(profileSchema.parse(clean)).catch(e => setError(errorMessage(e)));
    } catch (e) { setError(errorMessage(e)); }
  }}>
    <div className="section-heading"><div><p className="eyebrow">Requirements editor</p><h1>{initial.name ? `Edit ${initial.name}` : 'Add a game'}</h1></div><button type="button" className="quiet" onClick={onCancel}><ArrowLeft size={16} />Back to library</button></div>
    <p className="muted">Use requirements from the game's publisher. Every profile stays on this computer.</p>
    {error && <p role="alert" className="error">{error}</p>}
    <div className="panel form-row"><label>Game name<input required maxLength={200} value={profile.name} onChange={e => setProfile({ ...profile, name: e.target.value })} /></label><label>Game version<input maxLength={200} value={profile.version} onChange={e => setProfile({ ...profile, version: e.target.value })} /></label></div>
    <div className="two-column"><TierFields title="Minimum" value={profile.minimum} onChange={minimum => setProfile({ ...profile, minimum })} /><TierFields title="Recommended" value={profile.recommended} onChange={recommended => setProfile({ ...profile, recommended })} /></div>
    <label className="panel">Additional notes<textarea rows={4} maxLength={20000} value={profile.notes} onChange={e => setProfile({ ...profile, notes: e.target.value })} /></label>
    <datalist id="graphics-apis"><option value="DirectX 12" /><option value="DirectX 11" /><option value="Vulkan" /></datalist>
    <div className="editor-footer"><small>GB = decimal storage. GiB = binary memory capacity.</small><button disabled={busy} type="submit"><Save size={16} />{busy ? 'Saving…' : 'Save game profile'}</button></div>
  </form>;
}
