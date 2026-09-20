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

it("keeps provider progress when going back and requires all three readiness confirmations", async () => {
  const user = userEvent.setup();
  render(<App initialBridge={new DemoBridge()} />);
  await user.click(
    await screen.findByRole("button", { name: /Prepare my accounts/ }),
  );
  const progress = () =>
    within(screen.getByRole("navigation", { name: "Account preparation" }));
  await user.click(progress().getByRole("button", { name: /Google Cloud/ }));
  await user.click(
    screen.getByRole("button", { name: /Google Cloud is ready/ }),
  );
  expect(
    screen.getByRole("heading", { name: "Vercel", level: 1 }),
  ).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: /Start my setup/ }),
  ).not.toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: /Vercel is ready/ }));
  await user.click(screen.getByRole("button", { name: "Back to Vercel" }));
  expect(
    progress().getByRole("button", { name: /Vercel Ready/ }),
  ).toHaveAttribute("aria-current", "step");
  await user.click(progress().getByRole("button", { name: /Supabase/ }));
  await user.click(screen.getByRole("button", { name: /Supabase is ready/ }));
  await user.click(
    screen.getByRole("button", { name: /Google Cloud is ready/ }),
  );
  expect(
    await screen.findByRole("button", { name: /Start my setup/ }),
  ).toBeInTheDocument();
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

it("collects both real-mode tokens, clears them and requires account confirmation before creation", async () => {
  const user = userEvent.setup();
  const { bridge, call } = await accountFixture();
  render(<App initialBridge={bridge} />);
  const save = await screen.findByRole("button", {
    name: "Save tokens & read my accounts",
  });
  const vercel = screen.getByLabelText("Vercel access token");
  const supabase = screen.getByLabelText("Supabase management token");
  expect(
    screen.queryByRole("button", { name: "Load demo accounts" }),
  ).not.toBeInTheDocument();
  expect(save).toBeDisabled();
  await user.type(vercel, "synthetic-vercel");
  expect(save).toBeDisabled();
  await user.type(supabase, "synthetic-supabase");
  await user.click(save);
  expect(call).toHaveBeenCalledWith("save_credentials", {
    vercel: "synthetic-vercel",
    supabase: "synthetic-supabase",
  });
  expect(vercel).toHaveValue("");
  expect(supabase).toHaveValue("");
  expect(
    screen.getByRole("button", { name: "Confirm these accounts" }),
  ).toBeDisabled();
  expect(
    screen.queryByRole("button", { name: "Create Vercel project" }),
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

it("clears failed credentials and hides stale discovered accounts until a successful retry", async () => {
  const user = userEvent.setup();
  const { bridge, call, fixture } = await accountFixture();
  render(<App initialBridge={bridge} />);
  const save = await screen.findByRole("button", {
    name: "Save tokens & read my accounts",
  });
  const enter = async () => {
    await user.type(
      screen.getByLabelText("Vercel access token"),
      "synthetic-vercel",
    );
    await user.type(
      screen.getByLabelText("Supabase management token"),
      "synthetic-supabase",
    );
  };
  await enter();
  await user.click(save);
  expect(
    screen.getByRole("button", { name: "Confirm these accounts" }),
  ).toBeInTheDocument();
  call.mockImplementation(async (command, args) => {
    if (command === "discover_accounts")
      throw new Error("Synthetic token rejection");
    return fixture.call(command, args);
  });
  await enter();
  await user.click(save);
  expect(await screen.findByRole("alert")).toHaveFocus();
  expect(
    screen.queryByRole("button", { name: "Confirm these accounts" }),
  ).not.toBeInTheDocument();
  expect(screen.getByLabelText("Vercel access token")).toHaveValue("");
  expect(screen.getByLabelText("Supabase management token")).toHaveValue("");
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
    await screen.findByRole("button", { name: /Prepare my accounts/ }),
  ).toBeInTheDocument();
  await user.click(toggle);
  await waitFor(() =>
    expect(screen.queryByLabelText("Demo failure")).not.toBeInTheDocument(),
  );
  expect(
    await screen.findByRole("button", { name: /Read the account guide/ }),
  ).toBeInTheDocument();
});
