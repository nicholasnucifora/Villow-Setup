import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
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
it("requires backup acknowledgment and resets it when the authenticated offer changes", () => {
  const action = vi.fn().mockResolvedValue(false);
  const open = vi.fn();
  const ui = render(
    <InstalledRepair
      s={state()}
      offer={offer}
      message=""
      busy={false}
      action={action}
      open={open}
    />,
  );
  const apply = screen.getByRole("button", { name: /Apply repair/ });
  expect(apply).toBeDisabled();
  expect(
    screen.getByText(/cannot independently check your backup/),
  ).toBeVisible();
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(apply);
  expect(action).toHaveBeenCalledWith("apply_installed_repair", {
    digest: "new-digest",
    backupConfirmed: true,
  });
  ui.rerender(
    <InstalledRepair
      s={state()}
      offer={{ ...offer, digest: "different" }}
      message=""
      busy={false}
      action={action}
      open={open}
    />,
  );
  expect(screen.getByRole("checkbox")).not.toBeChecked();
  expect(screen.getByRole("button", { name: /Apply repair/ })).toBeDisabled();
});
it("reopening a repair waits for explicit resume and keeps the original destination", async () => {
  const action = vi.fn().mockResolvedValue(false);
  render(
    <InstalledRepair
      s={state(pending)}
      offer={{ ...offer, digest: "newer-channel-release" }}
      message=""
      busy={false}
      action={action}
      open={vi.fn()}
    />,
  );
  expect(action).not.toHaveBeenCalled();
  expect(screen.queryByRole("checkbox")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Resume repair" }));
  await waitFor(() => expect(action).toHaveBeenCalledTimes(1));
  expect(action).toHaveBeenCalledWith("apply_installed_repair", {
    digest: "new-digest",
    backupConfirmed: false,
  });
});
it("pauses automatic progress on failure and exposes build recovery with no create button", async () => {
  const action = vi.fn().mockResolvedValue(false);
  render(
    <InstalledRepair
      s={state({
        ...pending,
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
      name: /Apply repair|Build my app|Prepare my database/,
    }),
  ).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Resume repair" }));
  await waitFor(() => expect(action).toHaveBeenCalledTimes(1));
  expect(
    screen.getByRole("button", { name: /Open Vercel build logs/ }),
  ).toBeEnabled();
});
it("shows honest no-repair availability without enabling database writes", async () => {
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
  expect(screen.queryByRole("button", { name: /Apply repair/ })).toBeNull();
});
