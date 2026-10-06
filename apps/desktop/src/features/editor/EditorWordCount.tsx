import { useEffect, useState } from "react";

import { countDocumentWords } from "@bookmaker/editor-core";
import type { Editor } from "@tiptap/core";

import "./EditorWordCount.css";

interface EditorWordCountProps {
  editor: Editor | null;
}

function getEditorWordCount(editor: Editor | null): number {
  return editor === null ? 0 : countDocumentWords(editor.state.doc);
}

export function EditorWordCount({ editor }: EditorWordCountProps) {
  const [wordCount, setWordCount] = useState(() => getEditorWordCount(editor));

  useEffect(() => {
    const updateWordCount = (): void => {
      setWordCount(getEditorWordCount(editor));
    };

    updateWordCount();
    editor?.on("update", updateWordCount);

    return () => {
      editor?.off("update", updateWordCount);
    };
  }, [editor]);

  const label = wordCount === 1 ? "palavra" : "palavras";

  return (
    <output
      aria-label="Contagem de palavras"
      aria-live="polite"
      className="editor-word-count"
    >
      {wordCount.toLocaleString("pt-BR")} {label}
    </output>
  );
}
