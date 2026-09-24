import { createPublicKey, verify } from 'node:crypto';

export function publisherKey(raw) {
  if (!/^[A-Za-z0-9+/]{43}=$/.test(raw || '')) throw new Error('Configure a 32-byte base64 Ed25519 public key.');
  return createPublicKey({ format:'der', type:'spki', key:Buffer.concat([
    Buffer.from('302a300506032b6570032100','hex'),Buffer.from(raw,'base64'),
  ]) });
}

export function readAuthenticatedChannel(envelope, { publicKey, keyId, repository }) {
  if (envelope.key_id !== keyId || typeof envelope.payload !== 'string' || typeof envelope.signature !== 'string') {
    throw new Error('Previous channel publisher/key ID does not match.');
  }
  const bytes=Buffer.from(envelope.payload,'base64');
  if(!verify(null,bytes,publicKey,Buffer.from(envelope.signature,'base64'))) throw new Error('Previous channel is not authenticated.');
  const channel=JSON.parse(bytes);
  const generated=Date.parse(channel.generated_at),expires=Date.parse(channel.expires_at);
  if(channel.format!==1 || channel.channel!=='stable' || !Number.isSafeInteger(channel.sequence) || channel.sequence<1 ||
    !Number.isFinite(generated) || !Number.isFinite(expires) || expires<=generated || expires-generated>7*86400000 ||
    !Array.isArray(channel.releases) || channel.releases.length===0 || !Array.isArray(channel.revoked) ||
    channel.revoked.some(hash=>!/^[a-f0-9]{64}$/.test(hash))) throw new Error('Previous channel contract is invalid.');
  const versions=new Set(),digests=new Set();
  for(const pointer of channel.releases) {
    if(!/^\d+\.\d+\.\d+$/.test(pointer.version) || !/^[a-f0-9]{64}$/.test(pointer.sha256) ||
      pointer.url!==`https://github.com/${repository}/releases/download/villow-app-v${pointer.version}/manifest.json` ||
      versions.has(pointer.version) || digests.has(pointer.sha256) || channel.revoked.includes(pointer.sha256)) {
      throw new Error('Previous channel has an invalid, duplicate or revoked release pointer.');
    }
    versions.add(pointer.version); digests.add(pointer.sha256);
  }
  // An expired authentic channel can be renewed. Installers still enforce freshness.
  return channel;
}
