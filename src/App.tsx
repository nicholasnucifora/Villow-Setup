import {
  Children,
  lazy,
  Suspense,
  cloneElement,
  isValidElement,
  useEffect,
  useId,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import type { FormEvent, ReactNode } from "react";
import { nativeBridge } from "./bridge";
import { AccountGuide, ProviderAccountGuide, providers } from "./AccountGuide";
import { TokenGuide, TokenExpiry } from "./TokenGuide";
import { GuideImage } from "./GuideImage";
import { ReconciliationForm } from "./ReconciliationForm";
import type {
  Accounts,
  Bridge,
  Google,
  Installation,
  Snapshot,
  Step,
} from "./types";

const TestingTools = __TESTING_TOOLS__
  ? lazy(() => import("./TestingTools"))
  : null;

const stages: { key: Step; title: string; detail: string }[] = [
  {
    key: "projects",
    title: "Vercel & Supabase",
    detail: "Account, token, then projects",
  },
  { key: "origin", title: "Your address", detail: "Reserve a permanent home" },
  {
    key: "google",
    title: "Google Cloud",
    detail: "Project and Google sign-in",
  },
  {
    key: "database",
    title: "Prepare the database",
    detail: "Set up your private storage",
  },
  {
    key: "configuration",
    title: "Configure your app",
    detail: "Connect the services securely",
  },
  {
    key: "deployment",
    title: "Build your app",
    detail: "Prepare it on Vercel",
  },
  { key: "health", title: "Make it yours", detail: "Sign in and verify" },
];
const nextLabels: Record<Step, string> = {
  projects: "Create Vercel project",
  origin: "Reserve my address",
  google: "Continue to my database",
  database: "Prepare my database",
  configuration: "Connect my services",
  deployment: "Build my app on Vercel",
  health: "Check my installation",
  complete: "Open my Villow",
};
function Field({
  label,
  children,
  hint,
}: {
  label: string;
  children: ReactNode;
  hint?: string;
}) {
  const id = useId();
  return (
    <div className="field">
      <label htmlFor={id}>{label}</label>
      {Children.map(children, (child) =>
        isValidElement<{ id?: string; "aria-describedby"?: string }>(child) &&
        ["input", "select", "textarea"].includes(String(child.type))
          ? cloneElement(child, {
              id,
              "aria-describedby": hint ? `${id}-hint` : undefined,
            })
          : child,
      )}
      {hint && <small id={`${id}-hint`}>{hint}</small>}
    </div>
  );
}
function CheckBox({
  children,
  checked,
  onChange,
}: {
  children: ReactNode;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="checkbox">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      <span>{children}</span>
    </label>
  );
}
export function App({
  initialBridge = nativeBridge,
}: {
  initialBridge?: Bridge;
}) {
  const [bridge, setBridge] = useState<Bridge>(initialBridge);
  const [data, setData] = useState<Snapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [notice, setNotice] = useState("");
  const [accounts, setAccounts] = useState<Accounts | null>(null);
  const [tools, setTools] = useState(false);
  const [removeConfirm, setRemoveConfirm] = useState(false);
  const [confirmation, setConfirmation] = useState("");
  const [guidePage, setGuidePage] = useState(-1);
  const [guideReady, setGuideReady] = useState([false, false, false]);
  const headingRef = useRef<HTMLHeadingElement>(null);
  const errorRef = useRef<HTMLDivElement>(null);
  const noticeRef = useRef<HTMLDivElement>(null);
  const recoveryRef = useRef<HTMLElement>(null);
  const s = data?.installation;
  const screen = data
    ? `${s?.id ?? "welcome"}:${s?.read_only ? "recovered" : (s?.step ?? `guide-${guidePage}`)}`
    : null;
  useLayoutEffect(() => {
    if (!screen) return;
    headingRef.current?.focus({ preventScroll: true });
    window.scrollTo({ top: 0, left: 0, behavior: "auto" });
  }, [bridge, screen]);
  useLayoutEffect(() => {
    const target = error ? errorRef.current : notice ? noticeRef.current : null;
    if (!target) return;
    target.focus({ preventScroll: true });
    target.scrollIntoView({ block: "start", behavior: "auto" });
  }, [error, notice]);
  useLayoutEffect(() => {
    if (!tools) return;
    recoveryRef.current?.focus({ preventScroll: true });
    recoveryRef.current?.scrollIntoView({ block: "start", behavior: "auto" });
  }, [tools]);
  useEffect(() => {
    let active = true;
    setData(null);
    setError("");
    setAccounts(null);
    setNotice("");
    setTools(false);
    setRemoveConfirm(false);
    setConfirmation("");
    setGuidePage(-1);
    setGuideReady([false, false, false]);
    bridge
      .call<Snapshot>("snapshot")
      .then((v) => {
        if (active) setData(v);
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [bridge]);
  const run = async (fn: () => Promise<void>) => {
    if (busy) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      await fn();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      try {
        setData(await bridge.call<Snapshot>("snapshot"));
      } catch {
        /* Keep the last visible checkpoint. */
      }
    } finally {
      setBusy(false);
    }
  };
  const action: Action = (command, args, onSuccess) =>
    run(async () => {
      setData(await bridge.call<Snapshot>(command, args));
      onSuccess?.();
    });
  const open = (step: string) =>
    run(async () => {
      await bridge.call("open_step", { step });
      if (__TESTING_TOOLS__ && bridge.demo)
        setNotice(
          "Demo: this button opens the official service in your system browser in the desktop app.",
        );
    });
  const advance = () => action("advance");
  const stage =
    s?.step === "complete"
      ? stages.length
      : stages.findIndex((x) => x.key === s?.step);
  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand">
          <span className="brand-mark" aria-hidden="true">
            v
          </span>
          <div>
            villow<span>SETUP</span>
          </div>
        </div>
        <p className="sidebar-intro">
          A little setup.
          <br />A space of your own.
        </p>
        <nav aria-label="Setup progress">
          <ol className="steps">
            <li
              className={!s && guidePage === -1 ? "current" : "past"}
              aria-current={!s && guidePage === -1 ? "step" : undefined}
            >
              <span className="step-number">{s ? "✓" : "1"}</span>
              <div>
                Welcome<small>Understand your setup</small>
              </div>
            </li>
            <li
              className={!s && guidePage >= 0 ? "current" : s ? "past" : ""}
              aria-current={!s && guidePage >= 0 ? "step" : undefined}
            >
              <span className="step-number">{s ? "✓" : "2"}</span>
              <div>
                {data?.trust_configured
                  ? "Choose your release"
                  : "Account guide"}
                <small>
                  {data?.trust_configured
                    ? "Verify the app and choose its owner"
                    : "Vercel · Supabase · Google Cloud"}
                </small>
              </div>
            </li>
            {stages.map((x, i) => (
              <li
                key={x.key}
                className={stage === i ? "current" : stage > i ? "past" : ""}
                aria-current={stage === i ? "step" : undefined}
              >
                <span className="step-number">{stage > i ? "✓" : i + 3}</span>
                <div>
                  {x.title}
                  <small>{x.detail}</small>
                </div>
              </li>
            ))}
          </ol>
        </nav>
        <div className="sidebar-footer">
          <span className="tiny-dot" />
          Runs on your computer
          <small>Setup {data?.manager_version ?? "0.1.0"} · Windows</small>
        </div>
      </aside>
      <div className="main-shell">
        <header className="topbar">
          <span>{s ? s.name : "Your own Villow, at your pace"}</span>
          <div>
            {s && (
              <button
                className="text-button"
                disabled={busy}
                onClick={() => setTools(!tools)}
                aria-expanded={tools}
              >
                Recovery & settings
              </button>
            )}
          </div>
        </header>
        <main>
          {__UNSIGNED_ALPHA__ && (
            <div className="alert" aria-label="Unsigned alpha">
              <strong>Unsigned alpha · Fresh test installations</strong>
              <p>
                This version can create real cloud resources. Use dedicated test
                resources and review the provider costs before continuing.
              </p>
            </div>
          )}
          {TestingTools && (
            <Suspense fallback={null}>
              <TestingTools
                bridge={bridge}
                setBridge={setBridge}
                nativeBridge={nativeBridge}
                busy={busy}
              />
            </Suspense>
          )}
          {error && (
            <div
              className="alert error"
              role="alert"
              ref={errorRef}
              tabIndex={-1}
            >
              <strong>This step needs attention</strong>
              <p>{error}</p>
              <small>
                {s?.step === "projects" && !s.selection
                  ? "Connecting accounts does not create cloud projects. Your saved setup is still here; correct the access and try again."
                  : "Saved resources remain in your account. Closing this window does not remove them."}
              </small>
            </div>
          )}
          {notice && (
            <div className="alert" role="status" ref={noticeRef} tabIndex={-1}>
              {notice}
            </div>
          )}
          {busy && (
            <div role="status" className="working">
              <span className="spinner" />
              Working on this step. Please leave the window open until it
              returns.
            </div>
          )}
          {!data && !error && <p role="status">Opening your saved setup…</p>}
          {data && !s && (
            <>
              <div className="eyebrow">
                {guidePage === -1
                  ? "WELCOME TO VILLOW"
                  : data.trust_configured
                    ? "STEP 2 OF 9 · CHOOSE YOUR RELEASE"
                    : "ACCOUNT GUIDE PREVIEW"}
              </div>
              <h1
                ref={headingRef}
                tabIndex={-1}
                className={guidePage >= 0 ? "step-title" : undefined}
              >
                {guidePage === -1 ? (
                  <>
                    Make room for
                    <br />
                    <em>intentional watching.</em>
                  </>
                ) : guidePage < 3 ? (
                  providers[guidePage]
                ) : data.trust_configured ? (
                  "Choose your release"
                ) : (
                  "Account guide complete"
                )}
              </h1>
              {!data.trust_configured &&
                (guidePage >= 0 && guidePage < 3 ? (
                  <p className="guide-preview-note">
                    Guide preview · Account connection is unavailable in this
                    build.
                  </p>
                ) : (
                  <div className="alert">
                    <strong>
                      Cloud installation is not available in this build
                    </strong>
                    <p>
                      The verified Villow release and publisher are not
                      configured yet. You can read the account guide, but this
                      build cannot connect your accounts or create your app.
                    </p>
                    <p>
                      You do not need to create accounts or tokens just to
                      review the guide. An updated build with a verified release
                      is required before installation can continue.
                    </p>
                  </div>
                ))}
              {guidePage === -1 && (
                <>
                  <p className="lead">
                    Put Villow in accounts you own. We’ll guide you through each
                    service, one at a time.
                  </p>
                  <div className="ownership-grid">
                    <article>
                      <span className="service-icon">V</span>
                      <h2>Vercel</h2>
                      <p>
                        A home for your website. Create an account; Setup
                        creates the project.
                      </p>
                    </article>
                    <article>
                      <span className="service-icon">S</span>
                      <h2>Supabase</h2>
                      <p>
                        A place for your data. Choose an organization; Setup
                        creates the database.
                      </p>
                    </article>
                    <article>
                      <span className="service-icon">G</span>
                      <h2>Google Cloud</h2>
                      <p>
                        Your Google and YouTube connection. You create one
                        project with our guidance.
                      </p>
                    </article>
                  </div>
                  <p className="quiet">
                    You’ll need access to all three services for a real
                    installation. Existing accounts are fine. You do not need
                    GitHub, Git or a terminal. Review provider plans and limits;
                    Setup never chooses a paid upgrade for you.
                  </p>
                  <button
                    className="primary"
                    disabled={busy}
                    onClick={() => setGuidePage(data.trust_configured ? 3 : 0)}
                  >
                    {data.trust_configured
                      ? "Begin setup"
                      : "Read the account guide"}
                    <span>→</span>
                  </button>
                </>
              )}
              {guidePage >= 0 && guidePage < 3 && (
                <AccountGuide
                  previewOnly={!data.trust_configured}
                  page={guidePage}
                  ready={guideReady}
                  busy={busy}
                  open={open}
                  navigate={setGuidePage}
                  confirm={() => {
                    const ready = guideReady.map(
                      (value, index) => index === guidePage || value,
                    );
                    setGuideReady(ready);
                    setGuidePage(
                      guidePage < 2
                        ? guidePage + 1
                        : ready.every(Boolean)
                          ? 3
                          : ready.indexOf(false),
                    );
                  }}
                />
              )}
              {guidePage === 3 && (
                <>
                  <p className="lead">
                    First, verify the official app release and choose your
                    instance name and Google owner email. No cloud projects are
                    created here.
                  </p>
                  <p>
                    Then work through Vercel’s account and token together,
                    followed by Supabase’s organization and token. After Setup
                    creates the projects and reserves your website address,
                    complete Google Cloud’s project and OAuth configuration in
                    one place.
                  </p>
                  {data.trust_configured ? (
                    <ReleaseForm
                      bridge={bridge}
                      data={data}
                      busy={busy}
                      action={action}
                    />
                  ) : (
                    <p className="guide-takeaway">
                      Account connection will become available in a build with a
                      verified release. No tokens are needed in this build.
                    </p>
                  )}
                  <button
                    className="text-button"
                    disabled={busy}
                    onClick={() => setGuidePage(-1)}
                  >
                    Back to welcome
                  </button>
                </>
              )}
              <div className="form-section">
                <button
                  className="text-button"
                  disabled={busy || (__TESTING_TOOLS__ && bridge.demo)}
                  onClick={() => action("import_recovery")}
                >
                  Open a recovery file
                </button>
                <p className="privacy-note">
                  Your management credentials stay on this computer and go
                  directly to your chosen providers. The Villow website never
                  receives them.
                </p>
              </div>
            </>
          )}
          {s && (
            <>
              <div className="eyebrow">
                {s.step === "complete"
                  ? "YOUR SPACE IS READY"
                  : `STEP ${Math.max(0, stage) + 3} OF 9`}
              </div>
              <h1 className="step-title" ref={headingRef} tabIndex={-1}>
                {s.read_only
                  ? "Your recovered instance"
                  : s.step === "complete"
                    ? __TESTING_TOOLS__ && bridge.demo
                      ? "You’ve finished the demo."
                      : "Welcome to your Villow."
                    : stages[stage]?.title}
              </h1>
              <div className="instance-line">
                <span>App {s.app_version}</span>
                <span>Owner: {s.owner_email}</span>
              </div>
              {s.read_only ? (
                <div className="alert">
                  <strong>Recovery information, awaiting verification</strong>
                  <p>
                    This file restores your resource inventory. It does not
                    prove ownership or authorize changes. Automated repair and
                    legacy adoption are planned for a later release.
                  </p>
                </div>
              ) : (
                <>
                  {s.credentials_removed && (
                    <div className="alert">
                      <strong>Saved credentials removed</strong>
                      <p>
                        Your hosted app continues to run. Reconnecting requires
                        your original account access and any existing encryption
                        secret; setup will never replace it automatically.
                      </p>
                    </div>
                  )}
                  {(s.step === "projects" || s.credentials_removed) && (
                    <AccountForm
                      bridge={bridge}
                      s={s}
                      busy={busy}
                      accounts={accounts}
                      setAccounts={setAccounts}
                      refresh={setData}
                      action={action}
                      run={run}
                      open={open}
                    />
                  )}
                  {s.step === "origin" && (
                    <p className="lead">
                      Your permanent address will also be used by Google to
                      return you to Villow after sign-in. Setup verifies that it
                      belongs to your hosting project.
                    </p>
                  )}
                  {s.step === "google" && (
                    <GoogleForm
                      s={s}
                      busy={busy}
                      demo={__TESTING_TOOLS__ && bridge.demo}
                      action={action}
                      open={open}
                    />
                  )}
                  {s.step === "database" && (
                    <DatabaseStep
                      s={s}
                      busy={busy}
                      demo={__TESTING_TOOLS__ && bridge.demo}
                      action={action}
                    />
                  )}
                  {s.step === "configuration" && (
                    <>
                      <p className="lead">Connect the services securely.</p>
                      <p>
                        Setup sends your Google client secret, database service
                        key and the encryption key to your own Vercel project.
                        Public browser settings are kept separate from server
                        secrets.
                      </p>
                      <div className="alert">
                        <strong>One encryption key for this instance</strong>
                        <p>
                          It is generated once and preserved through retries.
                          Losing or replacing it may make encrypted app data
                          unreadable. Your nonsecret recovery file does not
                          include it.
                        </p>
                      </div>
                    </>
                  )}
                  {s.step === "deployment" && (
                    <>
                      <p className="lead">
                        Vercel will build the verified release in your account.
                      </p>
                      <p>
                        This uses the pinned source archive, without linking a
                        GitHub account. The app opens in a protected bootstrap
                        state until the intended owner signs in.
                      </p>
                    </>
                  )}
                  {s.step === "health" && (
                    <>
                      <p className="lead">
                        Sign in as {s.owner_email} to make this instance yours.
                      </p>
                      <p>
                        Open your app in the system browser, complete Google
                        sign-in, then return here. Setup checks the actual
                        owner, schema, configuration and a bounded authenticated
                        app operation.
                      </p>
                      <button
                        className="secondary"
                        disabled={busy}
                        onClick={() => open("app")}
                      >
                        Open my app to sign in ↗
                      </button>
                      <p className="quiet">
                        A successful build alone does not complete setup. Google
                        publishing is your confirmation; provider and app checks
                        are recorded separately.
                      </p>
                    </>
                  )}
                  {s.step === "complete" && (
                    <>
                      <p className="lead">
                        {__TESTING_TOOLS__ && bridge.demo
                          ? "The simulated checks passed. A real installation must pass them against your provider accounts and hosted app."
                          : "Your cloud instance passed its required checks. You can close this app and switch off your computer."}
                      </p>
                      <div className="address">{s.origin}</div>
                      <p>
                        Only the owner needs Villow Setup. Your friends use your
                        hosted web address. Optional Google Tasks and Todoist
                        connections can be set up later in Villow.
                      </p>
                    </>
                  )}
                  {s.step !== "google" &&
                    (s.step !== "projects" || !!s.selection) &&
                    (!s.credentials_removed || s.step === "complete") && (
                      <div className="action-row">
                        <button
                          className="primary"
                          disabled={busy || !s.selection}
                          onClick={
                            s.step === "complete" ? () => open("app") : advance
                          }
                        >
                          {s.step === "projects"
                            ? s.vercel
                              ? "Create Supabase project"
                              : "Create Vercel project"
                            : nextLabels[s.step]}{" "}
                          <span>→</span>
                        </button>
                        {s.step !== "complete" && (
                          <span className="quiet">
                            Progress saves after every operation.
                          </span>
                        )}
                      </div>
                    )}
                </>
              )}
              {!(__TESTING_TOOLS__ && bridge.demo) && !s.read_only && (
                <ReconciliationForm
                  s={s}
                  busy={busy}
                  action={action}
                  open={open}
                />
              )}
              <ResourceSummary s={s} demo={__TESTING_TOOLS__ && bridge.demo} />
              {tools && (
                <section
                  className="recovery-panel"
                  ref={recoveryRef}
                  tabIndex={-1}
                  aria-label="Recovery and settings"
                >
                  <h2>Keep a way back</h2>
                  <p>
                    A recovery file contains resource IDs, your owner email and
                    version details. It contains no passwords or tokens. Review
                    it before sharing.
                  </p>
                  <button
                    className="secondary"
                    disabled={busy}
                    onClick={() =>
                      run(async () => {
                        const saved =
                          await bridge.call<boolean>("export_recovery");
                        setNotice(
                          saved
                            ? __TESTING_TOOLS__ && bridge.demo
                              ? "Demo: recovery export simulated. Real exports use a native save dialog."
                              : "Recovery information saved without credentials."
                            : "Export cancelled.",
                        );
                      })
                    }
                  >
                    Save recovery information
                  </button>
                  {__TESTING_TOOLS__ && (
                    <details>
                      <summary>Review diagnostic information</summary>
                      <p>
                        Only the nonsecret checkpoint below is available. No
                        provider response bodies or viewing data are collected.
                      </p>
                      <pre>
                        {JSON.stringify(
                          {
                            id: s.id,
                            step: s.step,
                            effects: s.effects,
                            checks: s.checks,
                          },
                          null,
                          2,
                        )}
                      </pre>
                    </details>
                  )}
                  {!s.read_only && (
                    <details>
                      <summary>Reconnect expired provider access</summary>
                      <p>
                        Replace only your management tokens. Saved app secrets,
                        encryption key and resource IDs are preserved. Use the
                        same provider identities you selected originally. Your
                        hosted website does not use these management tokens and
                        keeps running when they expire. Leave a token field
                        blank to keep its saved value.
                      </p>
                      <AccountForm
                        reconnect
                        bridge={bridge}
                        s={s}
                        busy={busy}
                        accounts={accounts}
                        setAccounts={setAccounts}
                        refresh={setData}
                        action={action}
                        run={run}
                        open={open}
                      />
                    </details>
                  )}
                  <h3>Remove saved credentials</h3>
                  <p>
                    This removes local management tokens and setup secrets. It
                    does not revoke provider tokens or remove the copies needed
                    by your hosted app. Revoke management tokens in the provider
                    dashboards separately.
                  </p>
                  <CheckBox checked={removeConfirm} onChange={setRemoveConfirm}>
                    I understand this removes this computer’s saved access,
                    including its encryption-key copy.
                  </CheckBox>
                  <button
                    className="secondary"
                    disabled={busy || !removeConfirm || s.credentials_removed}
                    onClick={() =>
                      action("remove_credentials", undefined, () => {
                        setRemoveConfirm(false);
                        setNotice(
                          __TESTING_TOOLS__ && bridge.demo
                            ? "Demo: saved access removed. Your simulated instance is still available."
                            : "Saved credentials removed from this computer. Your cloud app and provider billing continue.",
                        );
                      })
                    }
                  >
                    Remove local credentials
                  </button>
                  <h3>Forget this instance</h3>
                  <p>
                    Remove the local checkpoint and saved credentials. Your
                    cloud app and provider billing continue.
                  </p>
                  <Field label={`Type ${s.name} to forget it`}>
                    <input
                      value={confirmation}
                      onChange={(e) => setConfirmation(e.target.value)}
                    />
                  </Field>
                  <button
                    className="danger-button"
                    disabled={busy || confirmation !== s.name}
                    onClick={() => action("forget_instance", { confirmation })}
                  >
                    Forget this instance locally
                  </button>
                  <h3>Maintenance</h3>
                  <p>
                    Automated upgrades, repair and cloud removal are not
                    available in this version. Use your provider dashboards to
                    inspect resources, bills and backups. Uninstalling Setup
                    does not close those accounts.
                  </p>
                  <div className="link-row">
                    {["vercel", "supabase", "google"].map((p) => (
                      <button
                        key={p}
                        className="text-button"
                        disabled={busy}
                        onClick={() => open(`${p}_dashboard`)}
                      >
                        {p} dashboard ↗
                      </button>
                    ))}
                  </div>
                </section>
              )}
            </>
          )}
        </main>
        <footer className="footer">
          Your accounts. Your instance. Your pace.
          <span>
            Independent software · not an official Google or YouTube product
          </span>
        </footer>
      </div>
    </div>
  );
}

type Action = (
  command: string,
  args?: Record<string, unknown>,
  onSuccess?: () => void,
) => Promise<void>;
function ReleaseForm({
  bridge,
  data,
  busy,
  action,
}: {
  bridge: Bridge;
  data: Snapshot;
  busy: boolean;
  action: Action;
}) {
  const [name, setName] = useState("my-villow"),
    [email, setEmail] = useState(""),
    [consent, setConsent] = useState(false);
  const digest =
    __TESTING_TOOLS__ && bridge.demo
      ? "demo-digest"
      : (data.release_digest ?? "");
  const submit = (e: FormEvent) => {
    e.preventDefault();
    void action("start_installation", { name, email, digest });
  };
  return (
    <section className="form-section">
      <h2>Choose your starting point</h2>
      {!data.release ? (
        <button
          className="primary"
          disabled={busy}
          onClick={() => action("check_release")}
        >
          Check the official release
        </button>
      ) : (
        <>
          <div className="release-line">
            <strong>Villow {data.release.app_version}</strong>
            <span>
              {__TESTING_TOOLS__ && bridge.demo
                ? "Simulated release"
                : "Authenticated release"}
            </span>
          </div>
          <details>
            <summary>Release notes & technical details</summary>
            <p className="plain-notes">{data.release.notes}</p>
            <code>{data.release.commit}</code>
            <p>
              Last checked:{" "}
              {data.release_checked_at
                ? new Date(data.release_checked_at).toLocaleString()
                : "Not checked"}
            </p>
          </details>
          <form onSubmit={submit}>
            <div className="form-grid">
              <Field
                label="Give your instance a name"
                hint="Lowercase letters, numbers and hyphens."
              >
                <input
                  required
                  pattern="[a-z0-9][a-z0-9-]{0,31}"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                />
              </Field>
              <Field
                label="Your Google account email"
                hint="This account will become the intended owner."
              >
                <input
                  type="email"
                  autoComplete="email"
                  required
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                />
              </Field>
            </div>
            <CheckBox checked={consent} onChange={setConsent}>
              I’ll use dedicated projects in my own accounts and review provider
              charges and verification requirements.
            </CheckBox>
            <button className="primary" disabled={busy || !consent || !digest}>
              Start my setup <span>→</span>
            </button>
          </form>
        </>
      )}
    </section>
  );
}
function AccountForm({
  bridge,
  s,
  busy,
  accounts,
  setAccounts,
  refresh,
  action,
  run,
  open,
  reconnect = false,
}: {
  bridge: Bridge;
  s: Installation;
  busy: boolean;
  accounts: Accounts | null;
  setAccounts: (v: Accounts) => void;
  refresh: (v: Snapshot) => void;
  action: Action;
  run: (fn: () => Promise<void>) => Promise<void>;
  open: (s: string) => Promise<void>;
  reconnect?: boolean;
}) {
  const demo = __TESTING_TOOLS__ && bridge.demo;
  const replacing = reconnect || s.credentials_removed;
  const [page, setPage] = useState(0);
  const [vercel, setVercel] = useState("");
  const [supabase, setSupabase] = useState("");
  const [savedVercel, setSavedVercel] = useState(false);
  const [savedSupabase, setSavedSupabase] = useState(false);
  const [team, setTeam] = useState("");
  const [org, setOrg] = useState("");
  const [costs, setCosts] = useState(false);
  const [region, setRegion] = useState("ap-southeast-2");
  const [connected, setConnected] = useState(false);
  const pageHeading = useRef<HTMLHeadingElement>(null);
  useLayoutEffect(() => {
    pageHeading.current?.focus({ preventScroll: true });
    pageHeading.current?.scrollIntoView({ block: "start", behavior: "auto" });
  }, [page]);
  const invalidate = () => {
    setConnected(false);
    setCosts(false);
  };
  const navigate = (next: number) => {
    setVercel("");
    setSupabase("");
    setPage(next);
  };
  const saveVercel = () =>
    run(async () => {
      invalidate();
      try {
        refresh(
          await bridge.call<Snapshot>("save_credentials", {
            vercel: vercel.trim(),
            supabase: "",
          }),
        );
        setSavedVercel(true);
        setPage(1);
      } finally {
        setVercel("");
      }
    });
  const connect = (useSaved = false) =>
    run(async () => {
      invalidate();
      try {
        if (demo || (!useSaved && (vercel.trim() || supabase.trim()))) {
          refresh(
            await bridge.call<Snapshot>("save_credentials", {
              vercel: demo ? "demo" : vercel.trim(),
              supabase: demo ? "demo" : supabase.trim(),
            }),
          );
          if (supabase.trim()) setSavedSupabase(true);
        }
        const a = await bridge.call<Accounts>("discover_accounts");
        setAccounts(a);
        setTeam(a.vercel[0]?.id ?? "");
        setOrg(a.supabase[0]?.id ?? "");
        setConnected(true);
        if (!replacing) setPage(2);
      } finally {
        setVercel("");
        setSupabase("");
      }
    });
  const tokenField = (provider: "vercel" | "supabase") => (
    <Field
      label={
        provider === "vercel"
          ? "Vercel access token"
          : "Supabase management token"
      }
      hint={
        replacing
          ? "Leave blank to keep the token already saved on this computer."
          : "Saved in Windows Credential Manager. The field clears after saving."
      }
    >
      <input
        type="password"
        autoComplete="off"
        spellCheck={false}
        disabled={busy}
        value={provider === "vercel" ? vercel : supabase}
        onChange={(e) => {
          (provider === "vercel" ? setVercel : setSupabase)(e.target.value);
          invalidate();
        }}
      />
    </Field>
  );
  if (s.selection && !replacing)
    return (
      <div className="alert">
        <strong>Accounts confirmed</strong>
        <p>
          Each next action creates one dedicated project. Setup checks account
          and resource ownership before continuing. After these projects and
          your website address are ready, you’ll complete Google Cloud in one
          step.
        </p>
        <TokenExpiry />
      </div>
    );
  return (
    <section>
      {replacing ? (
        <>
          <p>
            Replace the expired token for the same account and scope. Leave the
            other field blank to keep it. This reads your accounts without
            creating projects.
          </p>
          {!demo &&
            (["vercel", "supabase"] as const).map((provider) => (
              <section key={provider}>
                {tokenField(provider)}
                <details>
                  <summary>
                    {provider === "vercel" ? "Vercel" : "Supabase"} token
                    instructions
                  </summary>
                  <TokenGuide provider={provider} busy={busy} open={open} />
                </details>
              </section>
            ))}
          <button
            className="secondary"
            disabled={busy || (!demo && !vercel.trim() && !supabase.trim())}
            onClick={() => connect()}
          >
            {demo
              ? "Load demo accounts"
              : "Save replacement tokens & check access"}
          </button>
        </>
      ) : demo ? (
        <button className="secondary" disabled={busy} onClick={() => connect()}>
          Load demo accounts
        </button>
      ) : (
        <>
          <nav aria-label="Connect providers" className="provider-progress">
            {["Vercel", "Supabase", "Confirm accounts"].map((label, index) => (
              <button
                key={label}
                disabled={
                  busy ||
                  (index === 1 && !savedVercel) ||
                  (index === 2 && !connected)
                }
                aria-current={page === index ? "step" : undefined}
                onClick={() => navigate(index)}
              >
                <span aria-hidden="true">{index + 1}</span>
                {label}
              </button>
            ))}
          </nav>
          <h2 ref={pageHeading} tabIndex={-1}>
            {page === 0
              ? "Connect Vercel"
              : page === 1
                ? "Connect Supabase"
                : "Review your accounts"}
          </h2>
          {page < 2 && (
            <>
              <ProviderAccountGuide page={page} busy={busy} open={open} />
              <TokenGuide
                provider={page === 0 ? "vercel" : "supabase"}
                busy={busy}
                open={open}
              />
              {tokenField(page === 0 ? "vercel" : "supabase")}
              <div className="action-row">
                {page === 0 ? (
                  <button
                    className="primary"
                    disabled={busy || !vercel.trim()}
                    onClick={saveVercel}
                  >
                    Save Vercel token & continue →
                  </button>
                ) : (
                  <button
                    className="primary"
                    disabled={busy || !supabase.trim()}
                    onClick={() => connect()}
                  >
                    Save Supabase token & read accounts →
                  </button>
                )}
              </div>
              <p className="quiet">
                {page === 0
                  ? "Saving stores this token only. Setup checks both providers’ access after the Supabase step, before creating anything."
                  : "Reading accounts does not create projects. You’ll choose the accounts and confirm their costs next."}
              </p>
              <details>
                <summary>
                  {page === 0
                    ? "Already saved a Vercel token on this computer?"
                    : "Already saved a Supabase token on this computer?"}
                </summary>
                <p>
                  Use the token from this installation’s saved progress. If it
                  is missing, expired or refused when accounts are read, paste a
                  replacement.
                </p>
                <button
                  className="secondary"
                  disabled={busy}
                  onClick={() => {
                    if (page === 0) {
                      setSavedVercel(true);
                      navigate(1);
                    } else {
                      void connect(true);
                    }
                  }}
                >
                  {page === 0
                    ? "Use my saved Vercel token"
                    : "Read accounts with saved tokens"}
                </button>
              </details>
              {page === 1 && savedSupabase && !connected && (
                <p role="status">
                  Supabase token saved; account checks still need to pass. You
                  can retry with saved tokens.
                </p>
              )}
            </>
          )}
        </>
      )}
      {connected && (
        <p role="status">
          {demo ? "Demo accounts loaded." : "Account access loaded."}
          {s.selection &&
            " Continue setup to check access against your saved accounts. No new projects were created."}
        </p>
      )}
      {accounts &&
        connected &&
        !s.selection &&
        (demo || replacing || page === 2) && (
          <div className="form-section">
            <h2>Choose where Villow will live</h2>
            <p>
              These names come from your providers. Choose the accounts you just
              connected and the region closest to your users.
            </p>
            {(!accounts.vercel.length || !accounts.supabase.length) && (
              <p role="alert">
                No eligible account or organization was returned. Check the
                token permissions and that you have a Supabase organization,
                then reconnect.
              </p>
            )}
            <div className="form-grid">
              <Field label="Vercel account">
                <select value={team} onChange={(e) => setTeam(e.target.value)}>
                  {accounts.vercel.map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.name} · {a.id}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label="Supabase organization">
                <select value={org} onChange={(e) => setOrg(e.target.value)}>
                  {accounts.supabase.map((a) => (
                    <option key={a.id} value={a.id}>
                      {a.name} · {a.id}
                    </option>
                  ))}
                </select>
              </Field>
              <Field label="Database region">
                <select
                  value={region}
                  onChange={(e) => setRegion(e.target.value)}
                >
                  <option value="ap-southeast-2">Sydney</option>
                  <option value="us-east-1">US East</option>
                  <option value="eu-west-1">Ireland</option>
                  <option value="ap-southeast-1">Singapore</option>
                </select>
              </Field>
            </div>
            <CheckBox checked={costs} onChange={setCosts}>
              These are my intended accounts. Creating projects can use my
              plan’s resources and incur charges; I have reviewed my provider
              plan.
            </CheckBox>
            <button
              className="primary"
              disabled={busy || !team || !org || !costs}
              onClick={() =>
                action("select_accounts", {
                  selection: {
                    vercel_user: accounts.vercel_user,
                    supabase_user: accounts.supabase_user,
                    vercel_account: team,
                    supabase_organization: org,
                    supabase_slug: accounts.supabase.find((a) => a.id === org)
                      ?.slug,
                    region,
                    costs_acknowledged: costs,
                  },
                })
              }
            >
              Confirm these accounts
            </button>
          </div>
        )}
    </section>
  );
}

function GoogleForm({
  s,
  busy,
  demo,
  action,
  open,
}: {
  s: Installation;
  busy: boolean;
  demo: boolean;
  action: Action;
  open: (s: string) => Promise<void>;
}) {
  const [project, setProject] = useState(
      s.google?.project_id ?? (demo ? "demo-google-project" : ""),
    ),
    [client, setClient] = useState(
      s.google?.client_id ?? (demo ? "demo.apps.googleusercontent.com" : ""),
    ),
    [secret, setSecret] = useState("");
  const [enabled, setEnabled] = useState(
      s.google?.api_enabled_confirmed ?? false,
    ),
    [published, setPublished] = useState(
      s.google?.consent_published_confirmed ?? false,
    ),
    [audience, setAudience] = useState(
      s.google?.audience ?? "external_production",
    ),
    [secretPending, setSecretPending] = useState(false),
    [copied, setCopied] = useState("");
  const matchesSaved =
    !!s.google &&
    project === s.google.project_id &&
    client === s.google.client_id &&
    enabled === s.google.api_enabled_confirmed &&
    published === s.google.consent_published_confirmed &&
    audience === s.google.audience &&
    !secretPending;
  const copy = async (value: string) => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied("Copied.");
    } catch {
      setCopied("Select and copy the address above.");
    }
  };
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    const google: Google = {
      project_id: project,
      client_id: client,
      api_enabled_confirmed: enabled,
      audience,
      consent_published_confirmed: published,
    };
    try {
      await action(
        "set_google",
        {
          google,
          secret: demo ? "demo-secret" : secret,
        },
        () => setSecretPending(false),
      );
    } finally {
      setSecret("");
    }
  };
  return (
    <section>
      <p className="lead">
        Create or select your Google Cloud project, then configure sign-in and
        YouTube access here. Your website address is now ready.
      </p>
      <ProviderAccountGuide page={2} busy={busy} open={open} />
      <h2>Connect this Google project</h2>
      <ol className="instructions">
        <li>
          <button
            className="text-button"
            disabled={busy}
            onClick={() => open("google_api")}
          >
            Enable YouTube Data API v3 ↗
          </button>
          <p>An API key alone cannot authorize your private subscriptions.</p>
        </li>
        <li>
          <button
            className="text-button"
            disabled={busy}
            onClick={() => open("google_audience")}
          >
            Configure branding, audience and data access ↗
          </button>
          <p>
            Include the YouTube permission and identity scopes listed below. For
            a personal external project, publish the audience when ready.
            Publishing and Google verification are different processes.
          </p>
        </li>
        <li>
          <button
            className="text-button"
            disabled={busy}
            onClick={() => open("google_client")}
          >
            Create a Web application OAuth client ↗
          </button>
          <p>
            Use these exact hosted addresses, then return with the client ID and
            secret.
          </p>
        </li>
      </ol>
      <GuideImage name="google-oauth" />
      <div className="copy-row">
        <div>
          <small>Authorized JavaScript origin</small>
          <code>{s.origin}</code>
        </div>
        <button className="secondary" onClick={() => copy(s.origin ?? "")}>
          Copy origin
        </button>
      </div>
      <div className="copy-row">
        <div>
          <small>Authorized redirect URI</small>
          <code>{s.origin}/api/auth</code>
        </div>
        <button
          className="secondary"
          onClick={() => copy(`${s.origin}/api/auth`)}
        >
          Copy callback
        </button>
      </div>
      {copied && <p role="status">{copied}</p>}
      <details>
        <summary>Required Google permissions</summary>
        <ul>
          {s.google_scopes.map((scope) => (
            <li key={scope}>
              <code>{scope}</code>
            </li>
          ))}
        </ul>
        <p>
          The YouTube permission includes account write access used by Villow.
          Google Tasks and Todoist are optional later. Gemini is not required.
        </p>
      </details>
      <div className="alert">
        <strong>Testing mode expires</strong>
        <p>
          With Villow’s YouTube permission, an external app left in Google’s
          Testing mode typically loses refresh access after seven days.
          Personal-use exceptions may apply to verification, but warning-free
          access is not guaranteed.
        </p>
      </div>
      <form onSubmit={submit}>
        <div className="form-grid">
          <Field
            label="Google Cloud project ID"
            hint="Copy Project ID from Project info in your Google Cloud dashboard. Use the ID, not the project name or number."
          >
            <input
              required
              value={project}
              onChange={(e) => setProject(e.target.value)}
            />
          </Field>
          <Field label="OAuth Web client ID">
            <input
              required
              value={client}
              onChange={(e) => setClient(e.target.value)}
            />
          </Field>
          {!demo && (
            <Field label="OAuth client secret">
              <input
                required
                type="password"
                autoComplete="off"
                value={secret}
                onChange={(e) => {
                  setSecret(e.target.value);
                  setSecretPending(true);
                }}
              />
            </Field>
          )}
          <Field label="Audience">
            <select
              value={audience}
              onChange={(e) => setAudience(e.target.value)}
            >
              <option value="external_production">
                External · In production
              </option>
              <option value="internal">
                Internal · my Workspace organization only
              </option>
              <option value="external_testing">External · still Testing</option>
            </select>
          </Field>
        </div>
        <CheckBox checked={enabled} onChange={setEnabled}>
          I enabled YouTube Data API v3 in this Google project.
        </CheckBox>
        <CheckBox checked={published} onChange={setPublished}>
          I configured the audience, required scopes and exact callback. My
          external app is published, or this is an eligible internal Workspace
          app.
        </CheckBox>
        {s.google && (
          <p role="status">
            {matchesSaved
              ? "Google configuration saved."
              : "Save your Google changes before continuing."}
          </p>
        )}
        <div className="action-row">
          <button
            className={s.google ? "secondary" : "primary"}
            disabled={
              busy ||
              matchesSaved ||
              !enabled ||
              !published ||
              audience === "external_testing"
            }
          >
            Save my Google configuration
          </button>
          {s.google && !s.credentials_removed && (
            <button
              type="button"
              className="primary"
              disabled={busy || !matchesSaved}
              onClick={() => action("advance")}
            >
              Continue to my database →
            </button>
          )}
        </div>
      </form>
    </section>
  );
}
function DatabaseStep({
  s,
  busy,
  demo,
  action,
}: {
  s: Installation;
  busy: boolean;
  demo: boolean;
  action: Action;
}) {
  const [host, setHost] = useState(
      s.db_connection?.host ?? `db.${s.database?.id}.supabase.co`,
    ),
    [user, setUser] = useState(s.db_connection?.user ?? "postgres"),
    [password, setPassword] = useState("");
  return (
    <section>
      <p className="lead">
        Your database project is created. Now add Villow’s tables and access
        rules.
      </p>
      <p>
        Setup already saved the generated database password; you do not need to
        enter it again. Setup checks for existing app data, applies the signed
        migration plan under a database lock, and verifies its postconditions.
        If the schema differs, it stops for review.
      </p>
      {demo ? (
        <p className="quiet">
          The demo uses a fictional database. No connection details or password
          are needed.
        </p>
      ) : (
        <details>
          <summary>Connection settings for IPv4-only networks</summary>
          <p>
            The direct connection uses IPv6. In Supabase’s Connect dialog,
            choose Session pooler (port 5432) if your network needs IPv4. Copy
            only the host and username here. The generated database password is
            already saved.
          </p>
          <div className="form-grid">
            <Field label="Database host">
              <input value={host} onChange={(e) => setHost(e.target.value)} />
            </Field>
            <Field label="Database user">
              <input value={user} onChange={(e) => setUser(e.target.value)} />
            </Field>
            <Field label="Database password, only if changed">
              <input
                type="password"
                autoComplete="off"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
              />
            </Field>
          </div>
          <button
            className="secondary"
            disabled={busy}
            onClick={async () => {
              try {
                await action("set_database_connection", {
                  connection: { host, user },
                  password,
                });
              } finally {
                setPassword("");
              }
            }}
          >
            Save connection settings
          </button>
        </details>
      )}
    </section>
  );
}
function ResourceSummary({ s, demo }: { s: Installation; demo: boolean }) {
  return (
    <section className="resource-summary">
      <h2>{demo ? "Simulated resources" : "Your saved resources"}</h2>
      <dl>
        <div>
          <dt>Hosting project</dt>
          <dd>{s.vercel?.id ?? "Not created"}</dd>
        </div>
        <div>
          <dt>Database project</dt>
          <dd>{s.database?.id ?? "Not created"}</dd>
        </div>
        <div>
          <dt>Production address</dt>
          <dd>{s.origin ?? "Not reserved"}</dd>
        </div>
        <div>
          <dt>Schema</dt>
          <dd>
            {s.schema_revision} ·{" "}
            {s.effects.migrate?.status === "verified"
              ? "verified"
              : "not verified"}
          </dd>
        </div>
      </dl>
      {s.checks.length > 0 && (
        <details>
          <summary>Evidence from completed checks</summary>
          <ul className="check-list">
            {s.checks.map((c, i) => (
              <li key={`${c.title}-${i}`}>
                <span>{c.kind}</span>
                <div>
                  {c.title}
                  <small>{new Date(c.at).toLocaleString()}</small>
                </div>
              </li>
            ))}
          </ul>
        </details>
      )}
    </section>
  );
}
