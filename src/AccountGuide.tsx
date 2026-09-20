import { GuideImage } from "./GuideImage";

export const providers = ["Vercel", "Supabase", "Google Cloud"] as const;

export function AccountGuide({
  page,
  ready,
  busy,
  open,
  navigate,
  confirm,
  previewOnly,
}: {
  page: number;
  ready: boolean[];
  busy: boolean;
  open: (step: string) => Promise<void>;
  navigate: (page: number) => void;
  confirm: () => void;
  previewOnly: boolean;
}) {
  return (
    <>
      <nav aria-label="Account preparation" className="provider-progress">
        {providers.map((provider, index) => (
          <button
            key={provider}
            disabled={busy}
            aria-current={page === index ? "step" : undefined}
            onClick={() => navigate(index)}
          >
            <span aria-hidden="true">{ready[index] ? "✓" : index + 1}</span>
            {provider}
            <small>
              {ready[index]
                ? previewOnly
                  ? "Reviewed"
                  : "Ready"
                : "Account setup"}
            </small>
          </button>
        ))}
      </nav>
      {page === 0 && (
        <section>
          <p className="lead">
            Vercel keeps your Villow website online, even when your computer is
            off.
          </p>
          <ol className="instructions">
            <li>
              <strong>
                Create an account, or sign in to one you already own.
              </strong>
              <p>
                Complete Vercel’s email or account verification. Choose a plan
                that suits your intended use and review its limits.
              </p>
              <button
                className="secondary"
                disabled={busy}
                onClick={() => open("vercel_signup")}
              >
                Open Vercel ↗
              </button>
            </li>
            <li>
              <strong>Stop when you reach your dashboard.</strong>
              <p>
                There is no need to import a Git repository, choose a template
                or create a project. Setup will create a dedicated Villow
                project after you connect your account.
              </p>
            </li>
          </ol>
          <GuideImage name="vercel-account" />
          <div className="guide-takeaway">
            <strong>What to keep</strong>
            <p>
              Keep your normal account sign-in details. You do not need to copy
              account IDs or create an access token yet. The Connect your
              accounts step will show you how.
            </p>
          </div>
        </section>
      )}
      {page === 1 && (
        <section>
          <p className="lead">
            Supabase holds your Villow settings, subscriptions and library.
          </p>
          <ol className="instructions">
            <li>
              <strong>Sign up or sign in, then choose an organization.</strong>
              <p>
                An organization is a container for your projects and billing.
                You can use one you already own, or create one with a name you
                recognize. Review the selected plan.
              </p>
              <button
                className="secondary"
                disabled={busy}
                onClick={() => open("supabase_signup")}
              >
                Open Supabase ↗
              </button>
            </li>
            <li>
              <strong>Stop before creating a database project.</strong>
              <p>
                If Supabase opens “Create a new project”, go back to your
                organization. Setup will ask you to choose this organization and
                a region, then create the project for you.
              </p>
            </li>
          </ol>
          <GuideImage name="supabase-organization" />
          <div className="guide-takeaway">
            <strong>What about the database password?</strong>
            <p>
              Setup generates a strong password and saves it in Windows
              Credential Manager when it creates your project. You do not need
              to invent, copy or memorize it for this walkthrough.
            </p>
          </div>
          <details>
            <summary>
              I’m seeing security options or already created a project
            </summary>
            <p>
              Database password, Enable Data API, Automatically expose new
              tables, Enable automatic RLS and Advanced configuration belong to
              the manual project form. You do not need to fill in that form for
              Setup.
            </p>
            <p>
              Later, Prepare the database installs Villow’s tables and access
              rules into the project Setup created. Account registration and
              database preparation are separate steps.
            </p>
            <p>
              If you already made a project manually, keep its password in your
              password manager. This version cannot use an existing project; it
              creates a separate one, which also counts toward your provider
              limits. Review those limits before continuing.
            </p>
          </details>
        </section>
      )}
      {page === 2 && (
        <section>
          <p className="lead">
            Google Cloud lets your own Villow instance request Google sign-in
            and access to YouTube.
          </p>
          <ol className="instructions">
            <li>
              <strong>
                Open Google Cloud with the Google account you intend to use.
              </strong>
              <p>
                Use your existing Google sign-in. Complete any account
                verification or terms shown by Google.
              </p>
              <button
                className="secondary"
                disabled={busy}
                onClick={() => open("google_dashboard")}
              >
                Open Google Cloud ↗
              </button>
            </li>
            <li>
              <strong>
                Create one dedicated project, or select one you already made for
                Villow.
              </strong>
              <p>
                A name such as “My Villow” helps you find it later. Use the
                project selector at the top of the dashboard to check which
                project is open.
              </p>
              <button
                className="text-button"
                disabled={busy}
                onClick={() => open("google_project")}
              >
                Create a Google Cloud project ↗
              </button>
            </li>
          </ol>
          <GuideImage name="google-project" />
          <div className="guide-takeaway">
            <strong>What to keep from Project info</strong>
            <p>
              Bookmark your project dashboard. You will need its{" "}
              <b>Project ID</b> at Connect Google, and can copy it there later.
              Setup does not ask for the project number. The project name is
              just a label to help you find it.
            </p>
          </div>
          <details>
            <summary>What about the other Google Cloud options?</summary>
            <p>
              Gemini, Cloud Assist, API keys and service accounts are not needed
              for this setup. There is no setup action attached to the
              promotional trial banner.
            </p>
            <p>
              At Connect Google, after Setup has reserved your website address,
              you will enable YouTube Data API v3 and create an OAuth Web
              client. That step explains exactly where to paste its client ID
              and secret. You do not need them yet.
            </p>
          </details>
        </section>
      )}
      <div className="action-row guide-actions">
        <button
          className="secondary"
          disabled={busy}
          onClick={() => navigate(page - 1)}
        >
          {page === 0 ? "Back to welcome" : `Back to ${providers[page - 1]}`}
        </button>
        <button className="primary" disabled={busy} onClick={confirm}>
          {previewOnly
            ? page < 2
              ? `Next: ${providers[page + 1]}`
              : "Finish reading the guide"
            : `${providers[page]} is ready`}{" "}
          <span>→</span>
        </button>
      </div>
      <p className="quiet">
        {previewOnly
          ? "Reading these instructions does not connect accounts or save any credentials."
          : "Already have this account? Check the stopping point above, then mark it ready. You will authorize access in a later step."}
      </p>
    </>
  );
}
