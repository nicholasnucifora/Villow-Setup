import { useEffect, useRef, useState } from "react";
import type { Installation, Snapshot } from "./types";

type Action = (
  command: string,
  args?: Record<string, unknown>,
) => Promise<boolean>;
export function InstalledRepair({
  s,
  offer,
  message,
  busy,
  action,
  open,
}: {
  s: Installation;
  offer: Snapshot["installed_repair"];
  message: string;
  busy: boolean;
  action: Action;
  open: (step: string) => Promise<void>;
}) {
  const [backup, setBackup] = useState(false);
  const [checked, setChecked] = useState(false);
  const [running, setRunning] = useState(false);
  const until = useRef(0);
  const actionRef = useRef(action);
  actionRef.current = action;
  const pending = s.installed_repair;
  const terminalBuild = ["failed", "canceled", "address_failed"].includes(
    pending?.deployment_status ?? "",
  );
  useEffect(() => {
    setBackup(false);
  }, [offer?.digest]);
  useEffect(() => {
    if (!running || !pending || pending.phase === "complete" || busy) return;
    if (terminalBuild || Date.now() >= until.current) {
      setRunning(false);
      return;
    }
    const timer = setTimeout(
      async () => {
        const ok = await actionRef.current("apply_installed_repair", {
          digest: pending.to,
          backupConfirmed: false,
        });
        if (!ok) setRunning(false);
      },
      pending.phase === "verify" ? 10_000 : 50,
    );
    return () => clearTimeout(timer);
  }, [running, pending, busy, terminalBuild]);
  const resume = async (digest: string, confirmed: boolean) => {
    if (busy) return;
    until.current = Date.now() + 10 * 60_000;
    const ok = await action("apply_installed_repair", {
      digest,
      backupConfirmed: confirmed,
    });
    setRunning(ok);
  };
  if (pending?.phase === "complete") return null;
  return (
    <section className="repair-panel" aria-label="Repair your installed app">
      <h2>
        {pending
          ? "Repairing your installed app"
          : "If your app cannot finish setup"}
      </h2>
      <p>
        A signed app correction can repair this unfinished Alpha installation
        while keeping your accounts, database, website address and saved
        credentials.
      </p>
      {!pending && !offer && (
        <>
          <button
            className="secondary"
            disabled={busy}
            onClick={async () => {
              if (await action("check_installed_repair")) setChecked(true);
            }}
          >
            Check for an app repair
          </button>
          {checked && <p role="status">{message}</p>}
        </>
      )}
      {!pending && offer && (
        <>
          <p>
            <strong>Verified app correction: Villow {offer.app_version}</strong>
          </p>
          <p>
            Setup checked that this repair matches your installed database and
            owner. It will update the database, upload the corrected app to your
            existing Vercel project, then check the website.
          </p>
          <div className="alert">
            <strong>Save a backup before applying the repair</strong>
            <p>
              Save a database backup using Supabase’s backup or export tools and
              keep your existing Vercel ENCRYPTION_KEY in your password manager.
              Check how you would restore both. Setup’s recovery file does not
              include your database or encryption key.
            </p>
            <p>
              Setup relies on your confirmation below; it cannot independently
              check your backup. If you cannot make a backup, pause here and ask
              for help.
            </p>
            <button
              className="secondary"
              disabled={busy}
              onClick={() => open("supabase_dashboard")}
            >
              Open Supabase for backup ↗
            </button>
          </div>
          <label className="checkbox">
            <input
              type="checkbox"
              checked={backup}
              disabled={busy}
              onChange={(e) => setBackup(e.target.checked)}
            />
            <span>
              I have saved a database backup and the original encryption key,
              and know how to restore them.
            </span>
          </label>
          <button
            className="primary"
            disabled={busy || !backup}
            onClick={() => resume(offer.digest, backup)}
          >
            Apply repair and rebuild my app →
          </button>
        </>
      )}
      {pending && (
        <>
          <ol className="repair-progress" aria-label="App repair progress">
            <li
              aria-current={pending.phase === "database" ? "step" : undefined}
            >
              1.{" "}
              {pending.phase === "database"
                ? "Apply and verify the database correction"
                : "Database correction verified"}
            </li>
            <li
              aria-current={
                ["upload", "deploy"].includes(pending.phase)
                  ? "step"
                  : undefined
              }
            >
              2. Upload the corrected app to your existing Vercel project
            </li>
            <li aria-current={pending.phase === "verify" ? "step" : undefined}>
              3. Wait for the build and website address, then verify your
              installation
            </li>
          </ol>
          {running && !terminalBuild ? (
            <p role="status">
              <span className="spinner" aria-hidden="true" />{" "}
              {pending.phase === "verify"
                ? "Waiting for Vercel and checking your website…"
                : "Applying the saved repair…"}
            </p>
          ) : (
            <>
              <p>
                {terminalBuild
                  ? "Vercel reported a build or address problem. Open your project → Deployments → the latest deployment → Build Logs and report the error. Your repaired database and saved progress are retained."
                  : "Repair progress is saved. Resume to continue from the last confirmed operation."}
              </p>
              <button
                className="primary"
                disabled={busy}
                onClick={() => resume(pending.to, false)}
              >
                Resume repair
              </button>
            </>
          )}
          {terminalBuild && (
            <button
              className="secondary"
              disabled={busy}
              onClick={() => open("vercel_dashboard")}
            >
              Open Vercel build logs ↗
            </button>
          )}
          {pending.phase === "verify" &&
            pending.deployment_status === "ready" && (
              <button
                className="secondary"
                disabled={busy}
                onClick={() => open("app")}
              >
                Open my app to finish sign-in ↗
              </button>
            )}
          <p className="quiet">
            Your backup confirmation is saved with this repair. Reopening Setup
            resumes the same signed correction. Keep your existing database and
            credentials.
          </p>
        </>
      )}
    </section>
  );
}
