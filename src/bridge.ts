import { invoke, isTauri } from "@tauri-apps/api/core";
import type { Bridge, Snapshot } from "./types";

export const nativeBridge: Bridge = {
  demo: false,
  async call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    if (!isTauri()) {
      if (command === "snapshot")
        return {
          manager_version: "0.1.0",
          trust_configured: false,
          installation: null,
          release: null,
          release_checked_at: null,
          message:
            "Browser preview. Real setup runs in the installed desktop application. Use the separate demo to explore.",
        } satisfies Snapshot as T;
      throw new Error(
        "Open the installed desktop application to use this action.",
      );
    }
    return invoke<T>(command, args);
  },
};
