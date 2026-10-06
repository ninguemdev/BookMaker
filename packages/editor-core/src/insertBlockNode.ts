import {
  canInsertNode,
  isNodeSelection,
  type CommandProps,
} from "@tiptap/core";
import { NodeSelection, TextSelection } from "@tiptap/pm/state";

interface InsertBlockNodeOptions {
  attributes?: Record<string, unknown>;
  nodeName: string;
}

export function insertBlockNodeWithTrailingParagraph(
  { chain, state }: Pick<CommandProps, "chain" | "state">,
  { attributes, nodeName }: InsertBlockNodeOptions,
): boolean {
  const nodeType = state.schema.nodes[nodeName];
  if (nodeType === undefined || !canInsertNode(state, nodeType)) {
    return false;
  }

  const insertion = chain();
  const node = { type: nodeName, attrs: attributes };
  const { selection } = state;

  if (isNodeSelection(selection)) {
    insertion.insertContentAt(selection.$to.pos, node);
  } else {
    insertion.insertContent(node);
  }

  return insertion
    .command(({ state: nextState, tr, dispatch }) => {
      if (dispatch === undefined) {
        return true;
      }

      const { $to } = tr.selection;
      const nodeAfter = $to.nodeAfter;

      if (nodeAfter?.isTextblock) {
        tr.setSelection(TextSelection.create(tr.doc, $to.pos + 1));
      } else if (nodeAfter?.isBlock) {
        tr.setSelection(NodeSelection.create(tr.doc, $to.pos));
      } else if (nodeAfter === null) {
        const paragraph = nextState.schema.nodes.paragraph?.create();
        if (paragraph !== undefined) {
          const positionAfterInsertedNode = $to.end();
          tr.insert(positionAfterInsertedNode, paragraph);
          tr.setSelection(
            TextSelection.create(tr.doc, positionAfterInsertedNode + 1),
          );
        }
      }

      tr.scrollIntoView();
      return true;
    })
    .run();
}
