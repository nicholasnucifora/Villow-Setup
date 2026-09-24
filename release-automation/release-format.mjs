import { createHash } from 'node:crypto';
export const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
export const configuration = Object.fromEntries([
  ['VITE_YOUTUBE_CLIENT_ID', 'public'], ['YOUTUBE_CLIENT_SECRET', 'secret'],
  ['VITE_SUPABASE_URL', 'public'], ['VITE_SUPABASE_ANON_KEY', 'public'],
  ['SUPABASE_SERVICE_KEY', 'secret'], ['VITE_APP_URL', 'public'],
  ['ENCRYPTION_KEY', 'secret'], ['CRON_SECRET', 'secret'],
  ['VILLOW_EXPECTED_OWNER_EMAIL', 'server'], ['VILLOW_BOOTSTRAP_TOKEN_HASH', 'server'], ['VILLOW_INSTALLATION_ID', 'server'],
]);
export const googleScopes = ['https://www.googleapis.com/auth/youtube.force-ssl', 'https://www.googleapis.com/auth/userinfo.email', 'https://www.googleapis.com/auth/userinfo.profile'];
export function deployPath(path) {
  if (path.split('/').some(part => !part || part === '..' || part.startsWith('.env') || ['node_modules', 'villow-setup', '.git', '.vercel'].includes(part))) return false;
  if (/\.(test|spec)\.[cm]?[jt]sx?$/.test(path) || /(^|\/)(__tests__|fixtures|tests)\//.test(path)) return false;
  if (['package.json','package-lock.json','index.html','tsconfig.json','tsconfig.node.json','tsconfig.server.json','supabase/setup/history-map.json','vite.config.ts','tailwind.config.js','postcss.config.js','vercel.json','.vercelignore'].includes(path)) return true;
  return /^(api|lib|src|public|images)\//.test(path) && /\.(ts|tsx|js|mjs|json|css|svg|png|jpe?g|webp|gif|ico|woff2?|ttf|webmanifest|txt)$/.test(path);
}
function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) { crc ^= byte; for (let i=0;i<8;i++) crc = (crc >>> 1) ^ (0xedb88320 & -(crc & 1)); }
  return (crc ^ 0xffffffff) >>> 0;
}
/** Deterministic ZIP with stored regular files, fixed timestamp, no directory
 * records, links, executable bits, extra fields, comments or ZIP64 ambiguity. */
export function sourceZip(entries) {
  const local = [], central = []; let offset = 0;
  for (const [path, bytes] of [...entries].sort(([a],[b]) => a < b ? -1 : a > b ? 1 : 0)) {
    if (!/^[A-Za-z0-9_./ -]+$/.test(path) || path.startsWith('/') || path.split('/').includes('..') || bytes.length > 32*1024*1024) throw new Error('Unsafe archive entry');
    const name = Buffer.from(path), crc = crc32(bytes);
    const header = Buffer.alloc(30); header.writeUInt32LE(0x04034b50); header.writeUInt16LE(20,4); header.writeUInt16LE(33,12);
    header.writeUInt32LE(crc,14); header.writeUInt32LE(bytes.length,18); header.writeUInt32LE(bytes.length,22); header.writeUInt16LE(name.length,26);
    local.push(header,name,bytes);
    const record = Buffer.alloc(46); record.writeUInt32LE(0x02014b50); record.writeUInt16LE(0x0314,4); record.writeUInt16LE(20,6);
    record.writeUInt16LE(33,14); record.writeUInt32LE(crc,16); record.writeUInt32LE(bytes.length,20); record.writeUInt32LE(bytes.length,24);
    record.writeUInt16LE(name.length,28); record.writeUInt32LE((0o100644 << 16) >>> 0,38); record.writeUInt32LE(offset,42);
    central.push(record,name); offset += header.length + name.length + bytes.length;
  }
  const directory = Buffer.concat(central), end = Buffer.alloc(22); end.writeUInt32LE(0x06054b50);
  end.writeUInt16LE(entries.size,8); end.writeUInt16LE(entries.size,10); end.writeUInt32LE(directory.length,12); end.writeUInt32LE(offset,16);
  const archive = Buffer.concat([...local,directory,end]);
  if (archive.length > 128*1024*1024 || entries.size > 15000) throw new Error('Archive exceeds manager limits');
  return archive;
}
