import { describe, expect, it } from 'vitest';
import { profileSchema, reportSchema } from './types';
import sample from '../data/icarus.json';
import amd from '../crates/gamespec-core/tests/fixtures/amd.json';
import { snapshotSchema } from './types';

describe('profile import boundary', () => {
  it('accepts the editable Icarus sample', () => { expect(profileSchema.parse(sample).name).toBe('Icarus'); });
  it('rejects unsupported versions, traversal and malformed data', () => {
    for (const value of [{ ...sample, schema_version: 2 }, { ...sample, id: '../bad' }, { ...sample, minimum: { ...sample.minimum, ram_gib: -1 } }, { ...sample, minimum: { ...sample.minimum, cpu: [] } }, { ...sample, extra: 'unrecognized' }]) expect(profileSchema.safeParse(value).success).toBe(false);
  });
  it('rejects inconsistent recommendations', () => { expect(profileSchema.safeParse({ ...sample, recommended: { ...sample.recommended, ram_gib: 1 } }).success).toBe(false); });
});
describe('native IPC boundary', () => {
  it('accepts the shared hardware fixture and preserves dedicated memory', () => { expect(snapshotSchema.parse(amd).gpus[0]?.dedicated_bytes).toBe(8 * 1073741824); });
  it('rejects invented verdicts and incomplete reports', () => { expect(reportSchema.safeParse({ verdict: '60fps-guaranteed' }).success).toBe(false); });
});
