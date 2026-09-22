import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { App } from "../src/App";
import { DemoBridge } from "../src/demo";
import type { Bridge, Snapshot } from "../src/types";
beforeEach(() => {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
  Element.prototype.scrollIntoView = vi.fn();
});
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.restoreAllMocks();
});

describe("owner-facing setup", () => {
  it("keeps cloud setup unavailable without an authenticated release", async () => {
    const bridge: Bridge = {
      demo: false,
      call: vi.fn().mockResolvedValue({
        manager_version: "0.1.0",
        trust_configured: false,
        installation: null,
        release: null,
        release_checked_at: null,
        message: "Unconfigured release",
      } satisfies Snapshot),
    };
    render(<App initialBridge={bridge} />);
    expect(
      await screen.findByText(
        "Cloud installation is not available in this build",
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /Start my setup/ }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText(/management credentials stay on this computer/),
    ).toBeInTheDocument();
  });
  it("renders hostile release notes as inert text", async () => {
    const attack =
      '<img src=x onerror="window.stolen=true"><script>steal()</script>';
    const bridge: Bridge = {
      demo: true,
      call: vi.fn().mockResolvedValue({
        manager_version: "0.1.0",
        trust_configured: true,
        installation: null,
        release: {
          app_version: "test",
          commit: "test",
          notes: attack,
          google_scopes: [],
          schema: { revision: "test" },
        },
        release_checked_at: null,
        message: "",
      } satisfies Snapshot),
    };
    const { container } = render(<App initialBridge={bridge} />);
    await prepareAccounts(userEvent.setup());
    expect(await screen.findByText(attack)).toBeInTheDocument();
    expect(container.querySelector("img")).toBeNull();
    expect(container.querySelector("script")).toBeNull();
  });
  it("walks through the demo and exposes the exact callback", async () => {
    const user = userEvent.setup();
    const demo = new DemoBridge();
    render(<App initialBridge={demo} />);
    await prepareAccounts(user);
    await screen.findByRole("button", { name: /Start my setup/ });
    await user.type(
      screen.getByLabelText("Your Google account email"),
      "owner@example.test",
    );
    await user.click(
      screen.getByRole("checkbox", { name: /dedicated projects/ }),
    );
    await user.click(screen.getByRole("button", { name: /Start my setup/ }));
    await user.click(
      await screen.findByRole("button", { name: "Load demo accounts" }),
    );
    await user.click(
      await screen.findByRole("checkbox", { name: /intended accounts/ }),
    );
    await user.click(
      screen.getByRole("button", { name: "Confirm these accounts" }),
    );
    await waitFor(() =>
      expect(
        screen.getByRole("button", {
          name: /Create (?:Vercel|Supabase) project/,
        }),
      ).toBeEnabled(),
    );
    await user.click(
      screen.getByRole("button", {
        name: /Create (?:Vercel|Supabase) project/,
      }),
    );
    await waitFor(() =>
      expect(screen.getByText("demo-vercel-project")).toBeInTheDocument(),
    );
    await user.click(
      screen.getByRole("button", {
        name: /Create (?:Vercel|Supabase) project/,
      }),
    );
    await user.click(
      await screen.findByRole("button", { name: /Reserve my address/ }),
    );
    expect(
      await screen.findByText("https://your-villow-demo.vercel.app/api/auth"),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("combobox", { name: "Audience" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    ).toBeDisabled();
    expect(
      screen.getByText(/Google access expires after seven days from consent/),
    ).toBeInTheDocument();
  });
  it("persists a lost-response demo across a new application instance", async () => {
    const demo = new DemoBridge();
    await demo.call("start_installation", {
      name: "test",
      email: "owner@example.test",
    });
    await demo.call("select_accounts", {
      selection: {
        vercel_user: "demo-owner",
        vercel_account: "demo-team",
        supabase_organization: "demo-org",
        supabase_slug: "demo-org",
        region: "ap-southeast-2",
        costs_acknowledged: true,
      },
    });
    demo.failure = "lost_response";
    await expect(demo.call("advance")).rejects.toThrow(/reply was lost/);
    const reopened = new DemoBridge();
    const resumed = await reopened.call<Snapshot>("advance");
    expect(resumed.installation?.vercel?.id).toBe("demo-vercel-project");
    expect(resumed.installation?.effects.create_vercel.status).toBe("verified");
    expect(localStorage.getItem("villow-setup-demo-v1")).not.toContain(
      "demo-secret",
    );
  });
  it("does not collect real credentials in demo mode", async () => {
    const demo = new DemoBridge();
    await demo.call("start_installation", {
      name: "test",
      email: "owner@example.test",
    });
    render(<App initialBridge={demo} />);
    await screen.findByRole("button", { name: "Load demo accounts" });
    expect(document.querySelector("input[type=password]")).toBeNull();
  });
  it.each(["offline", "expired", "rate_limit", "lost_response"] as const)(
    "brings a %s failure to keyboard focus and allows a fresh retry",
    async (failure) => {
      const user = userEvent.setup();
      const demo = await selectedDemo();
      render(<App initialBridge={demo} />);
      await screen.findByRole("heading", { name: "Vercel & Supabase" });
      await user.selectOptions(screen.getByLabelText("Demo failure"), failure);
      await user.click(
        screen.getByRole("button", {
          name: /Create (?:Vercel|Supabase) project/,
        }),
      );
      expect(await screen.findByRole("alert")).toHaveFocus();
      expect(screen.getByLabelText("Demo failure")).toHaveValue("none");
      expect(screen.queryByText("demo-vercel-project")).not.toBeInTheDocument();
      await user.click(
        screen.getByRole("button", {
          name: /Create (?:Vercel|Supabase) project/,
        }),
      );
      expect(
        await screen.findByText("demo-vercel-project"),
      ).toBeInTheDocument();
      expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    },
  );
  it("moves keyboard focus to a new step and the opened recovery controls", async () => {
    const user = userEvent.setup();
    const demo = await selectedDemo();
    await demo.call("advance");
    render(<App initialBridge={demo} />);
    await user.click(
      await screen.findByRole("button", {
        name: /Create (?:Vercel|Supabase) project/,
      }),
    );
    expect(
      await screen.findByRole("heading", { name: "Your address" }),
    ).toHaveFocus();
    await user.click(
      screen.getByRole("button", { name: "Recovery & settings" }),
    );
    expect(
      screen.getByRole("region", { name: "Recovery and settings" }),
    ).toHaveFocus();
    await user.click(
      screen.getByRole("button", { name: "Save recovery information" }),
    );
    expect(await screen.findByRole("status")).toHaveFocus();
    expect(screen.getByRole("status")).toHaveTextContent("export simulated");
  });
  it("finishes the simulated journey without exposing a database password field", async () => {
    const user = userEvent.setup();
    const demo = await selectedDemo();
    await demo.call("advance");
    await demo.call("advance");
    await demo.call("advance");
    render(<App initialBridge={demo} />);
    await screen.findByRole("heading", { name: "Google Cloud", level: 1 });
    await chooseTesting(user);
    await user.click(
      screen.getByRole("checkbox", { name: /I enabled YouTube/ }),
    );
    await user.click(
      screen.getByRole("checkbox", {
        name: /I configured the required Google permissions/,
      }),
    );
    await user.click(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    );
    await user.click(
      await screen.findByRole("button", { name: /Continue to my database/ }),
    );
    expect(
      await screen.findByRole("heading", { name: "Prepare the database" }),
    ).toHaveFocus();
    expect(document.querySelector("input[type=password]")).toBeNull();
    expect(screen.queryByLabelText("Database host")).not.toBeInTheDocument();
    for (const name of [
      "Prepare my database",
      "Connect my services",
      "Build my app on Vercel",
    ]) {
      await user.click(screen.getByRole("button", { name: new RegExp(name) }));
    }
    await user.selectOptions(screen.getByLabelText("Demo failure"), "health");
    await user.click(
      screen.getByRole("button", { name: /Check my installation/ }),
    );
    expect(await screen.findByRole("alert")).toHaveFocus();
    expect(
      screen.queryByText("You’ve finished the demo."),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getByRole("button", { name: /Check my installation/ }),
    );
    expect(
      await screen.findByRole("heading", { name: "You’ve finished the demo." }),
    ).toHaveFocus();
    expect(screen.getByText(/simulated checks passed/)).toBeInTheDocument();
    expect(localStorage.getItem("villow-setup-demo-v1")).not.toContain(
      "demo-secret",
    );
  });
  it("keeps Google values entered along the guide and saves once through the native boundary", async () => {
    const user = userEvent.setup();
    const fixture = await selectedDemo();
    for (let i = 0; i < 3; i++) await fixture.call("advance");
    const call = vi.fn(fixture.call.bind(fixture));
    const bridge: Bridge = {
      demo: false,
      call: <T,>(command: string, args?: Record<string, unknown>) =>
        call(command, args) as Promise<T>,
    };
    render(<App initialBridge={bridge} />);
    const project = await screen.findByLabelText("Google Cloud project ID");
    const audienceImage = screen.getByRole("img", {
      name: /scroll down to Test users/,
    });
    const clientImage = screen.getByRole("img", {
      name: /Use the copy icon on the right of Client ID/,
    });
    expect(audienceImage).toHaveAttribute(
      "src",
      expect.stringContaining("google-test-users.png"),
    );
    expect(clientImage).toHaveAttribute(
      "src",
      expect.stringContaining("google-client-created.png"),
    );
    expect(audienceImage.closest("details")).toBeNull();
    expect(clientImage.closest("details")).toBeNull();
    expect(
      screen.queryByText("Already switched Google to In production?"),
    ).not.toBeInTheDocument();
    await user.type(project, "example-villow-123456");
    expect(
      screen.getByText("Testing is enough to continue"),
    ).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Open Branding/ }));
    expect(project).toHaveValue("example-villow-123456");
    await user.click(
      screen.getByRole("checkbox", { name: /I enabled YouTube/ }),
    );
    await user.type(
      screen.getByLabelText("OAuth Web client ID"),
      "example.apps.googleusercontent.com",
    );
    await user.type(
      screen.getByLabelText("OAuth client secret"),
      "synthetic-client-secret",
    );
    await user.click(
      screen.getByRole("checkbox", {
        name: /I configured the required Google permissions/,
      }),
    );
    const save = screen.getByRole("button", {
      name: "Save my Google configuration",
    });
    expect(save).toBeDisabled();
    expect(call.mock.calls.some(([command]) => command === "set_google")).toBe(
      false,
    );
    await chooseTesting(user);
    await user.click(save);
    expect(call).toHaveBeenCalledWith("set_google", {
      google: {
        project_id: "example-villow-123456",
        client_id: "example.apps.googleusercontent.com",
        api_enabled_confirmed: true,
        audience: "external_testing",
        consent_published_confirmed: false,
        testing_access_confirmed: true,
      },
      secret: "synthetic-client-secret",
    });
    expect(screen.getByLabelText("OAuth client secret")).toHaveValue("");
    expect(JSON.stringify(localStorage)).not.toContain(
      "synthetic-client-secret",
    );
    expect(
      screen.getByRole("button", { name: /Continue to my database/ }),
    ).toBeEnabled();
    expect(
      screen.getByText(/You can close Google’s client dialog/),
    ).toBeInTheDocument();
    expect(call.mock.calls.some(([command]) => command === "advance")).toBe(
      false,
    );
  });
  it("restores saved Google choices and blocks continuation while edits are unsaved", async () => {
    const user = userEvent.setup();
    await googleDemo();
    render(<App initialBridge={new DemoBridge()} />);
    const next = await screen.findByRole("button", {
      name: /Continue to my database/,
    });
    const audience = screen.getByRole("checkbox", {
      name: /I added owner@example.test as a Google test user/,
    });
    const enabled = screen.getByRole("checkbox", { name: /I enabled YouTube/ });
    const published = screen.getByRole("checkbox", {
      name: /I configured the required Google permissions/,
    });
    expect(audience).toBeChecked();
    expect(enabled).toBeChecked();
    expect(published).toBeChecked();
    expect(next).toBeEnabled();
    await user.click(audience);
    expect(next).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    ).toBeDisabled();
    await user.click(audience);
    for (const checkbox of [enabled, published]) {
      await user.click(checkbox);
      expect(next).toBeDisabled();
      await user.click(checkbox);
      expect(next).toBeEnabled();
    }
    for (const label of ["Google Cloud project ID", "OAuth Web client ID"]) {
      await user.type(screen.getByLabelText(label), "-changed");
      expect(next).toBeDisabled();
      await user.click(
        screen.getByRole("button", { name: "Save my Google configuration" }),
      );
      await waitFor(() => expect(next).toBeEnabled());
    }
    await user.click(next);
    expect(
      await screen.findByRole("heading", { name: "Prepare the database" }),
    ).toHaveFocus();
  });
  it("continues in External Testing only with test-user acknowledgment and retains the reminder after reopening", async () => {
    const user = userEvent.setup();
    const demo = await selectedDemo();
    for (let i = 0; i < 3; i++) await demo.call("advance");
    const view = render(<App initialBridge={demo} />);
    await screen.findByRole("heading", { name: "Google Cloud", level: 1 });
    await user.click(
      screen.getByRole("checkbox", { name: /I enabled YouTube/ }),
    );
    await user.click(
      screen.getByRole("checkbox", {
        name: /I configured the required Google permissions/,
      }),
    );
    const save = screen.getByRole("button", {
      name: "Save my Google configuration",
    });
    expect(save).toBeDisabled();
    await user.click(
      screen.getByRole("checkbox", {
        name: /I added owner@example.test as a Google test user/,
      }),
    );
    expect(save).toBeEnabled();
    await user.click(save);
    const google = (await demo.call<Snapshot>("snapshot")).installation?.google;
    expect(google?.audience).toBe("external_testing");
    expect(google?.consent_published_confirmed).toBe(false);
    expect(google?.testing_access_confirmed).toBe(true);
    await user.click(
      screen.getByRole("button", { name: /Continue to my database/ }),
    );
    expect(
      await screen.findByRole("heading", { name: "Prepare the database" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("complementary", { name: "Google testing reminder" }),
    ).toHaveTextContent("seven days");
    view.unmount();
    render(<App initialBridge={new DemoBridge()} />);
    expect(
      await screen.findByRole("complementary", {
        name: "Google testing reminder",
      }),
    ).toHaveTextContent("You chose Google’s External Testing mode");
  });
  it("preserves an existing internal setup without offering Internal to new users", async () => {
    const user = userEvent.setup();
    await googleDemo("internal");
    render(<App initialBridge={new DemoBridge()} />);
    const next = await screen.findByRole("button", {
      name: /Continue to my database/,
    });
    expect(next).toBeEnabled();
    expect(
      screen.queryByRole("combobox", { name: "Audience" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByText("Previously saved Google configuration"),
    ).toBeVisible();
    await user.click(
      screen.getByRole("button", {
        name: "I changed Google to External / Testing",
      }),
    );
    expect(next).toBeDisabled();
    expect(
      screen.getByRole("checkbox", {
        name: /I added owner@example.test as a Google test user/,
      }),
    ).not.toBeChecked();
  });
  it("shows database recovery settings and requires saving edits before retrying with the retained password", async () => {
    const user = userEvent.setup();
    const demo = await googleDemo();
    await demo.call("advance");
    const call = vi.fn(
      async <T,>(
        command: string,
        args?: Record<string, unknown>,
      ): Promise<T> => {
        if (command === "advance")
          throw new Error("The database connection could not finish.");
        return demo.call<T>(command, args);
      },
    );
    render(
      <App
        initialBridge={{
          demo: false,
          call: <T,>(command: string, args?: Record<string, unknown>) =>
            call(command, args) as Promise<T>,
        }}
      />,
    );
    const host = await screen.findByLabelText("Database host");
    expect(host.closest("details")).toBeNull();
    const prepare = screen.getByRole("button", { name: /Prepare my database/ });
    expect(prepare).toBeEnabled();
    await user.type(host, "aws-1-ap-southeast-2.pooler.supabase.com");
    expect(prepare).toBeDisabled();
    const password = screen.getByLabelText(
      "Replacement database password (usually leave blank)",
    );
    expect(password).not.toBeRequired();
    expect(password).toHaveValue("");
    await user.click(
      screen.getByRole("button", { name: "Save connection settings" }),
    );
    expect(call).toHaveBeenCalledWith("set_database_connection", {
      connection: {
        host: "aws-1-ap-southeast-2.pooler.supabase.com",
        user: expect.stringMatching(/^postgres\./),
      },
      password: "",
    });
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Connection settings saved",
    );
    expect(prepare).toBeEnabled();
    await user.click(prepare);
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The database connection could not finish.",
    );
    expect(host).toHaveValue("aws-1-ap-southeast-2.pooler.supabase.com");
  });
  it("keeps a failed secret replacement unsaved even after clearing its input", async () => {
    const user = userEvent.setup();
    const demo = await googleDemo();
    let failSave = true;
    const bridge: Bridge = {
      demo: false,
      call: async (command, args) => {
        if (command === "set_google" && failSave)
          throw new Error("Synthetic save failure");
        return demo.call(command, args);
      },
    };
    render(<App initialBridge={bridge} />);
    const next = await screen.findByRole("button", {
      name: /Continue to my database/,
    });
    const secret = screen.getByLabelText("OAuth client secret");
    await user.type(secret, "synthetic-secret");
    expect(next).toBeDisabled();
    await user.click(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Synthetic save failure",
    );
    expect(secret).toHaveValue("");
    expect(next).toBeDisabled();
    failSave = false;
    await user.type(secret, "synthetic-retry");
    await user.click(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    );
    await waitFor(() => expect(next).toBeEnabled());
    expect(secret).toHaveValue("");
  });
  it("confirms local removal, preserves progress and requires a fresh acknowledgment after reconnecting", async () => {
    const user = userEvent.setup();
    const demo = await selectedDemo();
    render(<App initialBridge={demo} />);
    await user.click(
      await screen.findByRole("button", { name: "Recovery & settings" }),
    );
    const remove = screen.getByRole("button", {
      name: "Remove local credentials",
    });
    expect(remove).toBeDisabled();
    const acknowledgment = screen.getByRole("checkbox", {
      name: /including its encryption-key copy/,
    });
    await user.click(acknowledgment);
    await user.click(remove);
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Demo: saved access removed.",
    );
    expect(screen.getByRole("status")).toHaveFocus();
    expect(acknowledgment).not.toBeChecked();
    expect(
      (await demo.call<Snapshot>("snapshot")).installation?.credentials_removed,
    ).toBe(true);
    expect(
      screen.queryByRole("button", {
        name: /Create (?:Vercel|Supabase) project/,
      }),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getAllByRole("button", { name: "Load demo accounts" })[0],
    );
    expect(
      await screen.findByRole("button", {
        name: /Create (?:Vercel|Supabase) project/,
      }),
    ).toBeEnabled();
    expect(remove).toBeDisabled();
    const forget = screen.getByRole("button", {
      name: "Forget this instance locally",
    });
    await user.type(
      screen.getByLabelText("Type test-demo to forget it"),
      "wrong",
    );
    expect(forget).toBeDisabled();
    await user.clear(screen.getByLabelText("Type test-demo to forget it"));
    await user.type(
      screen.getByLabelText("Type test-demo to forget it"),
      "test-demo",
    );
    await user.click(forget);
    expect(
      await screen.findByRole("button", { name: /Begin setup/ }),
    ).toBeInTheDocument();
    expect((await demo.call<Snapshot>("snapshot")).installation).toBeNull();
  });
  it("announces loaded account access and clears that feedback after a failed reconnect", async () => {
    const user = userEvent.setup();
    const demo = await selectedDemo();
    const call = demo.call.bind(demo);
    let failDiscovery = false;
    vi.spyOn(demo, "call").mockImplementation(async (command, args) => {
      if (command === "discover_accounts" && failDiscovery)
        throw new Error("Synthetic account failure");
      return call(command, args);
    });
    render(<App initialBridge={demo} />);
    await user.click(
      await screen.findByRole("button", { name: "Recovery & settings" }),
    );
    await user.click(screen.getByText("Reconnect expired provider access"));
    await user.click(
      screen.getByRole("button", { name: "Load demo accounts" }),
    );
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Demo accounts loaded. Continue setup to check access against your saved accounts.",
    );
    failDiscovery = true;
    await user.click(
      screen.getByRole("button", { name: "Load demo accounts" }),
    );
    expect(await screen.findByRole("alert")).toHaveTextContent(
      "Synthetic account failure",
    );
    expect(screen.queryByText(/Demo accounts loaded/)).not.toBeInTheDocument();
  });
});

async function googleDemo(audience = "external_testing") {
  const demo = await selectedDemo();
  for (let i = 0; i < 3; i++) await demo.call("advance");
  await demo.call("set_google", {
    google: {
      project_id: "demo-google-project",
      client_id: "demo.apps.googleusercontent.com",
      api_enabled_confirmed: true,
      consent_published_confirmed: audience !== "external_testing",
      testing_access_confirmed: audience === "external_testing",
      audience,
    },
  });
  return demo;
}

async function chooseTesting(user: ReturnType<typeof userEvent.setup>) {
  await user.click(
    screen.getByRole("checkbox", {
      name: /I added owner@example.test as a Google test user/,
    }),
  );
}

async function selectedDemo() {
  const demo = new DemoBridge();
  await demo.call("start_installation", {
    name: "test",
    email: "owner@example.test",
  });
  await demo.call("select_accounts", {
    selection: {
      vercel_user: "demo-owner",
      vercel_account: "demo-team",
      supabase_organization: "demo-org",
      supabase_slug: "demo-org",
      region: "ap-southeast-2",
      costs_acknowledged: true,
    },
  });
  return demo;
}

async function prepareAccounts(user: ReturnType<typeof userEvent.setup>) {
  await user.click(await screen.findByRole("button", { name: /Begin setup/ }));
}
