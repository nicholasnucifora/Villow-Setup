export type Step =
  | "projects"
  | "origin"
  | "google"
  | "database"
  | "configuration"
  | "deployment"
  | "health"
  | "complete";
export interface Resource {
  id: string;
  account_id: string;
  name: string;
  operation_id: string;
  evidence: string;
}
export interface Account {
  id: string;
  name: string;
  slug: string;
}
export interface Accounts {
  vercel_user: string;
  supabase_user: string;
  vercel: Account[];
  supabase: Account[];
}
export interface Selection {
  vercel_user: string;
  supabase_user: string;
  vercel_account: string;
  supabase_organization: string;
  supabase_slug: string;
  region: string;
  costs_acknowledged: boolean;
}
export interface Google {
  project_id: string;
  client_id: string;
  api_enabled_confirmed: boolean;
  audience: string;
  consent_published_confirmed: boolean;
}
export interface Installation {
  format: number;
  id: string;
  operation_id: string;
  name: string;
  owner_email: string;
  release_digest: string;
  app_version: string;
  commit: string;
  release_sequence: number;
  schema_revision: string;
  google_scopes: string[];
  created_at: string;
  updated_at: string;
  step: Step;
  selection: Selection | null;
  vercel: Resource | null;
  database: Resource | null;
  origin: string | null;
  google: Google | null;
  db_connection: { host: string; user: string } | null;
  deployment_id: string | null;
  effects: Record<
    string,
    { status: string; started_at: string; verified_at: string | null }
  >;
  checks: { kind: string; title: string; at: string }[];
  read_only: boolean;
  credentials_removed: boolean;
}
export interface Release {
  app_version: string;
  commit: string;
  notes: string;
  google_scopes: string[];
  schema: { revision: string };
  [key: string]: unknown;
}
export interface Snapshot {
  manager_version: string;
  trust_configured: boolean;
  installation: Installation | null;
  release: Release | null;
  release_digest?: string | null;
  release_checked_at: string | null;
  message: string;
}
export interface Bridge {
  call<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  demo: boolean;
}
