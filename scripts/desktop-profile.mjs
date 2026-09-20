// Build profiles select packaging/UI only. Native release verification is unchanged.
export const distributionRepository = "nicholasnucifora/Villow-Setup";
export const channelUrl = `https://github.com/${distributionRepository}/releases/download/villow-channel/channel.json`;

export function validateAlphaTrust(trust) {
  const errors = [];
  if (
    trust?.format !== 1 ||
    trust.repository !== distributionRepository ||
    trust.channel !== "stable" ||
    trust.manifest_url !== channelUrl
  )
    errors.push(
      "Configure the agreed format-1 Villow release repository and stable channel URL.",
    );
  if (typeof trust?.publisher !== "string" || !trust.publisher.trim())
    errors.push(
      "Supply the maintainer-approved publisher label (not a Windows certificate).",
    );
  if (
    !Number.isSafeInteger(trust?.minimum_sequence) ||
    trust.minimum_sequence < 1
  )
    errors.push("Supply a positive minimum channel sequence.");
  const keys = trust?.public_keys;
  if (
    !keys ||
    typeof keys !== "object" ||
    Array.isArray(keys) ||
    !Object.keys(keys).length
  ) {
    errors.push(
      "Supply the genuine release public key and key ID from the web-release maintainer.",
    );
  } else {
    for (const [id, value] of Object.entries(keys)) {
      if (
        !/^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/.test(id) ||
        /test|demo|fixture|synthetic/i.test(id)
      )
        errors.push(
          "Use a reviewed release key ID, not a fixture/demo key ID.",
        );
      const bytes =
        typeof value === "string"
          ? Buffer.from(value, "base64")
          : Buffer.alloc(0);
      if (
        bytes.length !== 32 ||
        bytes.toString("base64") !== value ||
        bytes.every((byte) => byte === 0)
      )
        errors.push(
          "Release public keys must be canonical base64 of 32 raw Ed25519 public-key bytes.",
        );
    }
  }
  if (errors.length)
    throw new Error(
      `Unsigned alpha is not configured:\n${errors.join("\n")}\nNo account tokens or Windows signing certificate are needed to resolve this step.`,
    );
  // Syntax validation cannot establish key ownership: review genuine public
  // material out of band. Never promote a fixture key into checked-in trust.
}

export function desktopProfile(args, environment, trust) {
  const testing = args.includes("--testing-tools");
  const alpha = args.includes("--unsigned-alpha");
  if (testing && alpha)
    throw new Error("Unsigned alpha cannot include testing tools.");
  if (
    (testing || alpha) &&
    (environment.TAURI_CONFIG ||
      args.some(
        (arg) =>
          arg === "--config" ||
          arg.startsWith("--config=") ||
          arg.startsWith("-c"),
      ))
  )
    throw new Error(
      "Testing and alpha profiles cannot be combined with a custom Tauri configuration.",
    );
  const cliArgs = args.filter(
    (arg) => arg !== "--testing-tools" && arg !== "--unsigned-alpha",
  );
  if (alpha) {
    if (cliArgs.join(" ") !== "build --bundles nsis")
      throw new Error(
        "Use npm run desktop:build:alpha without extra build arguments.",
      );
    validateAlphaTrust(trust);
  }
  const config =
    testing || alpha
      ? {
          productName: alpha ? "Villow Setup Alpha" : "Villow Setup Testing",
          identifier: alpha
            ? "app.villow.setup.alpha"
            : "app.villow.setup.dev.testing",
          app: {
            windows: [
              {
                label: "main",
                title: alpha
                  ? "Villow Setup — unsigned alpha"
                  : "Villow Setup — testing build",
                width: 1120,
                height: 790,
                minWidth: 760,
                minHeight: 620,
                devtools: false,
              },
            ],
          },
          ...(alpha
            ? {
                bundle: {
                  publisher: trust.publisher,
                  windows: {
                    certificateThumbprint: null,
                    digestAlgorithm: null,
                    timestampUrl: null,
                    signCommand: null,
                    nsis: { installMode: "currentUser" },
                  },
                },
              }
            : {}),
        }
      : null;
  return {
    args: [...cliArgs, ...(config ? ["--config", JSON.stringify(config)] : [])],
    env: {
      ...environment,
      VILLOW_SETUP_TESTING: testing ? "1" : "0",
      VILLOW_SETUP_ALPHA: alpha ? "1" : "0",
    },
    config,
  };
}
