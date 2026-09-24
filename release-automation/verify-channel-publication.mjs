import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { publisherKey, readAuthenticatedChannel } from './channel-format.mjs';
import { sha256 } from './release-format.mjs';

const repository = 'nicholasnucifora/Villow-Setup';
const channelUrl = `https://github.com/${repository}/releases/download/villow-channel/channel.json`;
const maxBytes = 1024 * 1024;

// No signing or publishing credentials are needed in this independent check.
export function validateRenewal(previousBytes, candidateBytes, plan, options, now = Date.now()) {
  assert.equal(plan.repository, options.repository, 'Unexpected distribution repository');
  assert.equal(sha256(previousBytes), plan.previous_channel_sha256, 'Previous channel bytes changed');
  const previous = readAuthenticatedChannel(JSON.parse(previousBytes), options);
  const candidate = readAuthenticatedChannel(JSON.parse(candidateBytes), options);
  assert.equal(candidate.sequence, previous.sequence + 1, 'Renewal must advance exactly one sequence');
  assert.equal(candidate.sequence, plan.next_sequence, 'Renewal differs from the prepared plan');
  assert.deepEqual(candidate.releases, previous.releases, 'Renewal must preserve every release');
  assert.deepEqual(candidate.revoked, previous.revoked, 'Renewal must preserve every revocation');
  assert.ok(Date.parse(candidate.generated_at) <= now + 300000, 'Candidate is dated in the future');
  assert.ok(Date.parse(candidate.expires_at) > now, 'Candidate has expired');
  return candidate;
}

export async function downloadChannel(fetcher = fetch) {
  const response = await fetcher(channelUrl, { signal: AbortSignal.timeout(30000) });
  if (!response.ok || !response.body) throw new Error('Public channel download failed');
  const chunks = [];
  let length = 0;
  for await (const chunk of response.body) {
    length += chunk.length;
    if (length > maxBytes) throw new Error('Public channel exceeds its size limit');
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks);
}

export async function verifyDownloadedHash(expected, fetcher = fetch) {
  assert.equal(sha256(await downloadChannel(fetcher)), expected, 'Public channel differs from the expected bytes');
}

async function main() {
  const [mode, directory] = process.argv.slice(2);
  if (!['before', 'after'].includes(mode) || !directory) throw new Error('Expected before|after and prepared directory');
  const previous = readFileSync(resolve(directory, 'previous-channel.json'));
  const candidate = readFileSync(resolve(directory, 'channel.json'));
  const plan = JSON.parse(readFileSync(resolve(directory, 'renewal-plan.json')));
  validateRenewal(previous, candidate, plan, {
    repository, keyId: process.env.VILLOW_RELEASE_KEY_ID,
    publicKey: publisherKey(process.env.VILLOW_RELEASE_PUBLIC_KEY),
  });
  if (mode === 'before') {
    await verifyDownloadedHash(plan.previous_channel_sha256);
  } else {
    // Allow short CDN propagation delays. A stale or uncertain result never passes.
    let passed = false;
    for (const delay of [0, 3000, 5000, 10000]) {
      if (delay) await new Promise(resolveDelay => setTimeout(resolveDelay, delay));
      try { await verifyDownloadedHash(sha256(candidate)); passed = true; break; }
      catch { /* Retry a bounded read, never a publishing write. */ }
    }
    if (!passed) throw new Error('Published renewal could not be verified anonymously');
  }
  console.log(mode === 'before' ? 'Renewal and unchanged public source verified.' : 'Published renewal verified anonymously.');
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch(() => { console.error('Channel verification failed; inspect the renewal job before retrying publication.'); process.exitCode = 1; });
}
