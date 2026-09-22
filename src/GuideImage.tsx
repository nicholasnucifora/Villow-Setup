import googleIdentityScopes from "./assets/account-guide/google-identity-scopes.png";
import googleYoutubeScope from "./assets/account-guide/google-youtube-scope.png";
import googleTestUsers from "./assets/account-guide/google-test-users.png";
import googleClientCreated from "./assets/account-guide/google-client-created.png";

// Replace remaining null entries with reviewed, redacted screenshots.
const images: Record<string, { src: string | null; caption: string }> = {
  "vercel-account": {
    src: null,
    caption: "Vercel dashboard — highlight the account menu and where to stop.",
  },
  "supabase-organization": {
    src: null,
    caption:
      "Supabase organization — highlight the organization name and the route back from Create a new project.",
  },
  "google-project": {
    src: null,
    caption:
      "Google Cloud dashboard — highlight the project selector and Project ID in Project info.",
  },
  "vercel-token": {
    src: null,
    caption:
      "Vercel personal account token page — highlight the direct URL, team scope and expiry; hide the token.",
  },
  "supabase-token": {
    src: null,
    caption:
      "Supabase Generate token page — highlight the small Create legacy token link directly under Resource access, then the name and expiry fields; hide the token.",
  },
  "google-oauth": {
    src: null,
    caption:
      "Google Create OAuth client form — highlight Application type: Web application, Name, Authorized JavaScript origins and Authorized redirect URIs; hide all credentials.",
  },
  "google-audience": {
    src: googleTestUsers,
    caption:
      "In Google Auth Platform → Audience, scroll down to Test users and click + Add users, highlighted in red. Keep User type External and status Testing.",
  },
  "google-client-created": {
    src: googleClientCreated,
    caption:
      "Use the copy icon on the right of Client ID, then Client secret. Paste each into its matching field in Setup. The highlighted example values are placeholders, not credentials to use.",
  },
  "google-identity-scopes": {
    src: googleIdentityScopes,
    caption:
      "Select userinfo.email and userinfo.profile. The full permission URLs above are the source of truth for your release; this example leaves openid unchecked.",
  },
  "google-youtube-scope": {
    src: googleYoutubeScope,
    caption:
      "Select youtube.force-ssl under YouTube Data API v3. In this example it is on the last page (31–38 of 38). Use Manually add scopes below the table to avoid paging through the list, then click Update.",
  },
};

export function GuideImage({ name }: { name: keyof typeof images }) {
  const image = images[name];
  return (
    <figure className="guide-image" data-guide-image={name}>
      {image.src ? (
        <img src={image.src} alt={image.caption} />
      ) : (
        <div className="image-placeholder">
          <svg aria-hidden="true" viewBox="0 0 48 36" fill="none">
            <rect
              x="1"
              y="1"
              width="46"
              height="34"
              rx="4"
              stroke="currentColor"
              strokeWidth="2"
            />
            <path
              d="m4 29 12-12 9 9 6-6 13 12"
              stroke="currentColor"
              strokeWidth="2"
            />
            <circle
              cx="34"
              cy="11"
              r="4"
              stroke="currentColor"
              strokeWidth="2"
            />
          </svg>
          <span>Screenshot coming soon</span>
        </div>
      )}
      <figcaption>{image.caption}</figcaption>
    </figure>
  );
}
