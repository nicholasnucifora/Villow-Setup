import { useEffect, useRef, useState } from "react";
import type { Installation } from "./types";

export function LaunchProgress({ s }: { s: Installation }) {
  const status = s.deployment_status;
  const later = s.step === "health" || s.step === "complete";
  const configured = s.step !== "configuration";
  const uploaded = !!s.deployment_id;
  const built =
    later || status === "assigning_address" || status === "address_failed";
  const rows = [
    { title: "Connect your services", done: configured },
    { title: "Upload your app to Vercel", done: uploaded },
    { title: "Wait for Vercel to finish building", done: built },
    { title: "Connect your website address", done: later },
    {
      title: "Sign in with Google and check your app",
      done: s.step === "complete",
    },
  ];
  const current = rows.findIndex((row) => !row.done);
  return (
    <section className="launch-progress" aria-label="Remaining setup steps">
      <h2>Your app, step by step</h2>
      <ol>
        {rows.map((row, index) => (
          <li
            key={row.title}
            className={row.done ? "done" : index === current ? "active" : ""}
            aria-current={index === current ? "step" : undefined}
          >
            <span aria-hidden="true">{row.done ? "✓" : index + 1}</span>
            <div>
              {row.title}
              <small>
                {row.done
                  ? "Done"
                  : index === current
                    ? "Current step"
                    : "Up next"}
              </small>
            </div>
          </li>
        ))}
      </ol>
    </section>
  );
}

const labels = {
  queued: "Waiting for Vercel to start",
  building: "Vercel is building your app",
  assigning_address: "Connecting your website address",
  ready: "Your website is ready",
  failed: "Vercel could not build your app",
  canceled: "The Vercel build was canceled",
  address_failed: "Vercel could not connect your website address",
};
export function DeploymentStep({
  s,
  busy,
  action,
  open,
}: {
  s: Installation;
  busy: boolean;
  action: (command: string) => Promise<boolean>;
  open: (step: string) => void;
}) {
  const [watching, setWatching] = useState(true);
  const [pauseReason, setPauseReason] = useState("");
  const statusRef = useRef<HTMLElement>(null);
  const actionRef = useRef(action);
  actionRef.current = action;
  const busyRef = useRef(busy);
  busyRef.current = busy;
  const status = s.deployment_status;
  const stopped =
    status === "failed" || status === "canceled" || status === "address_failed";
  useEffect(() => {
    if (!stopped) return;
    statusRef.current?.focus({ preventScroll: true });
    statusRef.current?.scrollIntoView({ block: "center", behavior: "auto" });
  }, [stopped]);
  useEffect(() => {
    if (!s.deployment_id || stopped || !watching || s.credentials_removed)
      return;
    let canceled = false;
    let timer: ReturnType<typeof setTimeout>;
    const started = Date.now();
    const check = async () => {
      if (busyRef.current) {
        timer = setTimeout(check, 1000);
        return;
      }
      const ok = await actionRef.current("check_deployment");
      if (canceled) return;
      if (!ok || Date.now() - started >= 10 * 60_000) {
        setWatching(false);
        setPauseReason(
          ok
            ? "This is taking longer than usual. Your build is still saved in Vercel."
            : "Automatic checks paused. Resolve the message above, then check again.",
        );
        return;
      }
      timer = setTimeout(check, 10_000);
    };
    // Let an in-flight submission finish before the first read-only poll.
    timer = setTimeout(check, 1000);
    return () => {
      canceled = true;
      clearTimeout(timer);
    };
  }, [s.deployment_id, stopped, watching, s.credentials_removed]);
  if (!s.deployment_id)
    return (
      <>
        <p className="lead">Vercel will build Villow in your account.</p>
        <p>
          Click Build my app on Vercel below. Setup uploads the verified app,
          watches the build, and checks your website address before asking you
          to sign in.
        </p>
      </>
    );
  return (
    <section
      className={`deployment-status ${stopped ? "needs-attention" : ""}`}
      aria-label="Vercel build status"
      aria-live="polite"
      ref={statusRef}
      tabIndex={-1}
    >
      <h2>
        {!stopped && watching && !s.credentials_removed && (
          <span className="spinner" aria-hidden="true" />
        )}
        {status ? labels[status] : "Checking your existing Vercel build"}
      </h2>
      {stopped ? (
        <>
          <p>
            Your website is not ready for sign-in. Open Vercel, choose{" "}
            <b>{s.name}</b>, then{" "}
            <b>Deployments → the latest deployment → Build Logs</b>.
          </p>
          <p>
            {status === "address_failed"
              ? "Report the domain-assignment error shown there."
              : "Report the first build error shown there so the cause can be fixed."}{" "}
            Keep your existing projects and database.
          </p>
        </>
      ) : (
        <p>
          {s.credentials_removed
            ? "Reconnect your provider accounts to check this saved build."
            : watching
              ? "Setup checks automatically every 10 seconds. This can take a few minutes. The sign-in button appears once Vercel finishes and your address is connected."
              : pauseReason}
        </p>
      )}
      <p className="quiet">
        Reserved address: {s.origin}. This is not a live website until these
        checks pass.
      </p>
      <button
        className="secondary"
        disabled={busy}
        onClick={() => open("vercel_dashboard")}
      >
        Open Vercel dashboard ↗
      </button>
      {(stopped || !watching) && (
        <button
          className="primary"
          disabled={busy || s.credentials_removed}
          onClick={async () => {
            if (await action("check_deployment")) {
              setPauseReason("");
              setWatching(true);
            }
          }}
        >
          Check build again
        </button>
      )}
    </section>
  );
}
