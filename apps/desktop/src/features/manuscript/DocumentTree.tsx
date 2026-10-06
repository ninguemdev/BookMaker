import { useRef, useState } from "react";
import type { KeyboardEvent, MouseEvent } from "react";

const DEFAULT_TREE_LABEL = "Estrutura do livro";
const EMPTY_TREE_MESSAGE = "Nenhum documento neste projeto.";

export type DocumentTreeNode = {
  id: string;
  title: string;
  children?: readonly DocumentTreeNode[];
};

type DocumentTreeProps = {
  nodes: readonly DocumentTreeNode[];
  selectedId?: string | null;
  defaultExpandedIds?: readonly string[];
  label?: string;
  onSelect: (documentId: string) => void;
};

type VisibleTreeItem = {
  node: DocumentTreeNode;
  parentId: string | null;
};

function flattenVisibleNodes(
  nodes: readonly DocumentTreeNode[],
  expandedIds: ReadonlySet<string>,
  parentId: string | null = null,
): VisibleTreeItem[] {
  const visibleItems: VisibleTreeItem[] = [];

  for (const node of nodes) {
    visibleItems.push({ node, parentId });
    if (node.children?.length && expandedIds.has(node.id)) {
      visibleItems.push(
        ...flattenVisibleNodes(node.children, expandedIds, node.id),
      );
    }
  }

  return visibleItems;
}

export function DocumentTree({
  nodes,
  selectedId = null,
  defaultExpandedIds = [],
  label = DEFAULT_TREE_LABEL,
  onSelect,
}: DocumentTreeProps) {
  const [expandedIds, setExpandedIds] = useState(
    () => new Set(defaultExpandedIds),
  );
  const initialVisibleItems = flattenVisibleNodes(nodes, expandedIds);
  const [focusedId, setFocusedId] = useState<string | null>(() => {
    if (
      selectedId &&
      initialVisibleItems.some(({ node }) => node.id === selectedId)
    ) {
      return selectedId;
    }
    return initialVisibleItems[0]?.node.id ?? null;
  });
  const itemElements = useRef(new Map<string, HTMLLIElement>());
  const visibleItems = flattenVisibleNodes(nodes, expandedIds);
  const resolvedFocusedId = visibleItems.some(
    ({ node }) => node.id === focusedId,
  )
    ? focusedId
    : (visibleItems[0]?.node.id ?? null);

  function focusItem(documentId: string) {
    setFocusedId(documentId);
    itemElements.current.get(documentId)?.focus();
  }

  function setExpanded(documentId: string, expanded: boolean) {
    setExpandedIds((currentIds) => {
      const nextIds = new Set(currentIds);
      if (expanded) {
        nextIds.add(documentId);
      } else {
        nextIds.delete(documentId);
      }
      return nextIds;
    });
  }

  function handleKeyDown(
    event: KeyboardEvent<HTMLLIElement>,
    node: DocumentTreeNode,
  ) {
    event.stopPropagation();
    const currentIndex = visibleItems.findIndex(
      ({ node: visibleNode }) => visibleNode.id === node.id,
    );
    const currentItem = visibleItems[currentIndex];
    const hasChildren = Boolean(node.children?.length);
    const isExpanded = expandedIds.has(node.id);

    switch (event.key) {
      case "ArrowDown": {
        event.preventDefault();
        const nextItem = visibleItems[currentIndex + 1];
        if (nextItem) focusItem(nextItem.node.id);
        break;
      }
      case "ArrowUp": {
        event.preventDefault();
        const previousItem = visibleItems[currentIndex - 1];
        if (previousItem) focusItem(previousItem.node.id);
        break;
      }
      case "ArrowRight":
        event.preventDefault();
        if (hasChildren && !isExpanded) {
          setExpanded(node.id, true);
        } else if (hasChildren && node.children?.[0]) {
          focusItem(node.children[0].id);
        }
        break;
      case "ArrowLeft":
        event.preventDefault();
        if (hasChildren && isExpanded) {
          setExpanded(node.id, false);
        } else if (currentItem?.parentId) {
          focusItem(currentItem.parentId);
        }
        break;
      case "Home":
        event.preventDefault();
        if (visibleItems[0]) focusItem(visibleItems[0].node.id);
        break;
      case "End": {
        event.preventDefault();
        const lastItem = visibleItems[visibleItems.length - 1];
        if (lastItem) focusItem(lastItem.node.id);
        break;
      }
      case "Enter":
      case " ":
        event.preventDefault();
        onSelect(node.id);
        break;
    }
  }

  function handleClick(event: MouseEvent<HTMLLIElement>, documentId: string) {
    event.stopPropagation();
    focusItem(documentId);
    onSelect(documentId);
  }

  function renderNodes(treeNodes: readonly DocumentTreeNode[], level: number) {
    return treeNodes.map((node, index) => {
      const hasChildren = Boolean(node.children?.length);
      const isExpanded = expandedIds.has(node.id);
      const isSelected = selectedId === node.id;

      return (
        <li
          key={node.id}
          ref={(element) => {
            if (element) itemElements.current.set(node.id, element);
            else itemElements.current.delete(node.id);
          }}
          className="document-tree__item"
          role="treeitem"
          aria-label={node.title}
          aria-level={level}
          aria-posinset={index + 1}
          aria-setsize={treeNodes.length}
          aria-expanded={hasChildren ? isExpanded : undefined}
          aria-selected={isSelected}
          tabIndex={resolvedFocusedId === node.id ? 0 : -1}
          onClick={(event) => handleClick(event, node.id)}
          onFocus={() => setFocusedId(node.id)}
          onKeyDown={(event) => handleKeyDown(event, node)}
        >
          <span className="document-tree__row">
            <span className="document-tree__indicator" aria-hidden="true">
              {hasChildren ? (isExpanded ? "▾" : "▸") : ""}
            </span>
            <span>{node.title}</span>
          </span>
          {hasChildren && isExpanded ? (
            <ul className="document-tree__group" role="group">
              {renderNodes(node.children ?? [], level + 1)}
            </ul>
          ) : null}
        </li>
      );
    });
  }

  return (
    <div className="document-tree">
      <ul className="document-tree__root" role="tree" aria-label={label}>
        {renderNodes(nodes, 1)}
      </ul>
      {nodes.length === 0 ? (
        <p className="document-tree__empty">{EMPTY_TREE_MESSAGE}</p>
      ) : null}
    </div>
  );
}
