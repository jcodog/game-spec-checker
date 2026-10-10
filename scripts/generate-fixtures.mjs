import { mkdirSync, writeFileSync } from 'node:fs';

const gib = 1073741824;
const gpu = (id, model, manufacturer, vram, active = true) => ({ id, model, manufacturer, dedicated_bytes: vram * gib, driver_version: 'fixture driver', active, directx12: true, feature_level: '12_1' });
const base = {
  cpu: { model: 'AMD Ryzen 7 3700X 8-Core Processor', manufacturer: 'AuthenticAMD', architecture: 'x86_64', physical_cores: 8, logical_threads: 16, capabilities: ['AVX2'] },
  ram_total_bytes: 16 * gib, ram_available_bytes: 12 * gib,
  gpus: [gpu('amd', 'AMD Radeon RX 7600', 'AMD', 8)],
  os: { edition: 'Microsoft Windows 11 Pro', version: '10.0.26100', build: 26100, architecture: 'x86_64' },
  volumes: [{ mount: 'C:\\', name: 'Fixture SSD', total_bytes: 1e12, free_bytes: 500e9, file_system: 'NTFS', media: 'ssd' }],
  displays: [{ name: 'Fixture display', width: 1920, height: 1080, refresh_hz: 60 }], diagnostics: [],
};
const fixtures = {
  amd: base,
  nvidia: { ...base, gpus: [gpu('nvidia', 'NVIDIA GeForce RTX 2070', 'NVIDIA', 8)] },
  intel: { ...base, cpu: { ...base.cpu, model: '11th Gen Intel(R) Core(TM) i7-11700 @ 2.50GHz', manufacturer: 'GenuineIntel' }, gpus: [gpu('intel', 'Intel Arc A770', 'Intel', 16)] },
  multi_gpu: { ...base, gpus: [gpu('integrated', 'Intel UHD Graphics 630', 'Intel', 0), gpu('discrete', 'AMD Radeon RX 7600', 'AMD', 8, false)] },
  inadequate_storage: { ...base, volumes: [{ ...base.volumes[0], free_bytes: 50e9, media: 'hdd' }] },
  missing: { cpu: { model: null, manufacturer: null, architecture: null, physical_cores: null, logical_threads: null, capabilities: [] }, ram_total_bytes: null, ram_available_bytes: null, gpus: [], os: { edition: null, version: null, build: null, architecture: null }, volumes: [], displays: [], diagnostics: ['Test fixture: all properties unavailable'] },
};
mkdirSync('crates/gamespec-core/tests/fixtures', { recursive: true });
for (const [name, value] of Object.entries(fixtures)) writeFileSync(`crates/gamespec-core/tests/fixtures/${name}.json`, `${JSON.stringify(value, null, 2)}\n`);
