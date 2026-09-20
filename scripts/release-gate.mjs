import { readFileSync } from "node:fs";
import { resolve } from "node:path";
export function releaseGate() {
  const root = resolve(import.meta.dirname, ".."),
    trust = JSON.parse(
      readFileSync(resolve(root, "src-tauri/trust.json"), "utf8"),
    ),
    q = JSON.parse(
      readFileSync(resolve(root, "docs/qualification.json"), "utf8"),
    );
  const errors = [];
  if (
    !trust.repository ||
    !trust.manifest_url ||
    !trust.publisher ||
    Object.keys(trust.public_keys).length === 0
  )
    errors.push(
      "Authenticated production release source/publisher are not configured.",
    );
  for (const [key, value] of Object.entries(q))
    if (key !== "format" && (value === null || value === false || value === ""))
      errors.push(`Missing release qualification: ${key}`);
  if (errors.length) throw new Error(errors.join("\n"));
  return { root, trust };
}
if (process.argv[1] === import.meta.filename) {
  try {
    releaseGate();
    console.log(
      "Recorded production qualifications are complete; verify their underlying evidence and Windows signatures before publishing.",
    );
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}
