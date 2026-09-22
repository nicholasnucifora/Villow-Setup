// Replace each null src with an imported, redacted screenshot when available.
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
      "Supabase scoped token form — highlight Organization resource access and the six permissions listed above; hide the token.",
  },
  "google-oauth": {
    src: null,
    caption:
      "Google OAuth Web application form — highlight the origin, redirect URI and client ID; hide the secret.",
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
