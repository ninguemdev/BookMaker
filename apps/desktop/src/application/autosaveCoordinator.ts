import type { DocumentId } from "@bookmaker/domain";

export interface AutosavePort<Content> {
  saveDocument(documentId: DocumentId, content: Content): Promise<void>;
}

export type AutosaveStatus =
  | { state: "idle" }
  | { state: "dirty"; documentId: DocumentId }
  | { state: "saving"; documentId: DocumentId }
  | { state: "saved"; documentId: DocumentId; savedAt: string }
  | { state: "error"; documentId: DocumentId; error: unknown };

export interface AutosaveCoordinatorOptions {
  debounceMs?: number;
  onStatusChange?: (status: AutosaveStatus) => void;
  now?: () => Date;
}

interface PendingDocument<Content> {
  content: Content;
  revision: number;
  timer: ReturnType<typeof setTimeout> | undefined;
  inFlight: Promise<void> | undefined;
}

const DEFAULT_DEBOUNCE_MS = 750;

export class AutosaveCoordinator<Content> {
  private readonly pendingDocuments = new Map<
    DocumentId,
    PendingDocument<Content>
  >();

  private readonly debounceMs: number;
  private readonly now: () => Date;
  private status: AutosaveStatus = { state: "idle" };

  public constructor(
    private readonly port: AutosavePort<Content>,
    private readonly options: AutosaveCoordinatorOptions = {},
  ) {
    this.debounceMs = options.debounceMs ?? DEFAULT_DEBOUNCE_MS;
    this.now = options.now ?? (() => new Date());

    if (!Number.isFinite(this.debounceMs) || this.debounceMs < 0) {
      throw new RangeError("debounceMs must be a non-negative finite number");
    }
  }

  public markDirty(documentId: DocumentId, content: Content): void {
    const pending = this.pendingDocuments.get(documentId) ?? {
      content,
      revision: 0,
      timer: undefined,
      inFlight: undefined,
    };

    pending.content = content;
    pending.revision += 1;
    this.pendingDocuments.set(documentId, pending);
    this.updateStatus({ state: "dirty", documentId });

    if (pending.inFlight === undefined) {
      this.schedule(documentId, pending);
    }
  }

  public getStatus(): AutosaveStatus {
    return this.status;
  }

  public hasPendingChanges(): boolean {
    return this.pendingDocuments.size > 0;
  }

  public async flush(documentId: DocumentId): Promise<void> {
    while (this.pendingDocuments.has(documentId)) {
      const pending = this.pendingDocuments.get(documentId);
      if (pending === undefined) {
        return;
      }
      this.cancelTimer(pending);
      await this.savePendingDocument(documentId, pending);
    }
  }

  public async flushAll(): Promise<void> {
    const documentIds = [...this.pendingDocuments.keys()];
    await Promise.all(documentIds.map((documentId) => this.flush(documentId)));
  }

  public dispose(): void {
    for (const pending of this.pendingDocuments.values()) {
      this.cancelTimer(pending);
    }
  }

  private schedule(
    documentId: DocumentId,
    pending: PendingDocument<Content>,
  ): void {
    this.cancelTimer(pending);
    pending.timer = setTimeout(() => {
      pending.timer = undefined;
      void this.savePendingDocument(documentId, pending).catch(() => {
        // The error state is published by savePendingDocument. A later edit or
        // explicit flush retries the same latest content.
      });
    }, this.debounceMs);
  }

  private savePendingDocument(
    documentId: DocumentId,
    pending: PendingDocument<Content>,
  ): Promise<void> {
    if (pending.inFlight !== undefined) {
      return pending.inFlight;
    }

    const revision = pending.revision;
    const content = pending.content;
    this.updateStatus({ state: "saving", documentId });

    const save = this.performSave(documentId, pending, revision, content);
    pending.inFlight = save;
    return save;
  }

  private async performSave(
    documentId: DocumentId,
    pending: PendingDocument<Content>,
    revision: number,
    content: Content,
  ): Promise<void> {
    try {
      await this.port.saveDocument(documentId, content);
    } catch (error: unknown) {
      pending.inFlight = undefined;
      this.updateStatus({ state: "error", documentId, error });
      throw error;
    }

    pending.inFlight = undefined;
    const current = this.pendingDocuments.get(documentId);
    if (current === undefined) {
      return;
    }

    if (current.revision === revision) {
      this.pendingDocuments.delete(documentId);
      this.updateStatus({
        state: "saved",
        documentId,
        savedAt: this.now().toISOString(),
      });
      return;
    }

    this.updateStatus({ state: "dirty", documentId });
    this.schedule(documentId, current);
  }

  private cancelTimer(pending: PendingDocument<Content>): void {
    if (pending.timer !== undefined) {
      clearTimeout(pending.timer);
      pending.timer = undefined;
    }
  }

  private updateStatus(status: AutosaveStatus): void {
    this.status = status;
    this.options.onStatusChange?.(status);
  }
}
