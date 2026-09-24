// Fetches authenticated data only. Never rebuilds source or publishes a channel.
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { publisherKey, readAuthenticatedChannel } from './channel-format.mjs';
import { sha256 } from './release-format.mjs';
const repository='nicholasnucifora/Villow-Setup';
const directory=resolve(process.argv[2] || 'artifacts/channel-renewal');
const channelUrl=`https://github.com/${repository}/releases/download/villow-channel/channel.json`;
async function download(url,limit) {
  const response=await fetch(url,{signal:AbortSignal.timeout(120000)});
  if(!response.ok || !response.body) throw new Error(`Release download failed: HTTP ${response.status}`);
  const chunks=[]; let size=0;
  for await(const chunk of response.body) {
    size+=chunk.length; if(size>limit) throw new Error('Release asset exceeds its size limit.'); chunks.push(chunk);
  }
  return Buffer.concat(chunks);
}
const previous=await download(channelUrl,1024*1024);
const channel=readAuthenticatedChannel(JSON.parse(previous),{
  publicKey:publisherKey(process.env.VILLOW_RELEASE_PUBLIC_KEY),keyId:process.env.VILLOW_RELEASE_KEY_ID,repository,
});
const pointer=channel.releases[0];
const manifestBytes=await download(pointer.url,8*1024*1024);
if(sha256(manifestBytes)!==pointer.sha256) throw new Error('Published manifest differs from the authenticated channel.');
const manifest=JSON.parse(manifestBytes);
if(manifest.app_version!==pointer.version || manifest.archive_url!==pointer.url.replace(/manifest\.json$/,'villow-source.zip') ||
  !Number.isSafeInteger(manifest.archive_size) || manifest.archive_size<1 || manifest.archive_size>128*1024*1024) {
  throw new Error('Published manifest has an invalid archive reference.');
}
const archive=await download(manifest.archive_url,manifest.archive_size);
if(archive.length!==manifest.archive_size || sha256(archive)!==manifest.archive_sha256) throw new Error('Published archive checksum/size differs.');
if(!Number.isSafeInteger(channel.sequence+1)) throw new Error('Channel sequence exhausted.');
mkdirSync(directory,{recursive:true});
for(const [path,bytes] of Object.entries({
  'previous-channel.json':previous,'manifest.json':manifestBytes,'villow-source.zip':archive,
  'pointer.json':JSON.stringify(pointer,null,2)+'\n',
  'renewal-plan.json':JSON.stringify({repository,previous_channel_sha256:sha256(previous),next_sequence:channel.sequence+1,previous_expires_at:channel.expires_at},null,2)+'\n',
})) writeFileSync(resolve(directory,path),bytes);
console.log('Authenticated original release artifacts are ready for renewal in '+directory);
