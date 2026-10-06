import type { AssetId } from "@bookmaker/domain";
import { Node, type JSONContent } from "@tiptap/core";

import { insertBlockNodeWithTrailingParagraph } from "./insertBlockNode";

export const IMAGE_NODE_NAME = "image";
export const IMAGE_ALIGNMENTS = ["left", "center", "right"] as const;
export const IMAGE_WIDTH_PRESETS = [
  "small",
  "medium",
  "large",
  "full",
] as const;

export type ImageAlignment = (typeof IMAGE_ALIGNMENTS)[number];
export type ImageWidthPreset = (typeof IMAGE_WIDTH_PRESETS)[number];

export interface BookImageAttributes {
  alignment: ImageAlignment;
  alt: string;
  assetId: AssetId;
  caption: string | null;
  decorative: boolean;
  width: ImageWidthPreset;
}

export interface InsertImageOptions {
  alignment?: ImageAlignment;
  alt?: string;
  assetId: AssetId;
  caption?: string | null;
  decorative?: boolean;
  width?: ImageWidthPreset;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    bookmakerImage: {
      insertImage: (options: InsertImageOptions) => ReturnType;
    };
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isOneOf<const Value extends string>(
  value: unknown,
  supportedValues: readonly Value[],
): value is Value {
  return typeof value === "string" && supportedValues.includes(value as Value);
}

export function assertValidImageAttributes(
  value: unknown,
): asserts value is BookImageAttributes {
  if (!isRecord(value)) {
    throw new TypeError("Image attributes must be an object");
  }

  if (typeof value.assetId !== "string" || value.assetId.trim().length === 0) {
    throw new RangeError("Image assetId must be a non-blank string");
  }
  if (typeof value.alt !== "string") {
    throw new TypeError("Image alt text must be a string");
  }
  if (value.caption !== null && typeof value.caption !== "string") {
    throw new TypeError("Image caption must be a string or null");
  }
  if (!isOneOf(value.alignment, IMAGE_ALIGNMENTS)) {
    throw new RangeError("Image alignment is not supported");
  }
  if (!isOneOf(value.width, IMAGE_WIDTH_PRESETS)) {
    throw new RangeError("Image width preset is not supported");
  }
  if (typeof value.decorative !== "boolean") {
    throw new TypeError("Image decorative flag must be a boolean");
  }
  if (value.decorative && value.alt.length > 0) {
    throw new RangeError("Decorative images cannot contain alt text");
  }
}

function createImageAttributes(
  options: InsertImageOptions,
): BookImageAttributes {
  const attributes: BookImageAttributes = {
    alignment: options.alignment ?? "center",
    alt: options.alt ?? "",
    assetId: options.assetId,
    caption: options.caption ?? null,
    decorative: options.decorative ?? false,
    width: options.width ?? "full",
  };
  assertValidImageAttributes(attributes);
  return attributes;
}

function parseBooleanAttribute(value: string | undefined): boolean | undefined {
  if (value === undefined) {
    return false;
  }
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  return undefined;
}

function parseImageElement(element: HTMLElement): BookImageAttributes | false {
  const image = element.querySelector("img");
  if (!(image instanceof HTMLImageElement)) {
    return false;
  }

  const decorative = parseBooleanAttribute(element.dataset.decorative);
  if (decorative === undefined) {
    return false;
  }

  const attributes = {
    alignment: element.dataset.alignment ?? "center",
    alt: image.getAttribute("alt") ?? "",
    assetId: element.dataset.assetId,
    caption: element.querySelector("figcaption")?.textContent ?? null,
    decorative,
    width: element.dataset.width ?? "full",
  };

  try {
    assertValidImageAttributes(attributes);
    return attributes;
  } catch {
    return false;
  }
}

export const BookImage = Node.create({
  name: IMAGE_NODE_NAME,
  group: "block",
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      alignment: { default: "center" },
      alt: { default: "" },
      assetId: { default: null },
      caption: { default: null },
      decorative: { default: false },
      width: { default: "full" },
    };
  },

  parseHTML() {
    return [
      {
        tag: "figure[data-bookmaker-image]",
        getAttrs: (element) => parseImageElement(element),
      },
    ];
  },

  renderHTML({ node }) {
    const attributes: unknown = node.attrs;
    assertValidImageAttributes(attributes);

    const figureAttributes = {
      "data-alignment": attributes.alignment,
      "data-asset-id": attributes.assetId,
      "data-bookmaker-image": "",
      "data-decorative": String(attributes.decorative),
      "data-width": attributes.width,
    };
    const image = ["img", { alt: attributes.alt }];

    return attributes.caption === null
      ? ["figure", figureAttributes, image]
      : [
          "figure",
          figureAttributes,
          image,
          ["figcaption", {}, attributes.caption],
        ];
  },

  addCommands() {
    return {
      insertImage: (options) => (commandProps) => {
        let attributes: BookImageAttributes;
        try {
          attributes = createImageAttributes(options);
        } catch {
          return false;
        }

        return insertBlockNodeWithTrailingParagraph(commandProps, {
          attributes: { ...attributes },
          nodeName: this.name,
        });
      },
    };
  },
});

export function createImageNode(options: InsertImageOptions): JSONContent {
  return {
    type: IMAGE_NODE_NAME,
    attrs: createImageAttributes(options),
  };
}
