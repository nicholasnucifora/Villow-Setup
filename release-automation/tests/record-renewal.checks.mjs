import {test} from 'node:test';
import assert from 'node:assert/strict';
import {saveReceipt,recordBranch} from '../record-renewal.mjs';

const receipt={sequence:4,workflow_commit:'a'.repeat(40),channel_sha256:'b'.repeat(64)};
test('creates only the separate receipt branch and a public receipt file',async()=>{
  const calls=[];
  await saveReceipt(receipt,async(path,method='GET',body)=>{
    calls.push({path,method,body});
    return method==='POST'||method==='PUT'?{status:201,data:{}}:{status:404,data:{}};
  });
  assert.equal(calls[1].body.ref,'refs/heads/'+recordBranch);
  assert.equal(calls[3].body.branch,recordBranch);
  assert.deepEqual(JSON.parse(Buffer.from(calls[3].body.content,'base64')),receipt);
});
test('updates with the previous blob identity and refuses a newer receipt',async()=>{
  let priorSequence=3,writes=0;
  const request=async(path,method='GET',body)=>{
    if(method==='PUT'){writes++;assert.equal(body.sha,'prior-blob');return{status:200,data:{}};}
    if(path.includes('/contents/'))return{status:200,data:{sha:'prior-blob',content:Buffer.from(JSON.stringify({sequence:priorSequence})).toString('base64')}};
    return{status:200,data:{}};
  };
  await saveReceipt(receipt,request);
  priorSequence=5;
  await assert.rejects(saveReceipt(receipt,request),/newer/);
  assert.equal(writes,1);
});
test('reports failed writes instead of recording successful maintenance',async()=>{
  await assert.rejects(saveReceipt(receipt,async(path,method='GET')=>method==='PUT'?{status:409,data:{}}:path.includes('/contents/')?{status:404,data:{}}:{status:200,data:{}}),/save/);
});
