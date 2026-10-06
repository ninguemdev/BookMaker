import {
  createContext,
  useContext,
  useEffect,
  useState,
  type HTMLAttributes,
  type ReactNode,
} from "react";

import type { Editor } from "@tiptap/core";

import "./EditorFocusMode.css";

const FocusModeContext = createContext(false);

interface EditorFocusModeProps {
  children: ReactNode;
  editor: Editor | null;
  onActiveChange?: (isActive: boolean) => void;
}

export function EditorFocusMode({
  children,
  editor,
  onActiveChange,
}: EditorFocusModeProps) {
  const [requestedActive, setRequestedActive] = useState(false);
  const isActive = editor !== null && requestedActive;

  const changeMode = (nextActive: boolean): void => {
    if (editor === null) {
      return;
    }

    setRequestedActive(nextActive);
    onActiveChange?.(nextActive);
    editor.commands.focus();
  };

  useEffect(() => {
    const handleShortcut = (event: KeyboardEvent): void => {
      if (event.defaultPrevented || editor === null) {
        return;
      }

      if (
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key === "Enter"
      ) {
        event.preventDefault();
        changeMode(!isActive);
        return;
      }

      if (isActive && event.key === "Escape") {
        event.preventDefault();
        changeMode(false);
      }
    };

    window.addEventListener("keydown", handleShortcut);
    return () => {
      window.removeEventListener("keydown", handleShortcut);
    };
  });

  return (
    <section
      aria-label="Área de escrita"
      className="editor-focus-mode"
      data-focus-mode={isActive ? "active" : "inactive"}
    >
      <div className="editor-focus-mode__controls">
        <button
          aria-keyshortcuts={
            isActive
              ? "Control+Shift+Enter Meta+Shift+Enter Escape"
              : "Control+Shift+Enter Meta+Shift+Enter"
          }
          aria-pressed={isActive}
          disabled={editor === null}
          onClick={() => changeMode(!isActive)}
          title="Modo foco (Ctrl/Cmd+Shift+Enter)"
          type="button"
        >
          {isActive ? "Sair do modo foco" : "Entrar no modo foco"}
        </button>
        <span
          aria-live="polite"
          className="editor-focus-mode__announcement"
          role="status"
        >
          {isActive ? "Modo foco ativado" : "Modo foco desativado"}
        </span>
      </div>
      <div className="editor-focus-mode__workspace">
        <FocusModeContext.Provider value={isActive}>
          {children}
        </FocusModeContext.Provider>
      </div>
    </section>
  );
}

export function FocusModeDistraction({
  children,
  className,
  hidden = false,
  ...attributes
}: HTMLAttributes<HTMLDivElement>) {
  const isFocusModeActive = useContext(FocusModeContext);
  const classes = ["editor-focus-mode__distraction", className]
    .filter(Boolean)
    .join(" ");

  return (
    <div
      {...attributes}
      className={classes}
      hidden={hidden || isFocusModeActive}
    >
      {children}
    </div>
  );
}
