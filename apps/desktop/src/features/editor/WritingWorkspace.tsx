import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { DocumentId } from "@bookmaker/domain";
import type { EditorContent as EditorJsonContent } from "@bookmaker/editor-core";

import {
  AutosaveCoordinator,
  type AutosaveStatus,
} from "../../application/autosaveCoordinator";
import { searchProject } from "../../application/projectSearch";
import type {
  ProjectSearchInput,
  ProjectSearchMatch,
  ProjectSearchResults,
} from "../../application/projectSearch";
import {
  nativeWritingProjectGateway,
  type LoadedDocumentContent,
  type WritingDocument,
  type WritingProject,
  type WritingProjectGateway,
} from "../../application/writingProject";
import type { DocumentTreeNode } from "../manuscript/DocumentTree";
import { WritingWorkspaceView } from "./WritingWorkspaceView";

import "./WritingWorkspace.css";

interface WritingWorkspaceProps {
  project: WritingProject;
  gateway?: WritingProjectGateway;
  onCloseProject?: () => void;
  search?: (input: ProjectSearchInput) => Promise<ProjectSearchResults>;
  autosaveDebounceMs?: number;
}

type DocumentLoadState =
  | { state: "loading"; documentId: DocumentId }
  | { state: "ready"; document: LoadedDocumentContent }
  | { state: "error"; documentId: DocumentId; message: string }
  | { state: "empty" };

function errorMessage(error: unknown, fallback: string): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return fallback;
}

function buildDocumentTree(
  documents: readonly WritingDocument[],
): DocumentTreeNode[] {
  const nodes = new Map<DocumentId, DocumentTreeNode>();
  const sortedDocuments = [...documents].sort(
    (left, right) =>
      left.position - right.position ||
      left.documentId.localeCompare(right.documentId),
  );

  for (const document of sortedDocuments) {
    nodes.set(document.documentId, {
      id: document.documentId,
      title: document.title,
      children: [],
    });
  }

  const roots: DocumentTreeNode[] = [];
  for (const document of sortedDocuments) {
    const node = nodes.get(document.documentId)!;
    const parent = document.parentId ? nodes.get(document.parentId) : undefined;
    if (parent?.children) {
      (parent.children as DocumentTreeNode[]).push(node);
    } else {
      roots.push(node);
    }
  }
  return roots;
}

function firstDocumentId(
  tree: readonly DocumentTreeNode[],
): DocumentId | undefined {
  return tree[0]?.id as DocumentId | undefined;
}

export function WritingWorkspace({
  project,
  gateway = nativeWritingProjectGateway,
  onCloseProject,
  search = searchProject,
  autosaveDebounceMs,
}: WritingWorkspaceProps) {
  const tree = useMemo(
    () => buildDocumentTree(project.documents),
    [project.documents],
  );
  const initialDocumentId = firstDocumentId(tree);
  const [loadState, setLoadState] = useState<DocumentLoadState>(() =>
    initialDocumentId
      ? { state: "loading", documentId: initialDocumentId }
      : { state: "empty" },
  );
  const [autosaveStatus, setAutosaveStatus] = useState<AutosaveStatus>({
    state: "idle",
  });
  const [isClosing, setIsClosing] = useState(false);
  const loadVersion = useRef(0);
  const [navigationMatch, setNavigationMatch] =
    useState<ProjectSearchMatch | null>(null);
  const coordinator = useMemo(
    () =>
      new AutosaveCoordinator<EditorJsonContent>(
        {
          saveDocument: async (documentId, content) => {
            await gateway.saveDocument(project.path, documentId, content);
          },
        },
        { debounceMs: autosaveDebounceMs, onStatusChange: setAutosaveStatus },
      ),
    [autosaveDebounceMs, gateway, project.path],
  );

  const loadDocument = useCallback(
    async (documentId: DocumentId): Promise<boolean> => {
      const currentVersion = ++loadVersion.current;
      try {
        const document = await gateway.loadDocument(project.path, documentId);
        if (loadVersion.current === currentVersion) {
          setAutosaveStatus({ state: "idle" });
          setLoadState({ state: "ready", document });
        }
        return true;
      } catch (error: unknown) {
        if (loadVersion.current === currentVersion) {
          setLoadState({
            state: "error",
            documentId,
            message: errorMessage(
              error,
              "Não foi possível carregar o documento.",
            ),
          });
        }
        return false;
      }
    },
    [gateway, project.path],
  );

  useEffect(() => {
    if (!initialDocumentId) return;
    const currentVersion = ++loadVersion.current;
    void gateway
      .loadDocument(project.path, initialDocumentId)
      .then((document) => {
        if (loadVersion.current === currentVersion) {
          setAutosaveStatus({ state: "idle" });
          setLoadState({ state: "ready", document });
        }
      })
      .catch((error: unknown) => {
        if (loadVersion.current === currentVersion) {
          setLoadState({
            state: "error",
            documentId: initialDocumentId,
            message: errorMessage(
              error,
              "Não foi possível carregar o documento.",
            ),
          });
        }
      });
  }, [gateway, initialDocumentId, project.path]);

  useEffect(
    () => () => {
      loadVersion.current += 1;
      coordinator.dispose();
    },
    [coordinator],
  );

  useEffect(() => {
    const warnBeforeBrowserClose = (event: BeforeUnloadEvent): void => {
      if (coordinator.hasPendingChanges()) {
        event.preventDefault();
      }
    };
    window.addEventListener("beforeunload", warnBeforeBrowserClose);
    return () =>
      window.removeEventListener("beforeunload", warnBeforeBrowserClose);
  }, [coordinator]);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) {
      return;
    }

    let disposed = false;
    let unlisten: (() => void) | undefined;
    void import("@tauri-apps/api/window").then(async ({ getCurrentWindow }) => {
      if (disposed) return;
      const applicationWindow = getCurrentWindow();
      unlisten = await applicationWindow.onCloseRequested(async (event) => {
        if (!coordinator.hasPendingChanges()) return;
        event.preventDefault();
        try {
          await coordinator.flushAll();
          await applicationWindow.destroy();
        } catch {
          // The coordinator publishes the visible error state and keeps the
          // latest content pending for an explicit retry.
        }
      });
    });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [coordinator]);

  const changeDocument = useCallback(
    async (
      documentId: DocumentId,
      match?: ProjectSearchMatch,
    ): Promise<void> => {
      if (
        loadState.state === "ready" &&
        loadState.document.documentId === documentId
      ) {
        return;
      }

      if (loadState.state === "ready") {
        try {
          await coordinator.flush(loadState.document.documentId);
        } catch {
          setNavigationMatch(null);
          return;
        }
      }
      setNavigationMatch(match ?? null);
      setLoadState({ state: "loading", documentId });
      await loadDocument(documentId);
    },
    [coordinator, loadDocument, loadState],
  );

  const retrySave = useCallback(async (): Promise<void> => {
    if (autosaveStatus.state !== "error") return;
    try {
      await coordinator.flush(autosaveStatus.documentId);
    } catch {
      // The error remains visible and the latest content remains pending.
    }
  }, [autosaveStatus, coordinator]);

  const closeProject = useCallback(async (): Promise<void> => {
    if (!onCloseProject) return;
    setIsClosing(true);
    try {
      await coordinator.flushAll();
      onCloseProject();
    } catch {
      setIsClosing(false);
    }
  }, [coordinator, onCloseProject]);

  if (loadState.state === "empty") {
    return (
      <main className="writing-workspace-state">
        <h1>{project.title}</h1>
        <p>Nenhum documento disponível neste projeto.</p>
        {onCloseProject && (
          <button onClick={onCloseProject} type="button">
            Fechar projeto
          </button>
        )}
      </main>
    );
  }

  if (loadState.state === "loading") {
    return (
      <main aria-busy="true" className="writing-workspace-state">
        <h1>BookMaker</h1>
        <p>Carregando documento…</p>
      </main>
    );
  }

  if (loadState.state === "error") {
    return (
      <main className="writing-workspace-state">
        <h1>Não foi possível abrir o documento</h1>
        <p role="alert">{loadState.message}</p>
        <div className="writing-workspace-state__actions">
          <button
            onClick={() => {
              setLoadState({
                state: "loading",
                documentId: loadState.documentId,
              });
              void loadDocument(loadState.documentId);
            }}
            type="button"
          >
            Tentar novamente
          </button>
          {onCloseProject && (
            <button onClick={onCloseProject} type="button">
              Fechar projeto
            </button>
          )}
        </div>
      </main>
    );
  }

  return (
    <WritingWorkspaceView
      autosaveStatus={autosaveStatus}
      documents={project.documents}
      isClosing={isClosing}
      loadedDocument={loadState.document}
      navigationMatch={navigationMatch}
      onChangeDocument={changeDocument}
      onCloseProject={onCloseProject ? () => void closeProject() : undefined}
      onContentChange={(documentId, content) =>
        coordinator.markDirty(documentId, content)
      }
      onRetrySave={retrySave}
      onNavigationApplied={() => setNavigationMatch(null)}
      project={project}
      search={search}
      tree={tree}
    />
  );
}
