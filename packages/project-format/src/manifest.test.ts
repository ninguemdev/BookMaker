import { describe, expect, it } from "vitest";

import type { ProjectId } from "@bookmaker/domain";

import {
  PROJECT_MANIFEST_CREATOR,
  PROJECT_MANIFEST_FORMAT,
  PROJECT_MANIFEST_FORMAT_VERSION,
  projectManifestSchema,
} from "./manifest";

const validManifest = {
  format: PROJECT_MANIFEST_FORMAT,
  formatVersion: PROJECT_MANIFEST_FORMAT_VERSION,
  projectId: "0199b8ec-2e71-7000-8000-000000000001",
  createdBy: PROJECT_MANIFEST_CREATOR,
  minimumAppVersion: "0.1.0",
};

describe("projectManifestSchema", () => {
  it("accepts the version 1 project manifest", () => {
    const manifest = projectManifestSchema.parse(validManifest);
    const projectId: ProjectId = manifest.projectId;

    expect(projectId).toBe(validManifest.projectId);
    expect(manifest).toEqual(validManifest);
  });

  it.each([
    ["an unknown format", { ...validManifest, format: "other-format" }],
    ["an unsupported format version", { ...validManifest, formatVersion: 2 }],
    ["a blank project id", { ...validManifest, projectId: "   " }],
    ["an unknown creator", { ...validManifest, createdBy: "OtherApp" }],
    [
      "a blank minimum app version",
      { ...validManifest, minimumAppVersion: "" },
    ],
    ["an unknown field", { ...validManifest, unexpected: true }],
  ])("rejects %s", (_case, manifest) => {
    expect(projectManifestSchema.safeParse(manifest).success).toBe(false);
  });
});
