import { afterEach, describe, expect, it, vi } from "vitest";

import { installGlobalErrorHandlers } from "./errorHandling";

const writeError = vi.hoisted(() => vi.fn(() => Promise.resolve()));

vi.mock("./logger", () => ({
  createLogger: () => ({ error: writeError }),
}));

describe("installGlobalErrorHandlers", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("reports global errors with stable event names and removes its listeners", () => {
    const removeGlobalErrorHandlers = installGlobalErrorHandlers();

    window.dispatchEvent(new Event("error"));
    window.dispatchEvent(new Event("unhandledrejection"));

    expect(writeError).toHaveBeenNthCalledWith(1, "uncaught_error");
    expect(writeError).toHaveBeenNthCalledWith(
      2,
      "unhandled_promise_rejection",
    );

    removeGlobalErrorHandlers();
    window.dispatchEvent(new Event("error"));

    expect(writeError).toHaveBeenCalledTimes(2);
  });
});
