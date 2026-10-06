import { Extension } from "@tiptap/core";
import { Plugin } from "@tiptap/pm/state";

import { assertValidImageAttributes, type BookImageAttributes } from "./image";
import { isSafeEditorLink } from "./link";

const dangerousElements = new Set([
  "audio",
  "base",
  "button",
  "canvas",
  "embed",
  "form",
  "iframe",
  "input",
  "link",
  "math",
  "meta",
  "object",
  "script",
  "select",
  "source",
  "style",
  "svg",
  "textarea",
  "video",
]);

const supportedElements = new Set([
  "blockquote",
  "br",
  "em",
  "h1",
  "h2",
  "h3",
  "li",
  "ol",
  "p",
  "strong",
  "u",
  "ul",
]);

const sourceBlockElements = new Set([
  "article",
  "blockquote",
  "div",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "li",
  "ol",
  "p",
  "pre",
  "section",
  "table",
  "ul",
]);

const renamedElements: Readonly<Record<string, string>> = {
  b: "strong",
  div: "p",
  h4: "p",
  h5: "p",
  h6: "p",
  i: "em",
  pre: "p",
};

function appendNodes(
  parent: globalThis.Node,
  children: globalThis.Node[],
): void {
  children.forEach((child) => parent.appendChild(child));
}

function sanitizeChildren(
  source: ParentNode,
  outputDocument: Document,
): globalThis.Node[] {
  return Array.from(source.childNodes).flatMap((child) =>
    sanitizeNode(child, outputDocument),
  );
}

function sanitizeLink(
  element: Element,
  outputDocument: Document,
): globalThis.Node[] {
  const children = sanitizeChildren(element, outputDocument);
  const href = element.getAttribute("href")?.trim();

  if (!isSafeEditorLink(href)) {
    return children;
  }

  const link = outputDocument.createElement("a");
  link.setAttribute("href", href);
  const title = element.getAttribute("title");
  if (title !== null) {
    link.setAttribute("title", title);
  }
  appendNodes(link, children);
  return [link];
}

function parseInternalImageAttributes(
  element: Element,
): BookImageAttributes | null {
  const image = element.querySelector("img");
  if (image === null) {
    return null;
  }

  const decorativeValue = element.getAttribute("data-decorative") ?? "false";
  if (decorativeValue !== "true" && decorativeValue !== "false") {
    return null;
  }

  const attributes = {
    alignment: element.getAttribute("data-alignment") ?? "center",
    alt: image.getAttribute("alt") ?? "",
    assetId: element.getAttribute("data-asset-id"),
    caption: element.querySelector("figcaption")?.textContent ?? null,
    decorative: decorativeValue === "true",
    width: element.getAttribute("data-width") ?? "full",
  };

  try {
    assertValidImageAttributes(attributes);
    return attributes;
  } catch {
    return null;
  }
}

function sanitizeInternalImage(
  element: Element,
  outputDocument: Document,
): globalThis.Node[] {
  const attributes = parseInternalImageAttributes(element);
  if (attributes === null) {
    return [];
  }

  const figure = outputDocument.createElement("figure");
  figure.setAttribute("data-bookmaker-image", "");
  figure.setAttribute("data-asset-id", attributes.assetId);
  figure.setAttribute("data-alignment", attributes.alignment);
  figure.setAttribute("data-width", attributes.width);
  figure.setAttribute("data-decorative", String(attributes.decorative));

  const image = outputDocument.createElement("img");
  image.setAttribute("alt", attributes.alt);
  figure.appendChild(image);

  if (attributes.caption !== null) {
    const caption = outputDocument.createElement("figcaption");
    caption.textContent = attributes.caption;
    figure.appendChild(caption);
  }

  return [figure];
}

function preserveExternalImageText(
  element: Element,
  outputDocument: Document,
): globalThis.Node[] {
  const caption = element.querySelector("figcaption")?.textContent;
  const alt = element.querySelector("img")?.getAttribute("alt");
  const description = caption ?? alt;

  if (
    description === undefined ||
    description === null ||
    description.length === 0
  ) {
    return [];
  }

  const paragraph = outputDocument.createElement("p");
  paragraph.textContent = description;
  return [paragraph];
}

function sanitizeOrderedListAttributes(source: Element, target: Element): void {
  const rawStart = source.getAttribute("start");
  if (rawStart === null) {
    return;
  }

  const start = Number(rawStart);
  if (Number.isInteger(start) && start >= 1) {
    target.setAttribute("start", String(start));
  }
}

function containsSourceBlock(element: Element): boolean {
  return Array.from(element.children).some((child) =>
    sourceBlockElements.has(child.tagName.toLowerCase()),
  );
}

function sanitizeElement(
  element: Element,
  outputDocument: Document,
): globalThis.Node[] {
  const tagName = element.tagName.toLowerCase();

  if (dangerousElements.has(tagName)) {
    return [];
  }
  if (tagName === "a") {
    return sanitizeLink(element, outputDocument);
  }
  if (tagName === "hr") {
    if (!element.hasAttribute("data-scene-break")) {
      return [];
    }
    const sceneBreak = outputDocument.createElement("hr");
    sceneBreak.setAttribute("data-scene-break", "");
    return [sceneBreak];
  }
  if (tagName === "figure") {
    return element.hasAttribute("data-bookmaker-image")
      ? sanitizeInternalImage(element, outputDocument)
      : preserveExternalImageText(element, outputDocument);
  }
  if (tagName === "img") {
    const alt = element.getAttribute("alt");
    return alt === null || alt.length === 0
      ? []
      : [outputDocument.createTextNode(alt)];
  }

  const targetTagName = renamedElements[tagName] ?? tagName;
  const children = sanitizeChildren(element, outputDocument);

  if (tagName === "div" && containsSourceBlock(element)) {
    return children;
  }
  if (!supportedElements.has(targetTagName)) {
    return children;
  }

  const sanitizedElement = outputDocument.createElement(targetTagName);
  if (targetTagName === "ol") {
    sanitizeOrderedListAttributes(element, sanitizedElement);
  }
  appendNodes(sanitizedElement, children);
  return [sanitizedElement];
}

function sanitizeNode(
  node: globalThis.Node,
  outputDocument: Document,
): globalThis.Node[] {
  if (node.nodeType === node.TEXT_NODE) {
    return [
      outputDocument.createTextNode(sanitizePastedText(node.nodeValue ?? "")),
    ];
  }
  if (node.nodeType !== node.ELEMENT_NODE) {
    return [];
  }
  return sanitizeElement(node as Element, outputDocument);
}

export function sanitizePastedText(text: string): string {
  return text.split("\u0000").join("").replace(/\r\n?/g, "\n");
}

export function sanitizePastedHtml(html: string): string {
  const parsedDocument = new DOMParser().parseFromString(html, "text/html");
  const outputContainer = parsedDocument.createElement("div");
  appendNodes(
    outputContainer,
    sanitizeChildren(parsedDocument.body, parsedDocument),
  );
  return outputContainer.innerHTML;
}

export const PasteSanitizer = Extension.create({
  name: "pasteSanitizer",
  priority: 1_000,

  addProseMirrorPlugins() {
    return [
      new Plugin({
        props: {
          transformPastedHTML: sanitizePastedHtml,
          transformPastedText: sanitizePastedText,
        },
      }),
    ];
  },
});
