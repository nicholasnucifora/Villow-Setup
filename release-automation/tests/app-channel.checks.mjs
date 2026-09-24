import { test, after } from 'node:test';
import assert from 'node:assert/strict';
import { generateKeyPairSync, sign } from 'node:crypto';
import { mkdtempSync, writeFileSync, readFileSync, rmSync } from 'node:fs';
import { resolve } from 'node:path';
import { tmpdir } from 'node:os';
import { signAppChannel } from '../sign-app-channel.mjs';
import { readAuthenticatedChannel, publisherKey } from '../channel-format.mjs';
import { sha256 } from '../release-format.mjs';
const directory=mkdtempSync(resolve(tmpdir(),'villow-channel-tests-'));
after(()=>rmSync(directory,{recursive:true,force:true}));
const {privateKey,publicKey}=generateKeyPairSync('ed25519');
const raw=publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64');
const repository='test-owner/test-releases',keyId='test-only';
const environment={VILLOW_RELEASE_SIGNING_KEY:privateKey.export({format:'pem',type:'pkcs8'}).toString(),VILLOW_RELEASE_PUBLIC_KEY:raw,VILLOW_INITIAL_CHANNEL:'true'};
const pointer=(version,hash)=>({version,sha256:hash,url:`https://github.com/${repository}/releases/download/villow-app-v${version}/manifest.json`});
function release(version='0.1.0') {
  const archive=Buffer.from('synthetic archive');
  const manifest=Buffer.from(JSON.stringify({app_version:version,sequence:1,archive_sha256:sha256(archive),archive_url:pointer(version,'').url.replace('manifest.json','villow-source.zip')}));
  const p=pointer(version,sha256(manifest));
  writeFileSync(resolve(directory,'villow-source.zip'),archive);
  writeFileSync(resolve(directory,'manifest.json'),manifest);
  writeFileSync(resolve(directory,'pointer.json'),JSON.stringify(p));
  return p;
}
function previous(payload) {
  const bytes=Buffer.from(JSON.stringify(payload));
  const envelope={key_id:keyId,payload:bytes.toString('base64'),signature:sign(null,bytes,privateKey).toString('base64')};
  const path=resolve(directory,'previous.json'); writeFileSync(path,JSON.stringify(envelope)); return path;
}
function channel(releases,revoked=[]) {
  // Deliberately expired: renewal must be able to recover from a missed run.
  return {format:1,channel:'stable',sequence:4,generated_at:'2020-01-01T00:00:00Z',expires_at:'2020-01-07T00:00:00Z',releases,revoked};
}
function current() {
  return readAuthenticatedChannel(JSON.parse(readFileSync(resolve(directory,'channel.json'))),{publicKey:publisherKey(raw),keyId,repository});
}
test('initial channel needs explicit initialization and the separately pinned key',()=>{
  release();
  assert.throws(()=>signAppChannel([directory,keyId,'1'],{environment:{...environment,VILLOW_INITIAL_CHANNEL:'false'}}),/explicitly initialize/);
  assert.throws(()=>signAppChannel([directory,keyId,'1'],{environment:{...environment,VILLOW_RELEASE_PUBLIC_KEY:'wrong'}}),/separately configured/);
  signAppChannel([directory,keyId,'1'],{environment});
  assert.equal(current().sequence,1);
  assert.equal(Date.parse(current().expires_at)-Date.parse(current().generated_at),6*86400000);
});
test('renewal preserves all supported releases and revocations and increases sequence',()=>{
  const p=release(), payload=channel([p,pointer('0.0.9','b'.repeat(64))],['c'.repeat(64)]);
  const env={...environment,VILLOW_PREVIOUS_CHANNEL:previous(payload)};
  signAppChannel([directory,keyId,'5'],{renewalOnly:true,environment:env});
  assert.deepEqual(current().releases,payload.releases); assert.deepEqual(current().revoked,payload.revoked);
  assert.throws(()=>signAppChannel([directory,keyId,'4'],{environment:env}),/sequence must increase/);
  assert.throws(()=>signAppChannel([directory,keyId,'5','d'.repeat(64)],{renewalOnly:true,environment:env}),/preserve/);
  release('0.2.0');
  assert.throws(()=>signAppChannel([directory,keyId,'5'],{renewalOnly:true,environment:env}),/preserve/);
  signAppChannel([directory,keyId,'5','b'.repeat(64)],{environment:env});
  assert.deepEqual(current().releases.map(p=>p.version),['0.2.0','0.1.0']);
  assert.ok(current().revoked.includes('c'.repeat(64))); assert.ok(current().revoked.includes('b'.repeat(64)));
});
test('refuses tampering, mismatched publisher and immutable version replacement',()=>{
  const p=release(),payload=channel([p]);
  const path=previous(payload),env={...environment,VILLOW_PREVIOUS_CHANNEL:path};
  const envelope=JSON.parse(readFileSync(path)); envelope.key_id='other'; writeFileSync(path,JSON.stringify(envelope));
  assert.throws(()=>signAppChannel([directory,keyId,'5'],{environment:env}),/publisher\/key ID/);
  previous(payload); const tampered=JSON.parse(readFileSync(path)); tampered.payload=Buffer.from('{}').toString('base64'); writeFileSync(path,JSON.stringify(tampered));
  assert.throws(()=>signAppChannel([directory,keyId,'5'],{environment:env}),/not authenticated/);
  previous(channel([pointer('0.1.0','d'.repeat(64))]));
  assert.throws(()=>signAppChannel([directory,keyId,'5'],{environment:env}),/immutable app version/);
});
test('accepts an encrypted private key file without exposing its contents',()=>{
  release(); const path=resolve(directory,'synthetic-encrypted-key.pem'),passphrase='synthetic testing passphrase only';
  writeFileSync(path,privateKey.export({format:'pem',type:'pkcs8',cipher:'aes-256-cbc',passphrase}));
  const env={...environment,VILLOW_RELEASE_SIGNING_KEY:undefined,VILLOW_RELEASE_SIGNING_KEY_FILE:path,VILLOW_RELEASE_KEY_PASSPHRASE:passphrase};
  signAppChannel([directory,keyId,'1'],{environment:env}); assert.equal(current().sequence,1);
  assert.throws(()=>signAppChannel([directory,keyId,'1'],{environment:{...env,VILLOW_RELEASE_KEY_PASSPHRASE:'wrong'}}),/Cannot open/);
});
