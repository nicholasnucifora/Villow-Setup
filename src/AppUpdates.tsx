import { useEffect, useRef, useState } from "react";
import type { Installation, Snapshot } from "./types";

export function AppUpdates({
  s,
  result,
  busy,
  action,
}: {
  s: Installation;
  result: Snapshot["app_update"];
  busy: boolean;
  action: (command: string, args?: Record<string, unknown>) => Promise<boolean>;
}) {
  const [checking, setChecking] = useState(false);
  const [failed, setFailed] = useState(false);
  const [running, setRunning] = useState(false);
  const until = useRef(0);
  const actionRef = useRef(action);
  actionRef.current = action;
  const intent = s.app_update;
  const pending = intent && intent.phase !== "complete" ? intent : null;
  const terminal = ["failed", "canceled", "address_failed"].includes(
    pending?.deployment_status ?? "",
  );
  useEffect(() => {
    if (!running || busy) return;
    if (
      intent?.phase === "complete" ||
      terminal ||
      Date.now() >= until.current
    ) {
      setRunning(false);
      return;
    }
    if (!pending) return;
    const timer = setTimeout(
      async () => {
        if (!(await actionRef.current("update_app", { digest: pending.to })))
          setRunning(false);
      },
      pending.phase === "verify" ? 10_000 : 100,
    );
    return () => clearTimeout(timer);
  }, [running, busy, intent, pending, terminal]);
  const update = async (digest: string) => {
    if (busy || running) return;
    until.current = Date.now() + 10 * 60_000;
    setRunning(true);
    if (!(await action("update_app", { digest }))) setRunning(false);
  };
  if (s.step !== "complete" || s.read_only) return null;
  return (
    <section className="repair-panel" aria-label="Villow updates">
      <h2>Keep Villow up to date</h2>
      <p>
        Installed app: <strong>Villow {s.app_version}</strong>
      </p>
      {intent?.phase === "complete" && (
        <p role="status" className="success">
          Update verified.{" "}
          {intent.backup?.removed_at
            ? "Setup removed the temporary recovery copy."
            : "The app passed its checks. Setup will retry removing its temporary recovery copy when you reopen this window."}
        </p>
      )}
      {(pending || running) && (
        <div aria-label="Update progress" aria-busy={running || busy}>
          <h3>
            {running || busy ? "Updating your app…" : "Your update is paused"}
          </h3>
          <ol className="update-progress">
            {(
              [
                ["backup", "Save and verify a temporary recovery copy"],
                ["database", "Apply and check database changes"],
                ["upload", "Upload the new app"],
                ["verify", "Wait for Vercel and check your app"],
              ] as const
            ).map(([phase, title], index) => {
              const active = pending
                ? { database: 1, upload: 2, deploy: 2, verify: 3 }[
                    pending.phase as "database" | "upload" | "deploy" | "verify"
                  ]
                : 0;
              return (
                <li
                  key={phase}
                  aria-current={index === active ? "step" : undefined}
                >
                  {index < active ? (
                    "✓ "
                  ) : index === active && (running || busy) ? (
                    <span className="spinner" aria-hidden="true" />
                  ) : (
                    ""
                  )}
                  {title}
                </li>
              );
            })}
          </ol>
          <p role="status">
            {running || busy
              ? "Keep this window open while Setup works. Progress is saved after each step."
              : "Saved progress and the recovery copy are retained. Review any message above, then resume."}
          </p>
          {terminal && (
            <p>
              The Vercel build or address needs attention. Open your Vercel
              project to review it; Setup has kept the recovery copy.
            </p>
          )}
          {pending && (
            <button
              className="primary"
              disabled={busy || running}
              onClick={() => update(pending.to)}
            >
              Resume update
            </button>
          )}
        </div>
      )}
      {!pending && !running && (
        <>
          <p>
            Check for a newer approved release. Checking does not change your
            website or database.
          </p>
          <button
            className="secondary"
            disabled={busy || checking}
            onClick={async () => {
              setChecking(true);
              setFailed(false);
              try {
                setFailed(!(await action("check_app_update")));
              } finally {
                setChecking(false);
              }
            }}
          >
            {checking ? "Checking for updates…" : "Check for updates"}
          </button>
          {checking && (
            <p role="status">
              <span className="spinner" aria-hidden="true" /> Checking the
              signed release list…
            </p>
          )}
          {!checking && failed && (
            <p role="status">
              The update check could not finish. Your app is unchanged. Check
              the message above and try again.
            </p>
          )}
          {!checking && !failed && result && (
            <div role="status">
              <p>
                <strong>{result.message}</strong>
              </p>
              {result.status === "manager_required" && (
                <p>
                  This release needs Villow Setup {result.minimum_manager} or
                  newer. Install the newer Setup EXE on this PC to keep your
                  saved accounts and progress.
                </p>
              )}
              {result.status === "unsupported" && (
                <p>
                  The release does not provide a supported update from your
                  installed version. Keep using your current app; do not run
                  fresh database setup on it.
                </p>
              )}
              {result.notes && <p className="plain-notes">{result.notes}</p>}
              {result.status === "available" && result.digest && (
                <>
                  <p>
                    Setup will protect your data with a verified temporary
                    recovery copy, update this same app and check that it works.
                    You do not need a backup password or the developer’s signing
                    passphrase.
                  </p>
                  {result.downtime && <p>{result.downtime}</p>}
                  <p>
                    The copy stays if a step fails and is removed after the
                    update passes. Failed database changes roll back; later
                    problems keep a resume path and the saved copy for assisted
                    recovery.
                  </p>
                  <button
                    className="primary"
                    disabled={busy}
                    onClick={() => update(result.digest!)}
                  >
                    Update my app
                  </button>
                </>
              )}
            </div>
          )}
        </>
      )}
    </section>
  );
}
