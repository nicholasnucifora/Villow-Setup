import { test } from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPairSync, sign } from 'node:crypto';
import { validateRenewal, downloadChannel, verifyDownloadedHash } from '../verify-channel-publication.mjs';
import { sha256 } from '../release-format.mjs';

const { privateKey, publicKey } = generateKeyPairSync('ed25519');
const options = { repository: 'synthetic-owner/test-releases', keyId: 'synthetic-only', publicKey };
const now = Date.parse('2026-09-24T00:00:00Z');
const releases = [{ version: '0.1.2', sha256: 'a'.repeat(64), url: 'https://github.com/synthetic-owner/test-releases/releases/download/villow-app-v0.1.2/manifest.json' }];
const old = { format: 1, channel: 'stable', sequence: 3, generated_at: '2020-01-01T00:00:00Z', expires_at: '2020-01-07T00:00:00Z', releases, revoked: ['b'.repeat(64)] };
const next = { ...old, sequence: 4, generated_at: '2026-09-24T00:00:00Z', expires_at: '2026-09-30T00:00:00Z' };
function signed(payload) {
  const bytes = Buffer.from(JSON.stringify(payload));
  return Buffer.from(JSON.stringify({ key_id: options.keyId, payload: bytes.toString('base64'), signature: sign(null, bytes, privateKey).toString('base64') }));
}
const previous = signed(old);
const plan = { repository: options.repository, previous_channel_sha256: sha256(previous), next_sequence: 4 };

test('verifies an expired authentic source renewed without changing releases', () => {
  assert.deepEqual(validateRenewal(previous, signed(next), plan, options, now), next);
});
test('rejects changed release identity, missing revocations and stale sequence', () => {
  for (const changed of [
    { ...next, releases: [{ ...releases[0], sha256: 'c'.repeat(64) }] },
    { ...next, revoked: [] },
    { ...next, sequence: 3 },
    { ...next, sequence: 5 },
  ]) assert.throws(() => validateRenewal(previous, signed(changed), plan, options, now));
});
test('rejects expiry, future timestamps, wrong source and forged signature', () => {
  assert.throws(() => validateRenewal(previous, signed({ ...old, sequence: 4 }), plan, options, now));
  assert.throws(() => validateRenewal(previous, signed({ ...next, generated_at: '2026-09-25T00:00:00Z' }), plan, options, now));
  assert.throws(() => validateRenewal(previous, signed(next), { ...plan, previous_channel_sha256: '0'.repeat(64) }, options, now));
  const forged = JSON.parse(signed(next));
  forged.payload = Buffer.from(JSON.stringify({ ...next, sequence: 5 })).toString('base64');
  assert.throws(() => validateRenewal(previous, Buffer.from(JSON.stringify(forged)), plan, options, now));
});
test('download check refuses stale bytes and HTTP errors without sending credentials', async () => {
  const candidate = signed(next);
  await verifyDownloadedHash(sha256(candidate), async (url, request) => {
    assert.equal(url, 'https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json');
    assert.equal(request.headers, undefined);
    return new Response(candidate);
  });
  await assert.rejects(verifyDownloadedHash(sha256(candidate), async () => new Response(previous)));
  await assert.rejects(downloadChannel(async () => new Response('', { status: 503 })));
  await assert.rejects(downloadChannel(async () => new Response(Buffer.alloc(1024 * 1024 + 1))));
});
