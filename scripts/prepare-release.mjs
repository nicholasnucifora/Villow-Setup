import { writeFileSync, mkdirSync } from "node:fs";
import { resolve } from "node:path";
import { releaseGate } from "./release-gate.mjs";
const { root, trust } = releaseGate();
const thumbprint = process.env.VILLOW_CERT_SHA1,
  identifier = process.env.VILLOW_PRODUCTION_IDENTIFIER,
  timestamp = process.env.VILLOW_TIMESTAMP_URL;
if (!/^[a-fA-F0-9]{40}$/.test(thumbprint ?? ""))
  throw new Error("Supply the verified signing certificate thumbprint.");
if (!/^[a-z][a-z0-9.]+$/.test(identifier ?? "") || identifier.endsWith(".dev"))
  throw new Error("Supply a verified production application identifier.");
const u = new URL(timestamp);
if (!["http:", "https:"].includes(u.protocol) || u.username || u.password)
  throw new Error("Supply the signing issuer’s timestamp endpoint.");
mkdirSync(resolve(root, "artifacts"), { recursive: true });
writeFileSync(
  resolve(root, "artifacts/tauri.release.conf.json"),
  JSON.stringify(
    {
      identifier,
      app: {
        windows: [
          {
            label: "main",
            title: "Villow Setup",
            width: 1120,
            height: 790,
            minWidth: 760,
            minHeight: 620,
            devtools: false,
          },
        ],
      },
      bundle: {
        publisher: trust.publisher,
        windows: {
          certificateThumbprint: thumbprint,
          digestAlgorithm: "sha256",
          timestampUrl: timestamp,
        },
      },
    },
    null,
    2,
  ),
);
console.log(
  "Prepared production Tauri override. No private signing material was written.",
);
