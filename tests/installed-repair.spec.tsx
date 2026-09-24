import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { InstalledRepair } from "../src/InstalledRepair";
import type { Installation, RepairIntent } from "../src/types";
afterEach(() => {
  cleanup();
  vi.useRealTimers();
});
const offer = { digest: "new-digest", app_version: "0.1.2" };
const pending: RepairIntent = {
  from: "old",
  to: "new-digest",
  repair_id: "villow-installed-159",
  operation_id: "repair-op",
  previous_operation_id: "original-op",
  previous_deployment_id: "old-deployment",
  backup_confirmed_at: "2026-09-22T00:00:00Z",
  phase: "database",
  deployment_id: null,
  deployment_status: null,
};
const state = (repair?: RepairIntent) =>
  ({ installed_repair: repair ?? null }) as Installation;
const backup = {
  managed: true,
  path: "native-owned-path",
  sha256: "hash",
  bytes: 100,
  captured_at: "2026-09-23T00:00:00Z",
  installation_id: "instance",
  operation_id: "repair-op",
  from: "old",
  to: "new-digest",
};
it("starts automatic protection with only the release ID and stops on backup failure", async () => {
  vi.useFakeTimers();
  const action = vi.fn().mockResolvedValue(false);
  render(
    <InstalledRepair
      s={state()}
      offer={offer}
      message=""
      busy={false}
      action={action}
      open={vi.fn()}
    />,
  );
  expect(screen.queryByLabelText(/password/i)).toBeNull();
  expect(
    screen.getByText(/There is no password to create or file to manage/),
  ).toBeVisible();
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: /Repair my app/ }));
  });
  await act(async () => {
    await vi.advanceTimersByTimeAsync(60_000);
  });
  expect(action).toHaveBeenCalledExactlyOnceWith("backup_and_repair", {
    digest: "new-digest",
  });
  expect(screen.queryByRole("button", { name: "Resume repair" })).toBeNull();
  expect(screen.getByRole("button", { name: /Repair my app/ })).toBeEnabled();
});
it("disables starting another repair while protection is being created", () => {
  const action = vi.fn();
  render(
    <InstalledRepair
      s={state()}
      offer={offer}
      message=""
      busy={true}
      action={action}
      open={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: /Repair my app/ }));
  expect(action).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: /Repair my app/ })).toBeDisabled();
});
it("reopening waits for explicit resume and keeps the original destination", async () => {
  const action = vi.fn().mockResolvedValue(false);
  render(
    <InstalledRepair
      s={state({ ...pending, backup })}
      offer={{ ...offer, digest: "newer-channel-release" }}
      message=""
      busy={false}
      action={action}
      open={vi.fn()}
    />,
  );
  expect(action).not.toHaveBeenCalled();
  expect(screen.getByText(/You do not need to manage it/)).toBeVisible();
  expect(screen.queryByText("native-owned-path")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Resume repair" }));
  await waitFor(() =>
    expect(action).toHaveBeenCalledExactlyOnceWith("apply_installed_repair", {
      digest: "new-digest",
    }),
  );
});
it("pauses on build failure and retains recovery with no create button", async () => {
  const action = vi.fn().mockResolvedValue(false);
  render(
    <InstalledRepair
      s={state({
        ...pending,
        backup,
        phase: "verify",
        deployment_id: "new-deploy",
        deployment_status: "failed",
      })}
      offer={null}
      message=""
      busy={false}
      action={action}
      open={vi.fn()}
    />,
  );
  expect(
    screen.getByText(/Vercel reported a build or address problem/),
  ).toBeVisible();
  expect(
    screen.queryByRole("button", {
      name: /Repair my app|Build my app|Prepare my database/,
    }),
  ).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Resume repair" }));
  await waitFor(() => expect(action).toHaveBeenCalledTimes(1));
  expect(
    screen.getByRole("button", { name: /Open Vercel build logs/ }),
  ).toBeEnabled();
});
it("reports cleanup accurately and retains older portable backups", () => {
  const props = {
    offer: null,
    message: "",
    busy: false,
    action: vi.fn(),
    open: vi.fn(),
  };
  const ui = render(
    <InstalledRepair
      {...props}
      s={state({ ...pending, backup, phase: "complete" })}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    /retry removing it when you reopen/,
  );
  ui.rerender(
    <InstalledRepair
      {...props}
      s={state({
        ...pending,
        backup: { ...backup, removed_at: "2026-09-23T01:00:00Z" },
        phase: "complete",
      })}
    />,
  );
  expect(screen.getByRole("status")).toHaveTextContent(
    /removed the temporary recovery copy automatically/,
  );
  ui.rerender(
    <InstalledRepair
      {...props}
      s={state({ ...pending, backup: { ...backup, managed: false } })}
    />,
  );
  expect(screen.getByText(/Keep that file and its password/)).toBeVisible();
  expect(screen.getByText("native-owned-path")).toBeVisible();
});
it("does not offer an unfinished-install repair after a fresh installation succeeds", () => {
  render(
    <InstalledRepair
      s={{ ...state(), step: "complete" }}
      offer={null}
      message=""
      busy={false}
      action={vi.fn()}
      open={vi.fn()}
    />,
  );
  expect(
    screen.queryByRole("button", { name: /Check for an app repair/ }),
  ).toBeNull();
});
it("shows no repair availability without enabling writes", async () => {
  const action = vi.fn().mockResolvedValue(true);
  render(
    <InstalledRepair
      s={state()}
      offer={null}
      message="No signed repair is available"
      busy={false}
      action={action}
      open={vi.fn()}
    />,
  );
  fireEvent.click(
    screen.getByRole("button", { name: /Check for an app repair/ }),
  );
  expect(await screen.findByRole("status")).toHaveTextContent(
    "No signed repair is available",
  );
  expect(action).toHaveBeenCalledWith("check_installed_repair");
  expect(screen.queryByRole("button", { name: /Repair my app/ })).toBeNull();
});
