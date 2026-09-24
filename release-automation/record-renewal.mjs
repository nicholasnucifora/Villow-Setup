import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { pathToFileURL } from 'node:url';
import { publisherKey } from './channel-format.mjs';
import { sha256 } from './release-format.mjs';
import { validateRenewal, verifyDownloadedHash } from './verify-channel-publication.mjs';

const repository = 'nicholasnucifora/Villow-Setup';
export const recordBranch = 'codex/channel-renewal-records';
const recordPath = `/repos/${repository}/contents/renewal/latest.json`;

export async function saveReceipt(receipt, request) {
  if (!/^[a-f0-9]{40}$/.test(receipt.workflow_commit) || !Number.isSafeInteger(receipt.sequence) || receipt.sequence < 1) throw new Error('Invalid renewal receipt');
  const refPath = `/repos/${repository}/git/ref/heads/${recordBranch}`;
  const branch = await request(refPath);
  if (branch.status === 404) {
    const created = await request(`/repos/${repository}/git/refs`, 'POST', {ref:`refs/heads/${recordBranch}`, sha:receipt.workflow_commit});
    if (created.status !== 201) throw new Error('Could not create the renewal record branch');
  } else if (branch.status !== 200) throw new Error('Could not check the renewal record branch');
  const old = await request(`${recordPath}?ref=${encodeURIComponent(recordBranch)}`);
  if (![200,404].includes(old.status)) throw new Error('Could not check the prior renewal receipt');
  let prior;
  if (old.status === 200) {
    prior = JSON.parse(Buffer.from(old.data.content, 'base64').toString());
    if (!Number.isSafeInteger(prior.sequence) || prior.sequence >= receipt.sequence) throw new Error('A same or newer renewal receipt is already recorded');
  }
  const result = await request(recordPath, 'PUT', {
    message:`Record verified Villow channel sequence ${receipt.sequence}`,
    branch:recordBranch,
    content:Buffer.from(JSON.stringify(receipt,null,2)+'\n').toString('base64'),
    ...(old.status===200?{sha:old.data.sha}:{}),
  });
  if (![200,201].includes(result.status)) throw new Error('Could not save the verified renewal receipt');
}

async function main() {
  const directory=process.argv[2];
  if (!directory || process.env.GITHUB_REPOSITORY !== repository || !process.env.GH_TOKEN || !/^\d+$/.test(process.env.GITHUB_RUN_ID || '')) throw new Error('Expected the authenticated distribution workflow');
  const previous=readFileSync(resolve(directory,'previous-channel.json'));
  const bytes=readFileSync(resolve(directory,'channel.json'));
  const plan=JSON.parse(readFileSync(resolve(directory,'renewal-plan.json')));
  const channel=validateRenewal(previous,bytes,plan,{repository,keyId:process.env.VILLOW_RELEASE_KEY_ID,publicKey:publisherKey(process.env.VILLOW_RELEASE_PUBLIC_KEY)});
  await verifyDownloadedHash(sha256(bytes));
  const receipt={format:1,sequence:channel.sequence,expires_at:channel.expires_at,channel_sha256:sha256(bytes),channel:JSON.parse(bytes),verified_at:new Date().toISOString(),workflow_commit:process.env.GITHUB_SHA,run_url:`https://github.com/${repository}/actions/runs/${process.env.GITHUB_RUN_ID}`};
  const request=async(path,method='GET',body)=>{
    const response=await fetch('https://api.github.com'+path,{method,headers:{Authorization:'Bearer '+process.env.GH_TOKEN,Accept:'application/vnd.github+json','X-GitHub-Api-Version':'2022-11-28',...(body?{'Content-Type':'application/json'}:{})},...(body?{body:JSON.stringify(body)}:{}),signal:AbortSignal.timeout(30000)});
    return {status:response.status,data:await response.json()};
  };
  await saveReceipt(receipt,request);
  console.log('Verified public renewal receipt saved on the separate record branch.');
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href)main().catch(()=>{console.error('Renewal receipt recording failed. Check the published listing and repository activity.');process.exitCode=1;});
