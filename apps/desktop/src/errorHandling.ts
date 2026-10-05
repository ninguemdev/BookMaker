import { createLogger } from "./logger";

const applicationLogger = createLogger("native");

function reportError(eventName: string): void {
  void applicationLogger.error(eventName).catch((loggingError: unknown) => {
    console.error("BookMaker could not write an error log.", loggingError);
  });
}

export function reportRenderError(): void {
  reportError("react_render_failed");
}

export function installGlobalErrorHandlers(
  target: Window = window,
): () => void {
  const handleError = () => reportError("uncaught_error");
  const handleUnhandledRejection = () =>
    reportError("unhandled_promise_rejection");

  target.addEventListener("error", handleError);
  target.addEventListener("unhandledrejection", handleUnhandledRejection);

  return () => {
    target.removeEventListener("error", handleError);
    target.removeEventListener("unhandledrejection", handleUnhandledRejection);
  };
}
