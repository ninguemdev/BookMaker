import type { DocumentId } from "@bookmaker/domain";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  AutosaveCoordinator,
  type AutosavePort,
  type AutosaveStatus,
} from "./autosaveCoordinator";

const documentId = "document-1" as DocumentId;

function deferred(): {
  promise: Promise<void>;
  resolve: () => void;
  reject: (error: unknown) => void;
} {
  let resolve!: () => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<void>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

describe("AutosaveCoordinator", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("debounces changes and saves only the latest content", async () => {
    const save = deferred();
    const port: AutosavePort<string> = {
      saveDocument: vi.fn(() => save.promise),
    };
    const statuses: AutosaveStatus[] = [];
    const coordinator = new AutosaveCoordinator(port, {
      debounceMs: 500,
      now: () => new Date("2026-10-05T18:00:00.000Z"),
      onStatusChange: (status) => statuses.push(status),
    });

    coordinator.markDirty(documentId, "first");
    await vi.advanceTimersByTimeAsync(300);
    coordinator.markDirty(documentId, "latest");
    await vi.advanceTimersByTimeAsync(499);
    expect(port.saveDocument).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(port.saveDocument).toHaveBeenCalledWith(documentId, "latest");
    expect(coordinator.getStatus().state).toBe("saving");
    expect(statuses.some((status) => status.state === "saved")).toBe(false);

    save.resolve();
    await save.promise;
    await Promise.resolve();
    expect(coordinator.getStatus()).toEqual({
      state: "saved",
      documentId,
      savedAt: "2026-10-05T18:00:00.000Z",
    });
  });

  it("flushes pending content immediately", async () => {
    const port: AutosavePort<string> = {
      saveDocument: vi.fn(async () => undefined),
    };
    const coordinator = new AutosaveCoordinator(port, { debounceMs: 1_000 });
    coordinator.markDirty(documentId, "content");

    await coordinator.flush(documentId);

    expect(port.saveDocument).toHaveBeenCalledOnce();
    expect(coordinator.getStatus().state).toBe("saved");
  });

  it("saves again when content changes during an active save", async () => {
    const firstSave = deferred();
    const port: AutosavePort<string> = {
      saveDocument: vi
        .fn()
        .mockImplementationOnce(() => firstSave.promise)
        .mockResolvedValueOnce(undefined),
    };
    const coordinator = new AutosaveCoordinator(port, { debounceMs: 100 });
    coordinator.markDirty(documentId, "first");
    await vi.advanceTimersByTimeAsync(100);

    coordinator.markDirty(documentId, "second");
    firstSave.resolve();
    await firstSave.promise;
    await vi.advanceTimersByTimeAsync(100);

    expect(port.saveDocument).toHaveBeenNthCalledWith(1, documentId, "first");
    expect(port.saveDocument).toHaveBeenNthCalledWith(2, documentId, "second");
    expect(coordinator.getStatus().state).toBe("saved");
  });

  it("keeps flush waiting when content changes during an active save", async () => {
    const firstSave = deferred();
    const port: AutosavePort<string> = {
      saveDocument: vi
        .fn()
        .mockImplementationOnce(() => firstSave.promise)
        .mockResolvedValueOnce(undefined),
    };
    const coordinator = new AutosaveCoordinator(port, { debounceMs: 100 });
    coordinator.markDirty(documentId, "first");
    await vi.advanceTimersByTimeAsync(100);

    const flush = coordinator.flush(documentId);
    coordinator.markDirty(documentId, "second");
    firstSave.resolve();
    await flush;

    expect(port.saveDocument).toHaveBeenNthCalledWith(2, documentId, "second");
    expect(coordinator.getStatus().state).toBe("saved");
  });

  it("keeps failed content pending so an explicit flush can retry", async () => {
    const saveError = new Error("disk unavailable");
    const port: AutosavePort<string> = {
      saveDocument: vi
        .fn()
        .mockRejectedValueOnce(saveError)
        .mockResolvedValueOnce(undefined),
    };
    const coordinator = new AutosaveCoordinator(port, { debounceMs: 100 });
    coordinator.markDirty(documentId, "content");

    await vi.advanceTimersByTimeAsync(100);
    expect(coordinator.getStatus()).toEqual({
      state: "error",
      documentId,
      error: saveError,
    });

    await coordinator.flush(documentId);
    expect(port.saveDocument).toHaveBeenCalledTimes(2);
    expect(coordinator.getStatus().state).toBe("saved");
  });
});
