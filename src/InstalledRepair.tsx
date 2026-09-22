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
  const [password, setPassword] = useState("");
  const [repeat, setRepeat] = useState("");
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
    setPassword("");
    setRepeat("");
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
        });
        if (!ok) setRunning(false);
      },
      pending.phase === "verify" ? 10_000 : 50,
    );
    return () => clearTimeout(timer);
  }, [running, pending, busy, terminalBuild]);
  const resume = async (digest: string) => {
    if (busy) return;
    until.current = Date.now() + 10 * 60_000;
    const ok = await action("apply_installed_repair", {
      digest,
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
            <h3>First, save a copy of your Villow data</h3>
            <p>
              Setup will back up your settings, watch history, saved items and
              account connections before changing the database. It includes the
              existing app encryption key and saved setup credentials
              automatically. You do not need to find a key in Vercel or use
              Supabase backup tools.
            </p>
            <p>
              Choose where to save the encrypted backup file, such as a folder
              you already back up or a removable drive. Keep it until you have
              checked the repaired app. It contains your data at the time of the
              backup; later activity is not included.
            </p>
            <h3>Create a password for this backup file</h3>
            <p>
              Save this password in your password manager under “Villow data
              backup”. You would need it to recover the file on another
              computer. This is separate from Google, your database password and
              the developer’s release-signing passphrase.
            </p>
            <label>
              Backup password (at least 12 characters)
              <input
                type="password"
                autoComplete="new-password"
                maxLength={1024}
                value={password}
                disabled={busy}
                onChange={(e) => setPassword(e.target.value)}
              />
            </label>
            <label>
              Repeat backup password
              <input
                type="password"
                autoComplete="new-password"
                maxLength={1024}
                value={repeat}
                disabled={busy}
                onChange={(e) => setRepeat(e.target.value)}
              />
            </label>
            {repeat && repeat !== password && (
              <p role="status">The passwords do not match yet.</p>
            )}
            <p>
              Setup saves, reopens and checks the complete file before the
              repair begins. Canceling the save dialog leaves the repair
              unstarted. This Alpha creates the backup for you; restoring it
              still requires guided recovery into an empty database. It cannot
              overwrite your current app or undo changes made on Google or
              YouTube.
            </p>
          </div>
          <button
            className="primary"
            disabled={
              busy || [...password.trim()].length < 12 || password !== repeat
            }
            onClick={async () => {
              const unlock = password;
              setPassword("");
              setRepeat("");
              until.current = Date.now() + 10 * 60_000;
              const ok = await action("backup_and_repair", {
                digest: offer.digest,
                password: unlock,
              });
              setRunning(ok);
            }}
          >
            Back up and repair my app →
          </button>
        </>
      )}
      {pending && (
        <>
          {pending.backup ? (
            <div className="success" role="status">
              <strong>
                Your encrypted data backup was saved and verified.
              </strong>
              <p>{pending.backup.path}</p>
              <p>
                Captured {new Date(pending.backup.captured_at).toLocaleString()}
                . Keep this file and its password.
              </p>
            </div>
          ) : (
            <p>
              This repair was started in an earlier Alpha with your manual
              backup confirmation. Setup has not verified that backup.
            </p>
          )}
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
                onClick={() => resume(pending.to)}
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
            Your saved repair keeps its original release and backup record.
            Reopening Setup resumes the same signed correction. Keep your
            existing database and credentials.
          </p>
        </>
      )}
    </section>
  );
}
