import { CopyAddress } from "./CopyAddress";
import { GuideImage } from "./GuideImage";

export function GoogleScopes({
  scopes,
  busy,
}: {
  scopes: string[];
  busy: boolean;
}) {
  return (
    <section aria-label="Google permissions">
      <h3>Add the permissions without searching every page</h3>
      <ol className="instructions">
        <li>
          In Google’s <b>Data Access</b> tab, click <b>Add or remove scopes</b>.
        </li>
        <li>
          Click <b>Copy scope list</b> below. Scroll to{" "}
          <b>Manually add scopes</b>
          in Google’s panel, paste the list and click <b>Add to table</b>.
        </li>
        <li>
          Check the selected entries match this list. Click <b>Update</b> at the
          bottom of the panel, then <b>Save</b> on Data Access if Google offers
          it.
        </li>
      </ol>
      <CopyAddress
        label="Exact Google permissions for this release"
        value={scopes.join("\n")}
        buttonLabel="Copy scope list"
        disabled={busy}
        manualCopyHint="Couldn’t copy. Select the permission list and press Ctrl+C."
      />
      <p>
        The YouTube permission includes account write access used by Villow. If
        a scope is missing, check YouTube Data API v3 is enabled in the same
        project and refresh Google’s page.
      </p>
      <div>
        <h3>Matching checkboxes: identity and YouTube</h3>
        <p>
          These reference screenshots show the three permissions for Villow
          0.1.0. Page numbers may change. Always use the exact list above for
          your release.
        </p>
        <GuideImage name="google-identity-scopes" />
        <GuideImage name="google-youtube-scope" />
      </div>
    </section>
  );
}
