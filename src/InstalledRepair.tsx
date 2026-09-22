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
  if (pending?.phase === "complete") {
    if (!pending.backup?.managed) return null;
    return (
      <p role="status">
        {pending.backup.removed_at
          ? "Repair verified. Setup removed the temporary recovery copy automatically."
          : "Your repaired app passed its checks. The temporary recovery copy is still saved; Setup will retry removing it when you reopen this window."}
      </p>
    );
  }
  if (s.step === "complete") return null;
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
            <h3>Setup protects your data automatically</h3>
            <p>
              Before changing anything, Setup saves and checks an encrypted
              temporary copy of your Villow data and account connections on this
              PC. There is no password to create or file to manage.
            </p>
            <p>
              If the repair fails or you close Setup, the copy stays available
              for recovery. Setup removes it automatically once the repaired
              website passes its database and sign-in checks.
            </p>
            <p>
              A failed database change rolls back automatically. If something
              goes wrong after that, Setup keeps your progress and recovery copy
              for a guided repair. This copy protects the data captured before
              repair; it does not include later activity or replace a separate
              backup against losing this PC.
            </p>
          </div>
          <button
            className="primary"
            disabled={busy || running}
            onClick={async () => {
              until.current = Date.now() + 10 * 60_000;
              const ok = await action("backup_and_repair", {
                digest: offer.digest,
              });
              setRunning(ok);
            }}
          >
            Repair my app →
          </button>
        </>
      )}
      {pending && (
        <>
          {pending.backup ? (
            <div className="success" role="status">
              <strong>Your recovery copy was saved and verified.</strong>
              {pending.backup.managed ? (
                <p>
                  Setup keeps this temporary copy until your repaired app passes
                  its checks. You do not need to manage it.
                </p>
              ) : (
                <>
                  <p>{pending.backup.path}</p>
                  <p>
                    This backup was saved in an earlier Alpha. Keep that file
                    and its password; Setup will not delete it.
                  </p>
                </>
              )}
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
