import { afterEach, expect, it, vi } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import "@testing-library/jest-dom/vitest";
import { GoogleScopes } from "../src/GoogleScopes";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

it("copies only the supplied release scopes and displays both locally bundled references", async () => {
  const user = userEvent.setup();
  const copy = vi
    .spyOn(navigator.clipboard, "writeText")
    .mockResolvedValue(undefined);
  const scopes = [
    "https://www.googleapis.com/auth/userinfo.email",
    "https://www.googleapis.com/auth/youtube.force-ssl",
  ];
  render(<GoogleScopes scopes={scopes} busy={false} />);
  await user.click(screen.getByRole("button", { name: "Copy scope list" }));
  expect(copy).toHaveBeenCalledWith(scopes.join("\n"));
  const images = screen.getAllByRole("img");
  expect(images.every((image) => image.closest("details") === null)).toBe(true);
  expect(images).toHaveLength(2);
  expect(images[0]).toHaveAttribute(
    "src",
    expect.stringContaining("google-identity-scopes.png"),
  );
  expect(images[1]).toHaveAttribute(
    "src",
    expect.stringContaining("google-youtube-scope.png"),
  );
  expect(images[0]).toHaveAccessibleName(/userinfo.email and userinfo.profile/);
  expect(images[1]).toHaveAccessibleName(/youtube.force-ssl/);
});
