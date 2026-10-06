import { useCallback, useRef, useState } from "react";

import type { DocumentId } from "@bookmaker/domain";
import {
  createEditorExtensions,
  editorSchema,
  findDocumentMatches,
  type EditorContent as EditorJsonContent,
} from "@bookmaker/editor-core";
import { EditorContent as TiptapEditorContent, useEditor } from "@tiptap/react";

import type {
  ProjectSearchInput,
  ProjectSearchMatch,
  ProjectSearchResults,
} from "../../application/projectSearch";
import {
  DocumentTree,
  type DocumentTreeNode,
} from "../manuscript/DocumentTree";
import { EditorFind } from "./EditorFind";
import { EditorFocusMode, FocusModeDistraction } from "./EditorFocusMode";
import { EditorToolbar } from "./EditorToolbar";
import { EditorWordCount } from "./EditorWordCount";
import { ProjectSearch } from "./ProjectSearch";

import "./WritingWorkspace.css";

interface SessionDocument {
  id: DocumentId;
  title: string;
  content: EditorJsonContent;
}

const chapterOneId = "00000000-0000-7000-8000-000000000001" as DocumentId;
const chapterTwoId = "00000000-0000-7000-8000-000000000002" as DocumentId;
const partOneId = "00000000-0000-7000-8000-000000000003" as DocumentId;
const appendixId = "00000000-0000-7000-8000-000000000004" as DocumentId;

const documentTree: readonly DocumentTreeNode[] = [
  {
    id: partOneId,
    title: "Parte I",
    children: [
      { id: chapterOneId, title: "Capítulo 1" },
      { id: chapterTwoId, title: "Capítulo 2" },
    ],
  },
  { id: appendixId, title: "Apêndice" },
];

const initialDocuments: readonly SessionDocument[] = [
  {
    id: partOneId,
    title: "Parte I",
    content: {
      type: "doc",
      content: [
        {
          type: "heading",
          attrs: { level: 1 },
          content: [{ type: "text", text: "Parte I" }],
        },
        {
          type: "paragraph",
          content: [{ type: "text", text: "O começo de uma nova jornada." }],
        },
      ],
    },
  },
  {
    id: chapterOneId,
    title: "Capítulo 1",
    content: {
      type: "doc",
      content: [
        {
          type: "heading",
          attrs: { level: 1 },
          content: [{ type: "text", text: "A chegada" }],
        },
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "Clara abriu a janela e encontrou a cidade coberta de névoa.",
            },
          ],
        },
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "Comece a escrever aqui e experimente os controles do editor.",
            },
          ],
        },
      ],
    },
  },
  {
    id: chapterTwoId,
    title: "Capítulo 2",
    content: {
      type: "doc",
      content: [
        {
          type: "heading",
          attrs: { level: 1 },
          content: [{ type: "text", text: "O reencontro" }],
        },
        {
          type: "paragraph",
          content: [
            {
              type: "text",
              text: "O reencontro aconteceu na plataforma vazia da estação.",
            },
          ],
        },
      ],
    },
  },
  {
    id: appendixId,
    title: "Apêndice",
    content: {
      type: "doc",
      content: [
        {
          type: "heading",
          attrs: { level: 1 },
          content: [{ type: "text", text: "Notas do universo" }],
        },
        {
          type: "bulletList",
          content: [
            {
              type: "listItem",
              content: [
                {
                  type: "paragraph",
                  content: [{ type: "text", text: "Cidade portuária" }],
                },
              ],
            },
          ],
        },
      ],
    },
  },
];

function createSnippet(
  document: ReturnType<typeof editorSchema.nodeFromJSON>,
  from: number,
  to: number,
): string {
  const contextStart = Math.max(0, from - 40);
  const contextEnd = Math.min(document.content.size, to + 40);
  return document
    .textBetween(contextStart, contextEnd, " ", " ")
    .replace(/\s+/g, " ")
    .trim();
}

export function WritingWorkspace() {
  const [documents, setDocuments] = useState(() =>
    initialDocuments.map((document) => ({ ...document })),
  );
  const [selectedDocumentId, setSelectedDocumentId] =
    useState<DocumentId>(chapterOneId);
  const pendingNavigation = useRef<ProjectSearchMatch | null>(null);
  const selectedDocument =
    documents.find((document) => document.id === selectedDocumentId) ??
    documents[0]!;

  const updateDocumentContent = useCallback(
    (documentId: DocumentId, content: EditorJsonContent): void => {
      setDocuments((currentDocuments) =>
        currentDocuments.map((document) =>
          document.id === documentId ? { ...document, content } : document,
        ),
      );
    },
    [],
  );

  const editor = useEditor(
    {
      content: selectedDocument.content,
      editorProps: {
        attributes: {
          "aria-label": `Conteúdo de ${selectedDocument.title}`,
          class: "writing-workspace__prose",
        },
      },
      extensions: createEditorExtensions(),
      onCreate: ({ editor: createdEditor }) => {
        const match = pendingNavigation.current;
        if (match?.documentId === selectedDocument.id) {
          pendingNavigation.current = null;
          createdEditor
            .chain()
            .focus()
            .setTextSelection({ from: match.from, to: match.to })
            .scrollIntoView()
            .run();
        }
      },
      onUpdate: ({ editor: updatedEditor }) => {
        updateDocumentContent(selectedDocument.id, updatedEditor.getJSON());
      },
    },
    [selectedDocument.id],
  );

  const searchSession = useCallback(
    ({ query }: ProjectSearchInput): Promise<ProjectSearchResults> => {
      const matches: ProjectSearchMatch[] = [];

      for (const document of documents) {
        const parsedDocument = editorSchema.nodeFromJSON(document.content);
        for (const match of findDocumentMatches(parsedDocument, query)) {
          matches.push({
            documentId: document.id,
            documentTitle: document.title,
            from: match.from,
            snippet: createSnippet(parsedDocument, match.from, match.to),
            to: match.to,
          });
        }
      }

      return Promise.resolve({
        matches: matches.slice(0, 200),
        truncated: matches.length > 200,
      });
    },
    [documents],
  );

  const navigateToMatch = (match: ProjectSearchMatch): void => {
    if (match.documentId === selectedDocumentId && editor !== null) {
      editor
        .chain()
        .focus()
        .setTextSelection({ from: match.from, to: match.to })
        .scrollIntoView()
        .run();
      return;
    }

    pendingNavigation.current = match;
    setSelectedDocumentId(match.documentId);
  };

  return (
    <EditorFocusMode editor={editor}>
      <div className="writing-workspace">
        <FocusModeDistraction className="writing-workspace__topbar">
          <div className="writing-workspace__brand">
            <h1>BookMaker</h1>
            <p>Livro de demonstração</p>
          </div>
          <div className="writing-workspace__topbar-actions">
            <span className="writing-workspace__session-badge">
              Sessão não persistida
            </span>
            <ProjectSearch
              onNavigate={navigateToMatch}
              projectPath="session://writing-workspace"
              search={searchSession}
            />
          </div>
        </FocusModeDistraction>

        <FocusModeDistraction className="writing-workspace__sidebar">
          <nav aria-label="Manuscrito">
            <h2>Manuscrito</h2>
            <DocumentTree
              defaultExpandedIds={[partOneId]}
              nodes={documentTree}
              onSelect={(documentId) =>
                setSelectedDocumentId(documentId as DocumentId)
              }
              selectedId={selectedDocumentId}
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
            <span role="status">Alterações mantidas somente nesta sessão</span>
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
