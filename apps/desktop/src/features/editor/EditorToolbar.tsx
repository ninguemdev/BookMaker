import { useEffect, useReducer, useState, type FormEvent } from "react";

import { isSafeEditorLink } from "@bookmaker/editor-core";
import type { Editor } from "@tiptap/core";

import "./EditorToolbar.css";

interface EditorToolbarProps {
  editor: Editor | null;
}

interface ToolbarButtonProps {
  disabled: boolean;
  label: string;
  onClick: () => void;
  pressed?: boolean;
  title?: string;
}

function ToolbarButton({
  disabled,
  label,
  onClick,
  pressed,
  title,
}: ToolbarButtonProps) {
  return (
    <button
      aria-label={label}
      aria-pressed={pressed}
      className="editor-toolbar__button"
      disabled={disabled}
      onClick={onClick}
      title={title}
      type="button"
    >
      {label}
    </button>
  );
}

function useEditorUpdates(editor: Editor | null): void {
  const [, refresh] = useReducer((revision: number) => revision + 1, 0);

  useEffect(() => {
    if (editor === null) {
      return;
    }

    editor.on("transaction", refresh);
    return () => {
      editor.off("transaction", refresh);
    };
  }, [editor]);
}

export function EditorToolbar({ editor }: EditorToolbarProps) {
  const [isLinkEditorOpen, setIsLinkEditorOpen] = useState(false);
  const [linkHref, setLinkHref] = useState("");
  const [linkError, setLinkError] = useState<string | null>(null);

  useEditorUpdates(editor);

  const run = (command: (currentEditor: Editor) => void): void => {
    if (editor !== null) {
      command(editor);
    }
  };

  const openLinkEditor = (): void => {
    if (editor === null) {
      return;
    }

    const currentHref: unknown = editor.getAttributes("link").href;
    setLinkHref(typeof currentHref === "string" ? currentHref : "");
    setLinkError(null);
    setIsLinkEditorOpen(true);
  };

  const closeLinkEditor = (): void => {
    setIsLinkEditorOpen(false);
    setLinkError(null);
  };

  const applyLink = (event: FormEvent<HTMLFormElement>): void => {
    event.preventDefault();

    const normalizedHref = linkHref.trim();
    if (!isSafeEditorLink(normalizedHref)) {
      setLinkError("Informe um endereço seguro e válido.");
      return;
    }

    run((currentEditor) => {
      currentEditor
        .chain()
        .focus()
        .extendMarkRange("link")
        .setLink({ href: normalizedHref })
        .run();
    });
    closeLinkEditor();
  };

  const removeLink = (): void => {
    run((currentEditor) => {
      currentEditor.chain().focus().extendMarkRange("link").unsetLink().run();
    });
    closeLinkEditor();
  };

  const isUnavailable = editor === null;

  return (
    <div
      aria-label="Formatação do texto"
      className="editor-toolbar"
      role="toolbar"
    >
      <div
        aria-label="Estilo do bloco"
        className="editor-toolbar__group"
        role="group"
      >
        <ToolbarButton
          disabled={isUnavailable}
          label="Texto"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().setParagraph().run(),
            )
          }
          pressed={editor?.isActive("paragraph") ?? false}
        />
        {([1, 2, 3] as const).map((level) => (
          <ToolbarButton
            disabled={isUnavailable}
            key={level}
            label={`Título ${level}`}
            onClick={() =>
              run((currentEditor) =>
                currentEditor.chain().focus().toggleHeading({ level }).run(),
              )
            }
            pressed={editor?.isActive("heading", { level }) ?? false}
          />
        ))}
        <ToolbarButton
          disabled={isUnavailable}
          label="Citação"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleBlockquote().run(),
            )
          }
          pressed={editor?.isActive("blockquote") ?? false}
        />
        <ToolbarButton
          disabled={isUnavailable}
          label="Quebra de cena"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().insertSceneBreak().run(),
            )
          }
          pressed={editor?.isActive("sceneBreak") ?? false}
        />
      </div>

      <div aria-label="Ênfase" className="editor-toolbar__group" role="group">
        <ToolbarButton
          disabled={isUnavailable}
          label="Negrito"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleBold().run(),
            )
          }
          pressed={editor?.isActive("bold") ?? false}
          title="Negrito (Ctrl+B)"
        />
        <ToolbarButton
          disabled={isUnavailable}
          label="Itálico"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleItalic().run(),
            )
          }
          pressed={editor?.isActive("italic") ?? false}
          title="Itálico (Ctrl+I)"
        />
        <ToolbarButton
          disabled={isUnavailable}
          label="Sublinhado"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleUnderline().run(),
            )
          }
          pressed={editor?.isActive("underline") ?? false}
          title="Sublinhado (Ctrl+U)"
        />
        <ToolbarButton
          disabled={isUnavailable}
          label="Link"
          onClick={openLinkEditor}
          pressed={editor?.isActive("link") ?? false}
        />
      </div>

      <div aria-label="Listas" className="editor-toolbar__group" role="group">
        <ToolbarButton
          disabled={isUnavailable}
          label="Lista"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleBulletList().run(),
            )
          }
          pressed={editor?.isActive("bulletList") ?? false}
        />
        <ToolbarButton
          disabled={isUnavailable}
          label="Lista numerada"
          onClick={() =>
            run((currentEditor) =>
              currentEditor.chain().focus().toggleOrderedList().run(),
            )
          }
          pressed={editor?.isActive("orderedList") ?? false}
        />
      </div>

      <div
        aria-label="Histórico"
        className="editor-toolbar__group"
        role="group"
      >
        <ToolbarButton
          disabled={editor === null || !editor.can().undo()}
          label="Desfazer"
          onClick={() =>
            run((currentEditor) => currentEditor.chain().focus().undo().run())
          }
          title="Desfazer (Ctrl+Z)"
        />
        <ToolbarButton
          disabled={editor === null || !editor.can().redo()}
          label="Refazer"
          onClick={() =>
            run((currentEditor) => currentEditor.chain().focus().redo().run())
          }
          title="Refazer (Ctrl+Shift+Z)"
        />
      </div>

      {isLinkEditorOpen && (
        <form className="editor-toolbar__link-editor" onSubmit={applyLink}>
          <label
            className="editor-toolbar__link-label"
            htmlFor="editor-link-href"
          >
            Endereço do link
          </label>
          <input
            aria-describedby={
              linkError === null ? undefined : "editor-link-error"
            }
            aria-invalid={linkError !== null}
            autoFocus
            id="editor-link-href"
            onChange={(event) => {
              setLinkHref(event.target.value);
              setLinkError(null);
            }}
            placeholder="https://exemplo.com"
            inputMode="url"
            spellCheck={false}
            type="text"
            value={linkHref}
          />
          <button type="submit">Aplicar</button>
          {editor?.isActive("link") && (
            <button onClick={removeLink} type="button">
              Remover
            </button>
          )}
          <button onClick={closeLinkEditor} type="button">
            Cancelar
          </button>
          {linkError !== null && (
            <span
              className="editor-toolbar__link-error"
              id="editor-link-error"
              role="alert"
            >
              {linkError}
            </span>
          )}
        </form>
      )}
    </div>
  );
}
