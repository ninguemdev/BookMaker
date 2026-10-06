import type { DocumentId } from "@bookmaker/domain";
import {
  createEditorExtensions,
  type EditorContent as EditorJsonContent,
} from "@bookmaker/editor-core";
import { EditorContent as TiptapEditorContent, useEditor } from "@tiptap/react";

import type { AutosaveStatus } from "../../application/autosaveCoordinator";
import type {
  ProjectSearchInput,
  ProjectSearchMatch,
  ProjectSearchResults,
} from "../../application/projectSearch";
import type {
  LoadedDocumentContent,
  WritingDocument,
  WritingProject,
} from "../../application/writingProject";
import {
  DocumentTree,
  type DocumentTreeNode,
} from "../manuscript/DocumentTree";
import { EditorFind } from "./EditorFind";
import { EditorFocusMode, FocusModeDistraction } from "./EditorFocusMode";
import { EditorToolbar } from "./EditorToolbar";
import { EditorWordCount } from "./EditorWordCount";
import { ProjectSearch } from "./ProjectSearch";

interface WritingWorkspaceViewProps {
  project: WritingProject;
  documents: readonly WritingDocument[];
  tree: readonly DocumentTreeNode[];
  loadedDocument: LoadedDocumentContent;
  autosaveStatus: AutosaveStatus;
  isClosing: boolean;
  navigationMatch: ProjectSearchMatch | null;
  onChangeDocument: (
    documentId: DocumentId,
    match?: ProjectSearchMatch,
  ) => Promise<void>;
  onContentChange: (documentId: DocumentId, content: EditorJsonContent) => void;
  onRetrySave: () => Promise<void>;
  onCloseProject?: () => void;
  search: (input: ProjectSearchInput) => Promise<ProjectSearchResults>;
  onNavigationApplied: () => void;
}

function autosaveLabel(status: AutosaveStatus): string {
  switch (status.state) {
    case "dirty":
      return "Alterações pendentes";
    case "saving":
      return "Salvando…";
    case "saved":
      return `Salvo às ${new Intl.DateTimeFormat("pt-BR", {
        hour: "2-digit",
        minute: "2-digit",
      }).format(new Date(status.savedAt))}`;
    case "error":
      return "Erro ao salvar";
    case "idle":
      return "Sem alterações";
  }
}

export function WritingWorkspaceView({
  project,
  documents,
  tree,
  loadedDocument,
  autosaveStatus,
  isClosing,
  navigationMatch,
  onChangeDocument,
  onContentChange,
  onRetrySave,
  onCloseProject,
  search,
  onNavigationApplied,
}: WritingWorkspaceViewProps) {
  const selectedDocument = documents.find(
    (document) => document.documentId === loadedDocument.documentId,
  )!;
  const editor = useEditor(
    {
      content: loadedDocument.content,
      editorProps: {
        attributes: {
          "aria-label": `Conteúdo de ${selectedDocument.title}`,
          class: "writing-workspace__prose",
        },
      },
      extensions: createEditorExtensions(),
      onCreate: ({ editor: createdEditor }) => {
        const match = navigationMatch;
        if (match?.documentId === loadedDocument.documentId) {
          onNavigationApplied();
          createdEditor
            .chain()
            .focus()
            .setTextSelection({ from: match.from, to: match.to })
            .scrollIntoView()
            .run();
        }
      },
      onUpdate: ({ editor: updatedEditor }) => {
        onContentChange(loadedDocument.documentId, updatedEditor.getJSON());
      },
    },
    [loadedDocument.documentId],
  );

  const navigateToMatch = (match: ProjectSearchMatch): void => {
    if (match.documentId === loadedDocument.documentId && editor !== null) {
      editor
        .chain()
        .focus()
        .setTextSelection({ from: match.from, to: match.to })
        .scrollIntoView()
        .run();
      return;
    }
    void onChangeDocument(match.documentId, match);
  };

  return (
    <EditorFocusMode editor={editor}>
      <div className="writing-workspace">
        <FocusModeDistraction className="writing-workspace__topbar">
          <div className="writing-workspace__brand">
            <h1>BookMaker</h1>
            <p>{project.title}</p>
          </div>
          <div className="writing-workspace__topbar-actions">
            <ProjectSearch
              onNavigate={navigateToMatch}
              projectPath={project.path}
              search={search}
            />
            {onCloseProject && (
              <button
                className="writing-workspace__close-project"
                disabled={isClosing}
                onClick={onCloseProject}
                type="button"
              >
                {isClosing ? "Fechando…" : "Fechar projeto"}
              </button>
            )}
          </div>
        </FocusModeDistraction>

        <FocusModeDistraction className="writing-workspace__sidebar">
          <nav aria-label="Manuscrito">
            <h2>Manuscrito</h2>
            <DocumentTree
              defaultExpandedIds={documents
                .filter((document) =>
                  documents.some(
                    (candidate) => candidate.parentId === document.documentId,
                  ),
                )
                .map((document) => document.documentId)}
              nodes={tree}
              onSelect={(documentId) =>
                void onChangeDocument(documentId as DocumentId)
              }
              selectedId={loadedDocument.documentId}
            />
          </nav>
        </FocusModeDistraction>

        <main className="writing-workspace__editor">
          <header className="writing-workspace__document-header">
            <div>
              <p className="writing-workspace__eyebrow">Documento atual</p>
              <h2>{selectedDocument.title}</h2>
            </div>
            <EditorFind editor={editor} />
          </header>

          <EditorToolbar editor={editor} />
          <div className="writing-workspace__page">
            <TiptapEditorContent editor={editor} />
          </div>
          <footer className="writing-workspace__statusbar">
            <EditorWordCount editor={editor} />
            <div
              aria-live="polite"
              className={`writing-workspace__save-status writing-workspace__save-status--${autosaveStatus.state}`}
            >
              <span>{autosaveLabel(autosaveStatus)}</span>
              {autosaveStatus.state === "error" && (
                <button onClick={() => void onRetrySave()} type="button">
                  Tentar novamente
                </button>
              )}
            </div>
          </footer>
        </main>

        <FocusModeDistraction className="writing-workspace__inspector">
          <aside aria-label="Ajuda de escrita">
            <h2>Atalhos</h2>
            <dl>
              <div>
                <dt>Buscar</dt>
                <dd>Ctrl/Cmd+F</dd>
              </div>
              <div>
                <dt>Buscar no projeto</dt>
                <dd>Ctrl/Cmd+Shift+F</dd>
              </div>
              <div>
                <dt>Modo foco</dt>
                <dd>Ctrl/Cmd+Shift+Enter</dd>
              </div>
            </dl>
          </aside>
        </FocusModeDistraction>
      </div>
    </EditorFocusMode>
  );
}
