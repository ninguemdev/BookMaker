import {
  debug,
  error,
  info,
  warn,
  type LogOptions,
} from "@tauri-apps/plugin-log";
import { afterEach, describe, expect, it, vi } from "vitest";

import { createLogger } from "./logger";

vi.mock("@tauri-apps/plugin-log", () => ({
  debug: vi.fn(() => Promise.resolve()),
  error: vi.fn(() => Promise.resolve()),
  info: vi.fn(() => Promise.resolve()),
  warn: vi.fn(() => Promise.resolve()),
}));

const expectedOptions: LogOptions = {
  keyValues: { category: "project" },
};

describe("createLogger", () => {
  afterEach(() => {
    vi.clearAllMocks();
  });

  it("adds the category to every log level", async () => {
    const logger = createLogger("project");

    await logger.debug("project_open_started");
    await logger.info("project_opened");
    await logger.warn("project_open_slow");
    await logger.error("project_open_failed");

    expect(debug).toHaveBeenCalledWith("project_open_started", expectedOptions);
    expect(info).toHaveBeenCalledWith("project_opened", expectedOptions);
    expect(warn).toHaveBeenCalledWith("project_open_slow", expectedOptions);
    expect(error).toHaveBeenCalledWith("project_open_failed", expectedOptions);
  });
});
