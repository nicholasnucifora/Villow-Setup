import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cleanup,
  render,
  screen,
  waitFor,
  fireEvent,
} from "@testing-library/react";
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
        "Public installation is not available in this build",
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
    expect(await screen.findByText(attack)).toBeInTheDocument();
    expect(container.querySelector("img")).toBeNull();
    expect(container.querySelector("script")).toBeNull();
  });
  it("walks through the demo and exposes the exact callback", async () => {
    const user = userEvent.setup();
    const demo = new DemoBridge();
    render(<App initialBridge={demo} />);
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
        screen.getByRole("button", { name: /Create the next project/ }),
      ).toBeEnabled(),
    );
    await user.click(
      screen.getByRole("button", { name: /Create the next project/ }),
    );
    await waitFor(() =>
      expect(screen.getByText("demo-vercel-project")).toBeInTheDocument(),
    );
    await user.click(
      screen.getByRole("button", { name: /Create the next project/ }),
    );
    await user.click(
      await screen.findByRole("button", { name: /Reserve my address/ }),
    );
    expect(
      await screen.findByText("https://your-villow-demo.vercel.app/api/auth"),
    ).toBeInTheDocument();
    const select = screen.getByLabelText("Audience");
    fireEvent.change(select, { target: { value: "external_testing" } });
    expect(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    ).toBeDisabled();
    expect(screen.getByText(/seven days/)).toBeInTheDocument();
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
      await screen.findByRole("heading", { name: "Your accounts" });
      await user.selectOptions(screen.getByLabelText("Demo failure"), failure);
      await user.click(
        screen.getByRole("button", { name: /Create the next project/ }),
      );
      expect(await screen.findByRole("alert")).toHaveFocus();
      expect(screen.getByLabelText("Demo failure")).toHaveValue("none");
      expect(screen.queryByText("demo-vercel-project")).not.toBeInTheDocument();
      await user.click(
        screen.getByRole("button", { name: /Create the next project/ }),
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
      await screen.findByRole("button", { name: /Create the next project/ }),
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
    await screen.findByRole("heading", { name: "Connect Google" });
    await user.click(
      screen.getByRole("checkbox", { name: /I enabled YouTube/ }),
    );
    await user.click(
      screen.getByRole("checkbox", { name: /I configured the audience/ }),
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
  it("restores saved Google choices and blocks continuation while edits are unsaved", async () => {
    const user = userEvent.setup();
    await googleDemo();
    render(<App initialBridge={new DemoBridge()} />);
    const next = await screen.findByRole("button", {
      name: /Continue to my database/,
    });
    const audience = screen.getByLabelText("Audience");
    const enabled = screen.getByRole("checkbox", { name: /I enabled YouTube/ });
    const published = screen.getByRole("checkbox", {
      name: /I configured the audience/,
    });
    expect(audience).toHaveValue("internal");
    expect(enabled).toBeChecked();
    expect(published).toBeChecked();
    expect(next).toBeEnabled();
    await user.selectOptions(audience, "external_testing");
    expect(next).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Save my Google configuration" }),
    ).toBeDisabled();
    await user.selectOptions(audience, "internal");
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
      screen.queryByRole("button", { name: /Create the next project/ }),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getAllByRole("button", { name: "Load demo accounts" })[0],
    );
    expect(
      await screen.findByRole("button", { name: /Create the next project/ }),
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
      await screen.findByRole("button", { name: /Start my setup/ }),
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

async function googleDemo() {
  const demo = await selectedDemo();
  for (let i = 0; i < 3; i++) await demo.call("advance");
  await demo.call("set_google", {
    google: {
      project_id: "demo-google-project",
      client_id: "demo.apps.googleusercontent.com",
      api_enabled_confirmed: true,
      consent_published_confirmed: true,
      audience: "internal",
    },
  });
  return demo;
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
