import type { ProjectId } from "@bookmaker/domain";
import { z } from "zod";

export const PROJECT_MANIFEST_FORMAT = "bookmaker-project";
export const PROJECT_MANIFEST_FORMAT_VERSION = 1;
export const PROJECT_MANIFEST_CREATOR = "BookMaker";

const nonBlankString = z
  .string()
  .refine((value) => value.trim().length > 0, "Expected a non-blank string");

const projectIdSchema = nonBlankString.transform(
  (value): ProjectId => value as ProjectId,
);

export const projectManifestSchema = z.strictObject({
  format: z.literal(PROJECT_MANIFEST_FORMAT),
  formatVersion: z.literal(PROJECT_MANIFEST_FORMAT_VERSION),
  projectId: projectIdSchema,
  createdBy: z.literal(PROJECT_MANIFEST_CREATOR),
  minimumAppVersion: nonBlankString,
});

export type ProjectManifest = z.infer<typeof projectManifestSchema>;
