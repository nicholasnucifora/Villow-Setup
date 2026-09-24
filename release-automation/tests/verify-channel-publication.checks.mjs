import { test } from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPairSync, sign } from 'node:crypto';
import { validateRenewal, downloadChannel, verifyDownloadedHash, verifyPublishedChannel } from '../verify-channel-publication.mjs';
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
    assert.deepEqual(request.headers, { 'Cache-Control': 'no-cache' });
    return new Response(candidate);
  });
  await assert.rejects(verifyDownloadedHash(sha256(candidate), async () => new Response(previous)));
  await assert.rejects(downloadChannel(async () => new Response('', { status: 503 })));
  await assert.rejects(downloadChannel(async () => new Response(Buffer.alloc(1024 * 1024 + 1))));
});

test('publication tolerates propagation beyond the old 18-second window, using only anonymous reads', async () => {
  const candidate = signed(next), delays = [], messages = [];
  let calls = 0;
  await verifyPublishedChannel(sha256(candidate), {
    wait: async ms => { delays.push(ms); }, onRetry: message => messages.push(message),
    fetcher: async (url, request) => {
      assert.equal(url, 'https://github.com/nicholasnucifora/Villow-Setup/releases/download/villow-channel/channel.json');
      assert.deepEqual(request.headers, { 'Cache-Control': 'no-cache' });
      assert.equal(request.method, undefined);
      calls++;
      if (calls === 1) throw new Error('sensitive redirect URL must not appear');
      if (calls === 2) return new Response('private response body', { status: 404 });
      if (calls <= 4) return new Response(previous);
      return new Response(candidate);
    },
  });
  assert.equal(calls, 5);
  assert.deepEqual(delays, [5000, 10000, 20000, 30000]);
  assert.match(messages[1], /HTTP 404/);
  assert.match(messages[2], new RegExp(sha256(previous)));
  assert.doesNotMatch(messages.join('\n'), /sensitive|private response/);
});

test('persistent stale bytes fail after a bounded read-only retry budget', async () => {
  const delays = [], messages = [];
  let calls = 0;
  await assert.rejects(verifyPublishedChannel(sha256(signed(next)), {
    wait: async ms => { delays.push(ms); }, onRetry: message => messages.push(message),
    fetcher: async () => { calls++; return new Response(previous); },
  }), /could not be verified anonymously.*SHA-256 differs/);
  assert.equal(calls, 7);
  assert.equal(messages.length, 7);
  assert.equal(delays.reduce((a, b) => a + b, 0), 170000);
});

test('read errors give safe diagnostics and invalid expected hashes do not make requests', async () => {
  await assert.rejects(downloadChannel(async () => new Response('hidden body', { status: 503 })), /HTTP 503/);
  await assert.rejects(downloadChannel(async () => { throw new Error('secret'); }), error => {
    assert.equal(error.message, 'Public channel request failed or timed out'); return true;
  });
  const broken = new ReadableStream({ start(controller) { controller.error(new Error('secret')); } });
  await assert.rejects(downloadChannel(async () => new Response(broken)), /response was interrupted/);
  let calls = 0;
  await assert.rejects(verifyPublishedChannel('invalid', { fetcher: async () => { calls++; } }), /Expected a channel SHA-256/);
  assert.equal(calls, 0);
});
