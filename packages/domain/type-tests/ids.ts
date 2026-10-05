import type { AssetId, DocumentId, ProjectId } from "../src/ids";

declare const projectId: ProjectId;
declare const documentId: DocumentId;
declare const assetId: AssetId;

const projectIdAsString: string = projectId;
void projectIdAsString;

// @ts-expect-error Entity IDs must not be interchangeable.
const projectFromDocument: ProjectId = documentId;
void projectFromDocument;

// @ts-expect-error Entity IDs must not be interchangeable.
const documentFromAsset: DocumentId = assetId;
void documentFromAsset;

// @ts-expect-error Raw strings must be generated or validated at a boundary.
const projectFromRawString: ProjectId = "project-id";
void projectFromRawString;
