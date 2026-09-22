import type {
  Accounts,
  Bridge,
  Installation,
  Selection,
  Snapshot,
  Step,
} from "./types";

const key = "villow-setup-demo-v1";
const date = "2026-09-10T00:00:00Z";
const release = {
  app_version: "0.1.0-demo",
  commit: "demo-only-not-a-release",
  notes:
    "A fictional release used only to demonstrate setup and recovery. No accounts, credentials or network requests are used.",
  google_scopes: [
    "https://www.googleapis.com/auth/youtube.force-ssl",
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/userinfo.profile",
  ],
  schema: { revision: "demo-baseline" },
};
export type Failure =
  | "none"
  | "offline"
  | "expired"
  | "rate_limit"
  | "lost_response"
  | "health";
export class DemoBridge implements Bridge {
  demo = true;
  failure: Failure = "none";
  private snapshot: Snapshot;
  private remote: { vercel: boolean; database: boolean };
  constructor(
    private storage: Pick<
      Storage,
      "getItem" | "setItem" | "removeItem"
    > = localStorage,
  ) {
    this.snapshot = {
      manager_version: "0.1.0",
      trust_configured: true,
      installation: null,
      release,
      release_checked_at: date,
      message: "Demo only. All resources and checks are simulated.",
    };
    this.remote = { vercel: false, database: false };
    try {
      const saved = storage.getItem(key);
      if (saved) {
        const data = JSON.parse(saved);
        this.snapshot = data.snapshot;
        this.remote = data.remote;
      }
    } catch {
      storage.removeItem(key);
    }
  }
  private save() {
    this.storage.setItem(
      key,
      JSON.stringify({ snapshot: this.snapshot, remote: this.remote }),
    );
  }
  async call<T>(
    command: string,
    args: Record<string, unknown> = {},
  ): Promise<T> {
    const s = this.snapshot.installation;
    if (command === "snapshot" || command === "check_release")
      return structuredClone(this.snapshot) as T;
    if (command === "open_step") return undefined as T;
    if (command === "start_installation") {
      if (s) throw new Error("The demo already has an installation.");
      this.snapshot.installation = {
        format: 1,
        id: "00000000-0000-4000-8000-000000000001",
        operation_id: "demo-operation",
        name: `${args.name}-demo`,
        owner_email: String(args.email),
        release_digest: "demo-digest",
        app_version: release.app_version,
        commit: release.commit,
        release_sequence: 1,
        schema_revision: "demo-baseline",
        google_scopes: release.google_scopes,
        created_at: date,
        updated_at: date,
        step: "projects",
        selection: null,
        vercel: null,
        database: null,
        origin: null,
        google: null,
        db_connection: null,
        deployment_id: null,
        effects: {},
        checks: [],
        read_only: false,
        credentials_removed: false,
      };
    } else if (command === "discover_accounts") {
      return {
        vercel_user: "demo-owner",
        supabase_user: "demo-database-owner",
        vercel: [
          {
            id: "demo-team",
            slug: "demo-team",
            name: "Your hosting account (demo)",
          },
        ],
        supabase: [
          {
            id: "demo-org",
            slug: "demo-org",
            name: "Your database account (demo)",
          },
        ],
      } satisfies Accounts as T;
    } else if (command === "forget_instance") {
      if (args.confirmation !== s?.name)
        throw new Error("Type the exact instance name.");
      this.snapshot.installation = null;
      this.remote = { vercel: false, database: false };
    } else if (command === "import_recovery") {
      throw new Error(
        "Recovery import is available in the installed desktop application.",
      );
    } else {
      if (!s) throw new Error("Start the demo first.");
      if (command === "save_credentials") s.credentials_removed = false;
      else if (command === "select_accounts")
        s.selection = args.selection as Selection;
      else if (command === "set_google")
        s.google = args.google as Installation["google"];
      else if (command === "set_database_connection")
        s.db_connection = args.connection as Installation["db_connection"];
      else if (command === "remove_credentials") s.credentials_removed = true;
      else if (command === "export_recovery") return true as T;
      else if (command === "advance") {
        if (s.credentials_removed)
          throw new Error("Reconnect your demo credentials.");
        if (!s.selection)
          throw new Error("Choose and confirm the provider accounts.");
        const failure = this.failure;
        this.failure = "none";
        if (failure === "offline")
          throw new Error(
            "Demo: offline. Your saved progress is available when you reconnect.",
          );
        if (failure === "expired")
          throw new Error(
            "Demo: token expired. Reconnect the same account to continue.",
          );
        if (failure === "rate_limit")
          throw new Error(
            "Demo: provider rate limit. Progress is saved; retry later.",
          );
        if (failure === "health" || (s.step === "health" && failure !== "none"))
          throw new Error(
            "Demo: intended-owner sign-in has not passed verification.",
          );
        if (s.step === "projects") {
          const provider = !s.vercel ? "vercel" : "database";
          this.remote[provider] = true;
          if (failure === "lost_response") {
            s.effects[`create_${provider}`] = {
              status: "needs_review",
              started_at: date,
              verified_at: null,
            };
            this.save();
            throw new Error(
              "Demo: the project was created but its reply was lost. Resume reconciles the fake provider’s operation ID without creating a duplicate. Real providers may require manual review.",
            );
          }
          s[provider] = {
            id: `demo-${provider}-project`,
            account_id: provider === "vercel" ? "demo-team" : "demo-org",
            name: s.name,
            operation_id: s.operation_id,
            evidence: "simulated_operation_id",
          };
          s.effects[`create_${provider}`] = {
            status: "verified",
            started_at: date,
            verified_at: date,
          };
          if (s.vercel && s.database) s.step = "origin";
        } else if (s.step === "origin") {
          s.origin = "https://your-villow-demo.vercel.app";
          s.step = "google";
        } else if (s.step === "google") {
          if (
            !s.google?.api_enabled_confirmed ||
            !(s.google.audience === "external_testing"
              ? s.google.testing_access_confirmed &&
                !s.google.consent_published_confirmed
              : ["external_production", "internal"].includes(
                  s.google.audience,
                ) && s.google.consent_published_confirmed)
          )
            throw new Error(
              "Confirm Google configuration and the access requirements for its current publishing status.",
            );
          s.step = "database";
        } else {
          const next: Partial<Record<Step, Step>> = {
            database: "configuration",
            configuration: "deployment",
            deployment: "health",
            health: "complete",
          };
          s.step = next[s.step] ?? s.step;
          if (s.step === "health") s.deployment_id = "demo-deployment";
        }
        s.checks.push({
          kind: "demo",
          title: `Simulated progress: ${s.step}`,
          at: date,
        });
        s.updated_at = date;
      } else throw new Error("Unsupported demo command.");
    }
    this.save();
    return structuredClone(this.snapshot) as T;
  }
}
