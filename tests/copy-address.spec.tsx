import { afterEach, expect, it, vi } from "vitest";
import {
  cleanup,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { CopyAddress } from "../src/CopyAddress";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("acknowledges each address beside its own button only after clipboard success", async () => {
  const user = userEvent.setup();
  let finish: () => void = () => {};
  const write = vi
    .spyOn(navigator.clipboard, "writeText")
    .mockImplementationOnce(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    )
    .mockResolvedValue(undefined);
  const submit = vi.fn((e) => e.preventDefault());
  render(
    <form onSubmit={submit}>
      <CopyAddress
        label="Origin"
        value="https://example.test"
        buttonLabel="Copy origin"
        disabled={false}
      />
      <CopyAddress
        label="Callback"
        value="https://example.test/api/auth"
        buttonLabel="Copy callback"
        disabled={false}
      />
    </form>,
  );
  const origin = within(screen.getByRole("group", { name: "Origin" }));
  const callback = within(screen.getByRole("group", { name: "Callback" }));
  await user.click(origin.getByRole("button"));
  expect(write).toHaveBeenLastCalledWith("https://example.test");
  expect(origin.getByRole("button", { name: "Copying…" })).toBeDisabled();
  expect(origin.getByRole("status")).toBeEmptyDOMElement();
  finish();
  await waitFor(() =>
    expect(origin.getByRole("button")).toHaveTextContent("Copied!"),
  );
  expect(origin.getByRole("status")).toHaveTextContent("Origin copied.");
  expect(callback.getByRole("button")).toHaveTextContent("Copy callback");
  await user.click(callback.getByRole("button"));
  expect(write).toHaveBeenLastCalledWith("https://example.test/api/auth");
  expect(callback.getByRole("status")).toHaveTextContent("Callback copied.");
  expect(submit).not.toHaveBeenCalled();
});

it("replaces stale success with an adjacent manual-copy fallback when clipboard access fails", async () => {
  const user = userEvent.setup();
  const write = vi
    .spyOn(navigator.clipboard, "writeText")
    .mockResolvedValueOnce(undefined)
    .mockRejectedValueOnce(new Error("Clipboard unavailable"));
  render(
    <CopyAddress
      label="Origin"
      value="https://example.test"
      buttonLabel="Copy origin"
      disabled={false}
    />,
  );
  await user.click(screen.getByRole("button"));
  expect(screen.getByRole("status")).toHaveTextContent("Origin copied.");
  await user.click(screen.getByRole("button"));
  expect(write).toHaveBeenCalledTimes(2);
  expect(screen.getByRole("status")).toHaveTextContent(
    "Select the address and press Ctrl+C",
  );
  expect(screen.getByRole("button")).toHaveTextContent("Copy origin");
  expect(screen.getByText("https://example.test")).toBeInTheDocument();
});
