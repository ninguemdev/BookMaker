declare const entityIdBrand: unique symbol;

type EntityId<EntityName extends string> = string & {
  readonly [entityIdBrand]: EntityName;
};

export type ProjectId = EntityId<"ProjectId">;
export type DocumentId = EntityId<"DocumentId">;
export type AssetId = EntityId<"AssetId">;
