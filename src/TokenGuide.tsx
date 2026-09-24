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
            For this Alpha, use a <b>legacy management token</b>. This is the
            path confirmed during account connection testing. It is different
            from a database password or a project API key.
          </p>
          <ol className="instructions">
            <li>
              On Access Tokens, click the main <b>Generate new token</b> button.
              The small arrow beside it opens a different menu.
            </li>
            <li>
              On the Generate token page, look directly under{" "}
              <b>Resource access</b>, on the left. In “Need a token with full
              access to your account?”, click the small, underlined{" "}
              <b>Create legacy token</b> link. Do not choose “Generate token for
              experimental API”.
            </li>
            <li>
              Name it <b>Villow Setup Alpha</b> and choose an expiry covering
              setup and testing, for example <b>7 days</b>. Generate the token.
            </li>
            <li>
              Copy the complete token while it is shown, paste it below and
              select <b>Save Supabase token &amp; read accounts</b>.
            </li>
          </ol>
          <div className="guide-takeaway">
            <strong>Use your dedicated test account</strong>
            <p>
              A legacy token has your account’s full access across all its
              organizations and projects, including ones you join later. Use it
              with the dedicated account for this Alpha and revoke it after
              installation testing.
            </p>
          </div>
          <details>
            <summary>
              Already created a token with individual permissions?
            </summary>
            <p>
              That is a scoped token. Account testing encountered a refusal at
              Setup’s profile check; a legacy replacement passed. The earlier
              scoped-permission instructions are not a confirmed working path
              for this Alpha. There is no verified extra checkbox to recommend.
            </p>
            <p>
              Create the legacy token using the link above, then replace only
              the Supabase token. Keep your saved Vercel token and current
              setup. You do not need to recreate any cloud projects.
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
