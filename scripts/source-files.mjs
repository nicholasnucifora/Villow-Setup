import { readdirSync } from "node:fs";
import { resolve, relative } from "node:path";
export const rootEntries = [
  "package.json",
  "package-lock.json",
  "tsconfig.json",
  "vite.config.ts",
  "vitest.config.ts",
  "index.html",
  "README.md",
  "LICENSE",
  "AGENTS.md",
  ".gitignore",
  ".npmrc",
  ".prettierignore",
  ".github",
  "assets",
  "docs",
  "scripts",
  "src",
  "tests",
  "src-tauri",
];
const excluded = new Set([
  "target",
  "gen",
  "node_modules",
  ".tools",
  ".cache",
  "artifacts",
  ".DS_Store",
]);
export function sourceFiles(root) {
  const walk = (dir) =>
    readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
      if (
        excluded.has(e.name) ||
        e.name.startsWith(".env") ||
        /\.(pfx|p12|pem|key|log)$/.test(e.name)
      )
        return [];
      if (e.isSymbolicLink())
        throw new Error("Source transfer refuses symlinks");
      return e.isDirectory()
        ? walk(resolve(dir, e.name))
        : [relative(root, resolve(dir, e.name))];
    });
  return readdirSync(root, { withFileTypes: true })
    .filter((e) => rootEntries.includes(e.name))
    .flatMap((e) => (e.isDirectory() ? walk(resolve(root, e.name)) : [e.name]));
}
