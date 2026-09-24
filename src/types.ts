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
  testing_access_confirmed?: boolean;
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
  deployment_status?:
    | "queued"
    | "building"
    | "assigning_address"
    | "ready"
    | "failed"
    | "canceled"
    | "address_failed"
    | null;
  effects: Record<
    string,
    { status: string; started_at: string; verified_at: string | null }
  >;
  checks: { kind: string; title: string; at: string }[];
  read_only: boolean;
  credentials_removed: boolean;
  fresh_retry?: { from: string; to: string } | null;
  installed_repair?: RepairIntent | null;
  app_update?: RepairIntent | null;
}
export interface RepairIntent {
  from: string;
  to: string;
  repair_id: string;
  operation_id: string;
  previous_operation_id: string;
  previous_deployment_id: string;
  backup_confirmed_at: string;
  backup?: {
    managed?: boolean;
    removed_at?: string | null;
    path: string;
    sha256: string;
    bytes: number;
    captured_at: string;
    installation_id: string;
    operation_id: string;
    from: string;
    to: string;
  } | null;
  phase: "database" | "upload" | "deploy" | "verify" | "complete";
  deployment_id: string | null;
  deployment_status: Installation["deployment_status"];
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
  fresh_retry?: { digest: string; app_version: string } | null;
  installed_repair?: { digest: string; app_version: string } | null;
  app_update?: {
    status: "current" | "manager_required" | "unsupported" | "available";
    digest?: string;
    downtime?: string;
    app_version: string;
    minimum_manager: string;
    message: string;
    notes: string;
  } | null;
}
export interface Bridge {
  call<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  demo: boolean;
}
