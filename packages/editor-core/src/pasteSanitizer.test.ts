// @vitest-environment jsdom

import type { AssetId } from "@bookmaker/domain";
import { Editor } from "@tiptap/core";
import { afterEach, describe, expect, it } from "vitest";

import { createEditorExtensions, parseEditorContent } from "./editorSchema";
import { createImageNode } from "./image";
import { sanitizePastedHtml, sanitizePastedText } from "./pasteSanitizer";

const assetId = "0199bc4e-8b72-7000-8000-000000000001" as AssetId;
const editors: Editor[] = [];

function createEditor(): Editor {
  const editor = new Editor({
    content: { type: "doc", content: [{ type: "paragraph" }] },
    extensions: createEditorExtensions(),
  });
  editors.push(editor);
  return editor;
}

afterEach(() => {
  editors.splice(0).forEach((editor) => editor.destroy());
});

describe("paste sanitizer", () => {
  it("preserves text and compatible emphasis while removing foreign styling", () => {
    const sanitized = sanitizePastedHtml(
      '<div class="word" style="font: 17px Comic Sans" onclick="run()">Olá <span style="color:red"><b>mundo</b></span><img src="https://tracker.example/pixel" alt="Diagrama"></div>',
    );

    expect(sanitized).toBe("<p>Olá <strong>mundo</strong>Diagrama</p>");
    expect(sanitized).not.toMatch(/class=|style=|onclick=|src=/);
  });

  it("preserves supported headings, quotes, lists, and safe links", () => {
    const sanitized = sanitizePastedHtml(
      '<h2 id="source">Seção</h2><blockquote cite="external"><p>Uma <i>citação</i>.</p></blockquote><ol start="3" type="A"><li><a href="https://example.com" target="_blank" rel="opener"><u>Fonte</u></a></li></ol>',
    );
    const editor = createEditor();
    editor.commands.setContent(sanitized);
    const content = editor.getJSON();

    expect(content.content?.map((node) => node.type)).toEqual([
      "heading",
      "blockquote",
      "orderedList",
      "paragraph",
    ]);
    expect(content.content?.[0]?.attrs).toMatchObject({ level: 2 });
    expect(content.content?.[2]?.attrs).toMatchObject({ start: 3 });
    expect(JSON.stringify(content)).toContain('"type":"link"');
    expect(() => parseEditorContent(content)).not.toThrow();
    expect(sanitized).not.toMatch(/id=|cite=|target=|rel=|type=/);
  });

  it("removes executable content and unwraps unsafe links", () => {
    const sanitized = sanitizePastedHtml(
      '<p>Antes<script>alert(1)</script><style>body{display:none}</style><a href="java\nscript:alert(1)">clique</a><iframe src="https://example.com">frame</iframe> depois</p>',
    );

    expect(sanitized).toBe("<p>Antesclique depois</p>");
    expect(sanitized).not.toMatch(/script|style|iframe|href/i);
  });

  it("preserves valid BookMaker nodes and discards generic visual separators", () => {
    const imageEditor = new Editor({
      content: {
        type: "doc",
        content: [
          createImageNode({
            alt: "Mapa",
            assetId,
            caption: "Rota principal",
            width: "medium",
          }),
        ],
      },
      extensions: createEditorExtensions(),
    });
    editors.push(imageEditor);
    const imageHtml = imageEditor.getHTML();
    const sanitized = sanitizePastedHtml(
      `<hr><hr data-scene-break>${imageHtml.replace("<img", '<img src="file:///tmp/map.png"')}`,
    );

    expect(sanitized).not.toContain("<hr>");
    expect(sanitized).toContain('<hr data-scene-break="">');
    expect(sanitized).toContain(`data-asset-id="${assetId}"`);
    expect(sanitized).not.toContain("src=");

    const editor = createEditor();
    editor.commands.setContent(sanitized);
    expect(editor.getJSON().content?.map((node) => node.type)).toEqual([
      "sceneBreak",
      "image",
      "paragraph",
    ]);
    expect(() => parseEditorContent(editor.getJSON())).not.toThrow();
  });

  it("normalizes pasted plain text without changing Unicode content", () => {
    expect(sanitizePastedText("Olá\r\n世界\rCafé\u0000")).toBe(
      "Olá\n世界\nCafé",
    );
  });

  it("registers the sanitizer in the editor paste pipeline", () => {
    const editor = createEditor();
    const htmlPlugin = editor.state.plugins.find(
      (plugin) => plugin.props.transformPastedHTML !== undefined,
    );
    const textPlugin = editor.state.plugins.find(
      (plugin) => plugin.props.transformPastedText !== undefined,
    );
    const htmlTransformer = htmlPlugin?.props.transformPastedHTML;
    const textTransformer = textPlugin?.props.transformPastedText;

    expect(htmlTransformer).toBeDefined();
    expect(textTransformer).toBeDefined();
    if (
      htmlPlugin === undefined ||
      htmlTransformer === undefined ||
      textPlugin === undefined ||
      textTransformer === undefined
    ) {
      throw new Error("Paste sanitizer plugins must be registered");
    }
    expect(
      htmlTransformer.call(
        htmlPlugin,
        '<p style="color:red">Seguro<script>não</script></p>',
        editor.view,
      ),
    ).toBe("<p>Seguro</p>");
    expect(
      textTransformer.call(textPlugin, "A\r\nB\u0000", false, editor.view),
    ).toBe("A\nB");
  });
});
