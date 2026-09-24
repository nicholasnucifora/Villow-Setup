import { createPrivateKey, createPublicKey, sign, verify } from 'node:crypto';
import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { sha256 } from './release-format.mjs';
import { readAuthenticatedChannel } from './channel-format.mjs';

export function signAppChannel(args=process.argv.slice(2), { renewalOnly=false, environment=process.env }={}) {
const [directory,keyId,sequenceText,...revoked]=args;
const sequence=Number(sequenceText);
if(!directory || !/^[A-Za-z0-9_-]{1,64}$/.test(keyId || '') || !Number.isSafeInteger(sequence) || sequence<1 || revoked.some(value=>!/^[a-f0-9]{64}$/.test(value))) throw new Error('Usage: sign-app-channel.mjs <release-directory> <key-id> <monotonic-sequence> [revoked-manifest-sha256 ...]');
const pointer=JSON.parse(readFileSync(resolve(directory,'pointer.json'),'utf8'));
const manifestBytes=readFileSync(resolve(directory,'manifest.json')),manifest=JSON.parse(manifestBytes);
if(sha256(manifestBytes)!==pointer.sha256 || pointer.version!==manifest.app_version || manifest.archive_sha256!==sha256(readFileSync(resolve(directory,'villow-source.zip')))
  || !/^https:\/\/github\.com\/[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+\/releases\/download\/villow-app-v[0-9.]+\/manifest\.json$/.test(pointer.url)
  || manifest.archive_url!==pointer.url.replace(/manifest\.json$/, 'villow-source.zip') || sequence<manifest.sequence) throw new Error('Release pointer does not match the immutable artifacts');
if(environment.VILLOW_RELEASE_SIGNING_KEY && environment.VILLOW_RELEASE_SIGNING_KEY_FILE) throw new Error('Choose either a protected signing environment key or an encrypted key file.');
let key;
try {
  key=createPrivateKey(environment.VILLOW_RELEASE_SIGNING_KEY_FILE
    ? {key:readFileSync(environment.VILLOW_RELEASE_SIGNING_KEY_FILE),passphrase:environment.VILLOW_RELEASE_KEY_PASSPHRASE}
    : environment.VILLOW_RELEASE_SIGNING_KEY || '');
} catch { throw new Error('Cannot open the publisher signing key. Check the configured key file/passphrase or protected environment secret.'); }
if(key.asymmetricKeyType!=='ed25519') throw new Error('An explicit Ed25519 publisher signing key is required');
const publicKey=createPublicKey(key);
const expected=environment.VILLOW_RELEASE_PUBLIC_KEY;
const rawKey=publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64');
if(!expected || rawKey!==expected) throw new Error('Signing key does not match the separately configured publisher public key');
let previousReleases=[];
const revokedSet=new Set(revoked);
if(environment.VILLOW_PREVIOUS_CHANNEL) {
  const repository=new URL(pointer.url).pathname.split('/').slice(1,3).join('/');
  const channel=readAuthenticatedChannel(JSON.parse(readFileSync(environment.VILLOW_PREVIOUS_CHANNEL,'utf8')),{publicKey,keyId,repository});
  if(sequence<=channel.sequence)throw new Error('Channel sequence must increase');
  if(channel.releases.some(p=>p.version===pointer.version && p.sha256!==pointer.sha256)) throw new Error('Cannot replace an immutable app version. Bump the app version and publish a new tag.');
  if(renewalOnly && (revoked.length || channel.releases[0].sha256!==pointer.sha256 || channel.releases[0].url!==pointer.url)) throw new Error('Renewal must preserve the recommended release and revocations.');
  for(const digest of channel.revoked)revokedSet.add(digest);
  previousReleases=channel.releases.filter(p=>p.sha256!==pointer.sha256 && !revokedSet.has(p.sha256));
} else if(renewalOnly || environment.VILLOW_INITIAL_CHANNEL!=='true' || sequence!==1) throw new Error('Supply VILLOW_PREVIOUS_CHANNEL, or explicitly initialize sequence 1');
if(revokedSet.has(pointer.sha256))throw new Error('Cannot recommend a revoked release');
const now=new Date(), expires=new Date(now.getTime()+6*86400000);
const payload=Buffer.from(JSON.stringify({format:1,channel:'stable',sequence,generated_at:now.toISOString(),expires_at:expires.toISOString(),releases:[pointer,...previousReleases],revoked:[...revokedSet]}));
const signature=sign(null,payload,key);
if(!verify(null,payload,publicKey,signature)) throw new Error('Signature self-check failed');
writeFileSync(resolve(directory,'channel.json'),JSON.stringify({key_id:keyId,payload:payload.toString('base64'),signature:signature.toString('base64')},null,2)+'\n');
console.log('Signed channel prepared for review; no assets were published.');
return {sequence,expires_at:expires.toISOString(),recommended_manifest_sha256:pointer.sha256};
}

if(process.argv[1] && import.meta.url===pathToFileURL(resolve(process.argv[1])).href) signAppChannel();
