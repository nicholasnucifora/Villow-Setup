import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { AppUpdates } from "../src/AppUpdates";
import type { Installation, Snapshot } from "../src/types";
afterEach(cleanup);
const state = {
  step: "complete",
  read_only: false,
  app_version: "0.1.2",
} as Installation;
const result: NonNullable<Snapshot["app_update"]> = {
  status: "current",
  app_version: "0.1.2",
  minimum_manager: "0.1.3",
  message: "Villow 0.1.2 is the latest approved release.",
  notes: "",
};

it("checks using one bounded command and never offers a write from availability alone", async () => {
  const action = vi.fn().mockResolvedValue(true);
  render(<AppUpdates s={state} result={result} busy={false} action={action} />);
  fireEvent.click(screen.getByRole("button", { name: "Check for updates" }));
  await waitFor(() =>
    expect(action).toHaveBeenCalledExactlyOnceWith("check_app_update"),
  );
  expect(screen.queryByRole("button", { name: /Update my app/ })).toBeNull();
  expect(
    screen.getByText("Villow 0.1.2 is the latest approved release."),
  ).toBeInTheDocument();
});

it("shows visible waiting and hides an old success after a failed check", async () => {
  let resolve!: (ok: boolean) => void;
  const action = vi.fn(
    () =>
      new Promise<boolean>((done) => {
        resolve = done;
      }),
  );
  render(<AppUpdates s={state} result={result} busy={false} action={action} />);
  fireEvent.click(screen.getByRole("button", { name: "Check for updates" }));
  expect(
    screen.getByRole("button", { name: "Checking for updates…" }),
  ).toBeDisabled();
  expect(screen.queryByText(result.message)).toBeNull();
  resolve(false);
  await screen.findByText(/The update check could not finish/);
  expect(screen.queryByText(result.message)).toBeNull();
});

it("explains incompatible releases and renders release notes as text", () => {
  const attack = '<img src=x onerror="steal()">';
  const { container, rerender } = render(
    <AppUpdates
      s={state}
      result={{
        ...result,
        status: "manager_required",
        minimum_manager: "0.2.0",
        notes: attack,
      }}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(
    screen.getByText(/This release needs Villow Setup 0.2.0/),
  ).toBeInTheDocument();
  expect(container.querySelector("img")).toBeNull();
  rerender(
    <AppUpdates
      s={state}
      result={{ ...result, status: "unsupported" }}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(
    screen.getByText(/do not run fresh database setup/),
  ).toBeInTheDocument();
});

it("does not show completed-installation actions on read-only or unfinished setups", () => {
  const { rerender } = render(
    <AppUpdates
      s={{ ...state, read_only: true }}
      result={null}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(screen.queryByRole("button")).toBeNull();
  rerender(
    <AppUpdates
      s={{ ...state, step: "health" }}
      result={null}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(screen.queryByRole("button")).toBeNull();
});

it("starts only the authenticated offered digest and shows recovery progress", async () => {
  let resolve!: (ok: boolean) => void;
  const action = vi.fn(
    () =>
      new Promise<boolean>((done) => {
        resolve = done;
      }),
  );
  render(
    <AppUpdates
      s={state}
      result={{
        ...result,
        status: "available",
        digest: "verified-target",
        app_version: "0.2.0",
      }}
      busy={false}
      action={action}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: "Update my app" }));
  expect(action).toHaveBeenCalledExactlyOnceWith("update_app", {
    digest: "verified-target",
  });
  expect(screen.getByLabelText("Update progress")).toHaveAttribute(
    "aria-busy",
    "true",
  );
  expect(
    screen.queryByRole("button", { name: "Check for updates" }),
  ).toBeNull();
  resolve(false);
  await screen.findByRole("button", { name: "Update my app" });
});

it("resumes the saved release after reopening and never follows a newer offer", async () => {
  const action = vi.fn().mockResolvedValue(false);
  const pending = {
    from: "old",
    to: "saved-target",
    phase: "verify",
    deployment_status: "building",
  } as NonNullable<Installation["app_update"]>;
  render(
    <AppUpdates
      s={{ ...state, app_update: pending }}
      result={{ ...result, status: "available", digest: "different-target" }}
      busy={false}
      action={action}
    />,
  );
  expect(screen.getByText("Your update is paused")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: "Update my app" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Resume update" }));
  await waitFor(() =>
    expect(action).toHaveBeenCalledExactlyOnceWith("update_app", {
      digest: "saved-target",
    }),
  );
});

it("distinguishes verified update completion from pending cleanup", () => {
  const complete = {
    phase: "complete",
    backup: { managed: true, removed_at: null },
  } as NonNullable<Installation["app_update"]>;
  const { rerender } = render(
    <AppUpdates
      s={{ ...state, app_update: complete }}
      result={null}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(
    screen.getByText(/retry removing its temporary recovery copy/),
  ).toBeInTheDocument();
  rerender(
    <AppUpdates
      s={{
        ...state,
        app_update: {
          ...complete,
          backup: { ...complete.backup!, removed_at: "today" },
        },
      }}
      result={null}
      busy={false}
      action={vi.fn()}
    />,
  );
  expect(
    screen.getByText(/Setup removed the temporary recovery copy/),
  ).toBeInTheDocument();
});
