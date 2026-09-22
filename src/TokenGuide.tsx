import { GuideImage } from "./GuideImage";

export function TokenExpiry() {
  return (
    <details>
      <summary>What happens when a token expires?</summary>
      <p>
        Your deployed Villow website keeps running. These management tokens let
        Setup create and configure your cloud resources; they are not used by
        your website to serve pages or sign you in.
      </p>
      <p>
        Choose an expiry that covers your setup and testing, for example 7 days.
        If a token expires or is revoked before you finish, Setup cannot
        continue its provider actions until you replace it. Open Recovery &amp;
        settings → Reconnect expired provider access, create a replacement for
        the same account and scope, and replace only that token. Your saved
        progress and resources stay in place; do not start another installation.
      </p>
      <p>
        After setup and installation checks are complete, you can revoke these
        management tokens in the providers’ token pages. Google sign-in and
        database credentials are separate; do not revoke those as part of this.
      </p>
    </details>
  );
}

export function TokenGuide({
  provider,
  busy,
  open,
}: {
  provider: "vercel" | "supabase";
  busy: boolean;
  open: (step: string) => Promise<void>;
}) {
  return (
    <section
      aria-label={`${provider === "vercel" ? "Vercel" : "Supabase"} token instructions`}
    >
      <h3>Create an access token</h3>
      <button
        className="secondary"
        disabled={busy}
        onClick={() => open(`${provider}_token`)}
      >
        {provider === "vercel"
          ? "Open Vercel personal tokens"
          : "Open Supabase access tokens"}{" "}
        ↗
      </button>
      {provider === "vercel" ? (
        <>
          <p>
            This opens vercel.com/account/settings/tokens in your personal
            account. Team or project Settings can show Billing, Members and Key
            Management without a Tokens entry. Use this direct link, or switch
            to your personal account before opening Settings.
          </p>
          <ol className="instructions">
            <li>
              Name the token <b>Villow Setup</b>.
            </li>
            <li>
              Choose the scope for the account/team where you want Villow
              hosted.
            </li>
            <li>
              Choose an expiry that gives you time to finish and test, for
              example <b>7 days</b>.
            </li>
            <li>
              Copy the token while Vercel displays it, then paste it below.
            </li>
          </ol>
        </>
      ) : (
        <>
          <p>
            Name the token <b>Villow Setup</b> and choose an expiry that covers
            setup and testing, for example <b>7 days</b>. This is a management
            access token, not your database password or a project API key.
          </p>
          <p>
            Under <b>Resource access</b>, choose <b>Organization</b> (all
            projects in selected organizations), then select only your dedicated
            Villow organization. Setup creates a new project, so selecting an
            existing project cannot cover it.
          </p>
          <p>
            The <b>No access</b> preset and all-None defaults will not work.
            Expand these permission groups and change only the following
            entries:
          </p>
          <div className="permission-table-wrap">
            <table className="permission-table">
              <caption>Supabase permissions for Villow Setup</caption>
              <thead>
                <tr>
                  <th scope="col">Group</th>
                  <th scope="col">Permission</th>
                  <th scope="col">Access</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Project</td>
                  <td>Project Settings</td>
                  <td>Read</td>
                </tr>
                <tr>
                  <td>Application services</td>
                  <td>API Keys</td>
                  <td>Read</td>
                </tr>
                <tr>
                  <td>Application services</td>
                  <td>API Key Secrets</td>
                  <td>Read</td>
                </tr>
                <tr>
                  <td>Account and organization</td>
                  <td>Organizations</td>
                  <td>Read</td>
                </tr>
                <tr>
                  <td>Account and organization</td>
                  <td>Projects (account-wide)</td>
                  <td>Read</td>
                </tr>
                <tr>
                  <td>Account and organization</td>
                  <td>Organization Projects</td>
                  <td>Read-write</td>
                </tr>
              </tbody>
            </table>
          </div>
          <p>
            Leave everything else at <b>None</b>, including Database and
            Infrastructure and delivery. Setup uses its generated database
            password for the later SQL step; it does not use this token’s SQL
            permissions. Review the access, create the token and paste it below.
          </p>
          <details>
            <summary>
              If permissions are unavailable or access is refused
            </summary>
            <p>
              Your Supabase account must itself be allowed to create projects in
              that organization. A token cannot grant more access than your
              role. If an entry is unavailable, check your role and selected
              organization.
            </p>
            <p>
              Scoped tokens are still a Supabase alpha feature. These settings
              follow its published permissions and Setup’s requests; a full live
              installation with them has not yet been qualified. If access is
              refused, check the selections and expiry before retrying. A
              successful account list alone does not prove project creation and
              key access.
            </p>
            <p>
              “Create legacy token” grants your account’s full access across its
              organizations and projects. It is not needed to follow the scoped
              instructions above. If your account only offers legacy tokens, use
              an account dedicated to this test, a short expiry and revoke the
              token when finished.
            </p>
          </details>
        </>
      )}
      <details>
        <summary>Screenshot guide</summary>
        <GuideImage name={`${provider}-token`} />
      </details>
      <TokenExpiry />
    </section>
  );
}
