import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { publisherKey, readAuthenticatedChannel } from './channel-format.mjs';
import { sha256 } from './release-format.mjs';

const repository = 'nicholasnucifora/Villow-Setup';
const channelUrl = `https://github.com/${repository}/releases/download/villow-channel/channel.json`;
const maxBytes = 1024 * 1024;
const publicationDelays = [0, 5000, 10000, 20000, 30000, 45000, 60000];

// Only these controlled diagnostics may reach logs; never print raw fetch errors
// (which can include expiring redirect URLs) or downloaded response bodies.
class PublicChannelError extends Error {}

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
  let response;
  try {
    response = await fetcher(channelUrl, {
      headers: { 'Cache-Control': 'no-cache' }, signal: AbortSignal.timeout(30000),
    });
  } catch { throw new PublicChannelError('Public channel request failed or timed out'); }
  if (!response.ok) throw new PublicChannelError(`Public channel returned HTTP ${response.status}`);
  if (!response.body) throw new PublicChannelError('Public channel response has no body');
  const chunks = [];
  let length = 0;
  try {
    for await (const chunk of response.body) {
      length += chunk.length;
      if (length > maxBytes) throw new PublicChannelError('Public channel exceeds its size limit');
      chunks.push(Buffer.from(chunk));
    }
  } catch (error) {
    if (error instanceof PublicChannelError) throw error;
    throw new PublicChannelError('Public channel response was interrupted');
  }
  return Buffer.concat(chunks);
}

export async function verifyDownloadedHash(expected, fetcher = fetch) {
  assert.match(expected, /^[a-f0-9]{64}$/, 'Expected a channel SHA-256');
  const actual = sha256(await downloadChannel(fetcher));
  if (actual !== expected) throw new PublicChannelError(`Public channel SHA-256 differs: expected ${expected}; received ${actual}`);
}

export async function verifyPublishedChannel(expected, {
  fetcher = fetch, wait = ms => new Promise(done => setTimeout(done, ms)), onRetry = () => {},
} = {}) {
  // Replacement assets can briefly return old bytes or 404s. Retry only reads
  // of the real user-facing URL; every success still requires the exact hash.
  for (const [index, delay] of publicationDelays.entries()) {
    if (delay) await wait(delay);
    try { await verifyDownloadedHash(expected, fetcher); return; }
    catch (error) {
      if (!(error instanceof PublicChannelError)) throw error;
      onRetry(`Public download check ${index + 1}/${publicationDelays.length}: ${error.message}`);
      if (index === publicationDelays.length - 1) throw new PublicChannelError(`Published renewal could not be verified anonymously: ${error.message}`);
    }
  }
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
    await verifyPublishedChannel(sha256(candidate), { onRetry: message => console.log(message) });
  }
  console.log(mode === 'before' ? 'Renewal and unchanged public source verified.' : 'Published renewal verified anonymously.');
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  main().catch(error => {
    console.error(error instanceof PublicChannelError ? error.message : 'Channel verification failed; inspect the renewal job before retrying publication.');
    process.exitCode = 1;
  });
}
