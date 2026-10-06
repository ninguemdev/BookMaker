import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
  type FormEvent,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";

import { findDocumentMatches } from "@bookmaker/editor-core";
import type { Editor } from "@tiptap/core";

import "./EditorFind.css";

interface EditorFindProps {
  editor: Editor | null;
}

function useEditorDocument(editor: Editor | null) {
  const subscribe = useCallback(
    (notify: () => void) => {
      if (editor === null) {
        return () => undefined;
      }

      editor.on("update", notify);
      return () => {
        editor.off("update", notify);
      };
    },
    [editor],
  );
  const getSnapshot = useCallback(() => editor?.state.doc ?? null, [editor]);

  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
}

export function EditorFind({ editor }: EditorFindProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [activeMatchIndex, setActiveMatchIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const document = useEditorDocument(editor);
  const matches = useMemo(
    () => (document === null ? [] : findDocumentMatches(document, query)),
    [document, query],
  );
  const currentMatchIndex =
    matches.length === 0 ? 0 : activeMatchIndex % matches.length;

  useEffect(() => {
    const openWithShortcut = (event: KeyboardEvent): void => {
      if (
        editor !== null &&
        (event.ctrlKey || event.metaKey) &&
        !event.shiftKey &&
        event.key.toLowerCase() === "f"
      ) {
        event.preventDefault();
        setIsOpen(true);
      }
    };

    window.addEventListener("keydown", openWithShortcut);
    return () => {
      window.removeEventListener("keydown", openWithShortcut);
    };
  }, [editor]);

  useEffect(() => {
    if (isOpen) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [isOpen]);

  useEffect(() => {
    const match = matches[currentMatchIndex];
    if (editor === null || match === undefined) {
      return;
    }

    editor.chain().setTextSelection(match).scrollIntoView().run();
  }, [currentMatchIndex, editor, matches]);

  const moveToMatch = (offset: number): void => {
    if (matches.length === 0) {
      return;
    }

    setActiveMatchIndex(
      (currentIndex) =>
        ((currentIndex % matches.length) + offset + matches.length) %
        matches.length,
    );
  };

  const closeSearch = (): void => {
    setIsOpen(false);
    editor?.commands.focus();
  };

  const handleSubmit = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();
    moveToMatch(1);
  };

  const handleInputKeyDown = (
    event: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    if (event.key === "Escape") {
      event.preventDefault();
      closeSearch();
      return;
    }
    if (event.key === "Enter" && event.shiftKey) {
      event.preventDefault();
      moveToMatch(-1);
    }
  };

  const resultStatus =
    query.length === 0
      ? ""
      : matches.length === 0
        ? "Nenhum resultado"
        : `${currentMatchIndex + 1} de ${matches.length}`;

  return (
    <div className="editor-find">
      <button
        aria-expanded={isOpen}
        aria-keyshortcuts="Control+F Meta+F"
        className="editor-find__open"
        disabled={editor === null}
        onClick={() => setIsOpen(true)}
        title="Buscar no documento (Ctrl+F)"
        type="button"
      >
        Buscar
      </button>

      {isOpen && (
        <form
          aria-label="Buscar no documento"
          className="editor-find__panel"
          onSubmit={handleSubmit}
          role="search"
        >
          <label className="editor-find__label" htmlFor="editor-find-query">
            Buscar no documento
          </label>
          <input
            autoComplete="off"
            id="editor-find-query"
            onChange={(event) => {
              setQuery(event.target.value);
              setActiveMatchIndex(0);
            }}
            onKeyDown={handleInputKeyDown}
            ref={inputRef}
            type="search"
            value={query}
          />
          <span
            aria-live="polite"
            className="editor-find__status"
            role="status"
          >
            {resultStatus}
          </span>
          <button
            aria-label="Ocorrência anterior"
            disabled={matches.length === 0}
            onClick={() => moveToMatch(-1)}
            title="Ocorrência anterior (Shift+Enter)"
            type="button"
          >
            ↑
          </button>
          <button
            aria-label="Próxima ocorrência"
            disabled={matches.length === 0}
            title="Próxima ocorrência (Enter)"
            type="submit"
          >
            ↓
          </button>
          <button aria-label="Fechar busca" onClick={closeSearch} type="button">
            ×
          </button>
        </form>
      )}
    </div>
  );
}
