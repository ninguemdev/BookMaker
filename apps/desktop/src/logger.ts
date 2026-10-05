import {
  debug as writeDebug,
  error as writeError,
  info as writeInfo,
  warn as writeWarn,
  type LogOptions,
} from "@tauri-apps/plugin-log";

export const LOG_CATEGORIES = [
  "project",
  "persistence",
  "editor",
  "publishing",
  "migration",
  "native",
] as const;

export type LogCategory = (typeof LOG_CATEGORIES)[number];

export type Logger = Readonly<{
  debug: (eventName: string) => Promise<void>;
  info: (eventName: string) => Promise<void>;
  warn: (eventName: string) => Promise<void>;
  error: (eventName: string) => Promise<void>;
}>;

export function createLogger(category: LogCategory): Logger {
  const options: LogOptions = { keyValues: { category } };

  return {
    debug: (eventName) => writeDebug(eventName, options),
    info: (eventName) => writeInfo(eventName, options),
    warn: (eventName) => writeWarn(eventName, options),
    error: (eventName) => writeError(eventName, options),
  };
}
