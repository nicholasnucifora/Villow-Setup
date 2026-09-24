import { signAppChannel } from './sign-app-channel.mjs';

console.log(JSON.stringify(signAppChannel(process.argv.slice(2), { renewalOnly:true }),null,2));
