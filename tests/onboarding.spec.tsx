import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { App } from "../src/App";
import { DemoBridge } from "../src/demo";
import type { Accounts, Bridge, Snapshot } from "../src/types";

beforeEach(() => {
  vi.spyOn(window, "scrollTo").mockImplementation(() => {});
  Element.prototype.scrollIntoView = vi.fn();
});
afterEach(() => {
  cleanup();
  localStorage.clear();
  vi.restoreAllMocks();
});

it("lets a user read all provider pages without credentials or installation authorization", async () => {
  const user = userEvent.setup();
  const call = vi.fn().mockResolvedValue({
    manager_version: "test",
    trust_configured: false,
    installation: null,
    release: null,
    release_checked_at: null,
    message: "Unavailable",
  } satisfies Snapshot);
  render(<App initialBridge={{ demo: false, call }} />);
  await user.click(
    await screen.findByRole("button", { name: /Read the account guide/ }),
  );
  expect(
    screen.getByRole("heading", { name: "Vercel", level: 1 }),
  ).toHaveFocus();
  await user.click(screen.getByRole("button", { name: /Next: Supabase/ }));
  expect(
    screen.getByText(/Stop before creating a database project/),
  ).toBeInTheDocument();
  expect(
    screen.getByText(/Setup generates a strong password/),
  ).toBeInTheDocument();
  await user.click(screen.getByText(/I’m seeing security options/));
  expect(screen.getByText(/Enable Data API/)).toBeVisible();
  expect(
    screen.getByText(/This version cannot use an existing project/),
  ).toBeVisible();
  await user.click(screen.getByRole("button", { name: /Next: Google Cloud/ }));
  expect(
    screen.getByText(/Setup does not ask for the project number/),
  ).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: /Finish reading/ }));
  expect(
    screen.getByRole("heading", { name: "Account guide complete" }),
  ).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: /Start my setup/ }),
  ).not.toBeInTheDocument();
  expect(document.querySelector("input[type=password]")).toBeNull();
  expect(call.mock.calls.every(([command]) => command === "snapshot")).toBe(
    true,
  );
});

it("checks the release before asking a configured user to visit any provider", async () => {
  const user = userEvent.setup();
  const call = vi.fn().mockResolvedValue({
    manager_version: "test",
    trust_configured: true,
    installation: null,
    release: null,
    release_checked_at: null,
    message: "",
  } satisfies Snapshot);
  render(<App initialBridge={{ demo: false, call }} />);
  await user.click(await screen.findByRole("button", { name: /Begin setup/ }));
  expect(
    screen.getByRole("heading", { name: "Choose your release", level: 1 }),
  ).toHaveFocus();
  expect(
    screen.getByRole("button", { name: "Check the official release" }),
  ).toBeEnabled();
  expect(document.querySelector("input[type=password]")).toBeNull();
  expect(
    screen.queryByRole("button", { name: /Open Vercel/ }),
  ).not.toBeInTheDocument();
  expect(call.mock.calls.every(([command]) => command === "snapshot")).toBe(
    true,
  );
});

async function accountFixture() {
  const fixture = new DemoBridge();
  await fixture.call("start_installation", {
    name: "test",
    email: "owner@example.test",
  });
  // Exercise the normal UI with synthetic native responses; no real provider calls.
  const call = vi.fn(
    async <T,>(command: string, args?: Record<string, unknown>) =>
      fixture.call<T>(command, args),
  );
  const bridge: Bridge = {
    demo: false,
    call: <T,>(command: string, args?: Record<string, unknown>) =>
      call(command, args) as Promise<T>,
  };
  return { bridge, call, fixture };
}

async function saveVercel(user: ReturnType<typeof userEvent.setup>) {
  await user.type(
    await screen.findByLabelText("Vercel access token"),
    "synthetic-vercel",
  );
  await user.click(screen.getByRole("button", { name: /Save Vercel token/ }));
  await screen.findByLabelText("Supabase management token");
}

it("groups account and token steps by provider, saves each secret separately and confirms before creation", async () => {
  const user = userEvent.setup();
  const { bridge, call } = await accountFixture();
  render(<App initialBridge={bridge} />);
  const vercel = await screen.findByLabelText("Vercel access token");
  expect(
    screen.getByRole("button", { name: /Open Vercel personal tokens/ }),
  ).toBeEnabled();
  expect(
    screen.queryByLabelText("Supabase management token"),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: /Open Google Cloud/ }),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByRole("button", { name: "Load demo accounts" }),
  ).not.toBeInTheDocument();
  expect(
    screen.getByRole("button", { name: /Save Vercel token/ }),
  ).toBeDisabled();
  await saveVercel(user);
  expect(call).toHaveBeenCalledWith("save_credentials", {
    vercel: "synthetic-vercel",
    supabase: "",
  });
  expect(vercel).toHaveValue("");
  expect(
    screen.queryByLabelText("Vercel access token"),
  ).not.toBeInTheDocument();
  expect(
    call.mock.calls.some(([command]) => command === "discover_accounts"),
  ).toBe(false);
  expect(
    screen.getByRole("heading", { name: "Connect Supabase" }),
  ).toHaveFocus();
  expect(
    screen.getByText(/Stop before creating a database project/),
  ).toBeInTheDocument();
  const table = screen.getByRole("table", { name: /Supabase permissions/ });
  expect(within(table).getByText("Organization Projects")).toBeInTheDocument();
  expect(within(table).getByText("API Key Secrets")).toBeInTheDocument();
  expect(within(table).getAllByRole("row")).toHaveLength(7);
  const supabase = screen.getByLabelText("Supabase management token");
  await user.type(supabase, "synthetic-supabase");
  await user.click(screen.getByRole("button", { name: /Save Supabase token/ }));
  expect(call).toHaveBeenCalledWith("save_credentials", {
    vercel: "",
    supabase: "synthetic-supabase",
  });
  expect(
    screen.getByRole("heading", { name: "Review your accounts" }),
  ).toHaveFocus();
  expect(
    screen.getByRole("button", { name: "Confirm these accounts" }),
  ).toBeDisabled();
  expect(
    screen.queryByRole("button", { name: /Create Vercel project/ }),
  ).not.toBeInTheDocument();
  await user.click(screen.getByRole("checkbox", { name: /intended accounts/ }));
  await user.click(
    screen.getByRole("button", { name: "Confirm these accounts" }),
  );
  expect(
    await screen.findByRole("button", { name: /Create Vercel project/ }),
  ).toBeEnabled();
  expect(call.mock.calls.some(([command]) => command === "advance")).toBe(
    false,
  );
  expect(JSON.stringify(localStorage)).not.toContain("synthetic-vercel");
  expect(JSON.stringify(localStorage)).not.toContain("synthetic-supabase");
});

it("stays on Vercel and clears the input if native credential storage fails", async () => {
  const user = userEvent.setup();
  const { bridge, call, fixture } = await accountFixture();
  call.mockImplementation(async (command, args) => {
    if (command === "save_credentials")
      throw new Error("Windows Credential Manager unavailable");
    return fixture.call(command, args);
  });
  render(<App initialBridge={bridge} />);
  await user.type(
    await screen.findByLabelText("Vercel access token"),
    "synthetic-vercel",
  );
  await user.click(screen.getByRole("button", { name: /Save Vercel token/ }));
  expect(await screen.findByRole("alert")).toHaveFocus();
  expect(screen.getByLabelText("Vercel access token")).toHaveValue("");
  expect(
    screen.getByText(/Connecting accounts does not create cloud projects/),
  ).toBeInTheDocument();
  expect(
    screen.queryByText(/Saved resources remain in your account/),
  ).not.toBeInTheDocument();
  expect(
    screen.queryByLabelText("Supabase management token"),
  ).not.toBeInTheDocument();
  expect(
    call.mock.calls.some(([command]) => command === "discover_accounts"),
  ).toBe(false);
});

it("hides stale accounts after a rejected replacement and retries discovery without resending saved tokens", async () => {
  const user = userEvent.setup();
  const { bridge, call, fixture } = await accountFixture();
  render(<App initialBridge={bridge} />);
  await saveVercel(user);
  await user.type(
    screen.getByLabelText("Supabase management token"),
    "synthetic-supabase",
  );
  await user.click(screen.getByRole("button", { name: /Save Supabase token/ }));
  expect(
    screen.getByRole("button", { name: "Confirm these accounts" }),
  ).toBeInTheDocument();
  await user.click(
    within(
      screen.getByRole("navigation", { name: "Connect providers" }),
    ).getByRole("button", { name: /Supabase/ }),
  );
  call.mockImplementation(async (command, args) => {
    if (command === "discover_accounts")
      throw new Error("Synthetic token rejection");
    return fixture.call(command, args);
  });
  await user.type(
    screen.getByLabelText("Supabase management token"),
    "replacement-supabase",
  );
  await user.click(screen.getByRole("button", { name: /Save Supabase token/ }));
  expect(await screen.findByRole("alert")).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: "Confirm these accounts" }),
  ).not.toBeInTheDocument();
  expect(screen.getByLabelText("Supabase management token")).toHaveValue("");
  call.mockImplementation((command, args) => fixture.call(command, args));
  const savesBeforeRetry = call.mock.calls.filter(
    ([command]) => command === "save_credentials",
  ).length;
  await user.click(screen.getByText(/Already saved a Supabase token/));
  await user.click(
    screen.getByRole("button", { name: "Read accounts with saved tokens" }),
  );
  expect(
    await screen.findByRole("button", { name: "Confirm these accounts" }),
  ).toBeDisabled();
  expect(
    call.mock.calls.filter(([command]) => command === "save_credentials"),
  ).toHaveLength(savesBeforeRetry);
});

it("resumes account discovery after reopening using vault tokens without entering or exposing them", async () => {
  const user = userEvent.setup();
  const { bridge, call } = await accountFixture();
  render(<App initialBridge={bridge} />);
  await user.click(await screen.findByText(/Already saved a Vercel token/));
  await user.click(
    screen.getByRole("button", { name: "Use my saved Vercel token" }),
  );
  await user.click(screen.getByText(/Already saved a Supabase token/));
  await user.click(
    screen.getByRole("button", { name: "Read accounts with saved tokens" }),
  );
  expect(
    await screen.findByRole("button", { name: "Confirm these accounts" }),
  ).toBeDisabled();
  expect(
    call.mock.calls.some(
      ([command]) => command === "save_credentials" || command === "advance",
    ),
  ).toBe(false);
  expect(document.querySelector("input[type=password]")).toBeNull();
});

it("replaces only an expired provider token and preserves the current installation step", async () => {
  const user = userEvent.setup();
  const { bridge, call, fixture } = await accountFixture();
  const accounts = await fixture.call<Accounts>("discover_accounts");
  await fixture.call("select_accounts", {
    selection: {
      vercel_user: accounts.vercel_user,
      supabase_user: accounts.supabase_user,
      vercel_account: accounts.vercel[0].id,
      supabase_organization: accounts.supabase[0].id,
      supabase_slug: accounts.supabase[0].slug,
      region: "ap-southeast-2",
      costs_acknowledged: true,
    },
  });
  render(<App initialBridge={bridge} />);
  await user.click(
    await screen.findByRole("button", { name: "Recovery & settings" }),
  );
  await user.click(screen.getByText("Reconnect expired provider access"));
  await user.type(
    screen.getByLabelText("Vercel access token"),
    "replacement-vercel",
  );
  await user.click(
    screen.getByRole("button", {
      name: "Save replacement tokens & check access",
    }),
  );
  expect(call).toHaveBeenCalledWith("save_credentials", {
    vercel: "replacement-vercel",
    supabase: "",
  });
  expect(screen.getByLabelText("Vercel access token")).toHaveValue("");
  expect(screen.getByLabelText("Supabase management token")).toHaveValue("");
  expect(
    call.mock.calls.some(([command]) =>
      ["start_installation", "select_accounts", "advance"].includes(command),
    ),
  ).toBe(false);
  expect(
    screen.getByRole("button", { name: /Create Vercel project/ }),
  ).toBeEnabled();
});

it("requires opting in to testing and leaves demo when the tools are switched off", async () => {
  const user = userEvent.setup();
  render(<App />);
  const toggle = await screen.findByRole("checkbox", {
    name: "Show testing tools",
  });
  expect(toggle).not.toBeChecked();
  expect(
    screen.queryByRole("button", { name: "Explore demo" }),
  ).not.toBeInTheDocument();
  await user.click(toggle);
  await user.click(screen.getByRole("button", { name: "Explore demo" }));
  expect(
    await screen.findByRole("button", { name: /Begin setup/ }),
  ).toBeInTheDocument();
  await user.click(toggle);
  await waitFor(() =>
    expect(screen.queryByLabelText("Demo failure")).not.toBeInTheDocument(),
  );
  expect(
    await screen.findByRole("button", { name: /Read the account guide/ }),
  ).toBeInTheDocument();
});
