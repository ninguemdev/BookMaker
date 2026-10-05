import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { AppErrorBoundary } from "./AppErrorBoundary";
import { reportRenderError } from "./errorHandling";

vi.mock("./errorHandling", () => ({
  reportRenderError: vi.fn(),
}));

function BrokenContent(): never {
  throw new Error("render failed");
}

describe("AppErrorBoundary", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("shows a friendly recovery screen without exposing technical details", () => {
    const reloadApplication = vi.fn();
    vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(
      <AppErrorBoundary reloadApplication={reloadApplication}>
        <BrokenContent />
      </AppErrorBoundary>,
    );

    expect(
      screen.getByRole("heading", { name: "Algo deu errado" }),
    ).toBeInTheDocument();
    expect(screen.getByText("Erro BM-UNEXPECTED-001")).toBeInTheDocument();
    expect(screen.queryByText("render failed")).not.toBeInTheDocument();
    expect(reportRenderError).toHaveBeenCalledOnce();

    fireEvent.click(
      screen.getByRole("button", { name: "Recarregar aplicação" }),
    );
    expect(reloadApplication).toHaveBeenCalledOnce();
  });
});
