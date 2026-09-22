# Supabase database root certificate

`supabase-prod-ca-2021.crt` is a public CA certificate, not a client credential or a Villow release-signing key. It is embedded only in the PostgreSQL TLS connector. It is not installed into Windows' trust store and does not change HTTP or release trust.

Retrieved over verified HTTPS on 2026-09-22 from the production URL used by Supabase's dashboard:

https://supabase-downloads.s3-ap-southeast-1.amazonaws.com/prod/ssl/prod-ca-2021.crt

Source of the URL: `apps/studio/hooks/custom-content/custom-content.json` in official `supabase/supabase` commit `2f1ad03640d23006d7ba8f4b6e60da936b800ae2`, consumed by `apps/studio/components/interfaces/Settings/Database/SSLConfiguration.tsx`.

- Subject/issuer: Supabase Root 2021 CA, Supabase Inc
- CA: true
- Validity: 2021-04-28 10:56:53 UTC to 2031-04-26 10:56:53 UTC
- SHA-256 of DER certificate: `807025ad50d4ed219d2c9c7d299c004f824eb00cf7f65afef607d07b72e6cafa`

Native TLS retains chain and hostname verification and a TLS 1.2 minimum. Review certificate replacements against Supabase's official dashboard source; never accept an arbitrary server-presented certificate or disable verification to recover a connection.
