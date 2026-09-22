import { useState } from "react";
import type { Installation } from "./types";
export function ReconciliationForm({
  s,
  busy,
  action,
  open,
}: {
  s: Installation;
  busy: boolean;
  action: (c: string, a?: Record<string, unknown>) => Promise<boolean>;
  open: (s: string) => Promise<void>;
}) {
  const [id, setId] = useState(""),
    [confirmation, setConfirmation] = useState("");
  const provider =
    !s.vercel &&
    ["executing", "needs_review"].includes(s.effects.create_vercel?.status)
      ? "vercel"
      : !s.database &&
          ["executing", "needs_review"].includes(
            s.effects.create_database?.status,
          )
        ? "supabase"
        : null;
  if (!provider) return null;
  return (
    <section className="recovery-panel">
      <h2>Find the project that may already exist</h2>
      <p>
        Setup will not repeat this creation. In your {provider} dashboard, find
        the dedicated project named <strong>{s.name}</strong>. Check its account
        and creation time, then copy its stable project ID here.
      </p>
      <p>
        A name match alone is insufficient. Setup reads the selected ID and
        verifies its account, name and creation time. Confirm only a project
        created by this setup attempt.
      </p>
      <button
        className="text-button"
        disabled={busy}
        onClick={() => open(`${provider}_dashboard`)}
      >
        Open {provider} dashboard ↗
      </button>
      <div className="form-grid">
        <label className="field">
          Project ID
          <input value={id} onChange={(e) => setId(e.target.value)} />
        </label>
        <label className="field">
          Type {s.name} to confirm this project
          <input
            value={confirmation}
            onChange={(e) => setConfirmation(e.target.value)}
          />
        </label>
      </div>
      <button
        className="secondary"
        disabled={busy || !id || confirmation !== s.name}
        onClick={() =>
          action("reconcile_created", { provider, id, confirmation })
        }
      >
        Verify and link this created project
      </button>
      <p className="quiet">
        If no matching project exists or the details differ, leave this attempt
        paused and consult the provider. Do not choose another shared project.
      </p>
    </section>
  );
}
