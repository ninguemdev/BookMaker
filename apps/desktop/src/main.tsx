import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AppErrorBoundary } from "./AppErrorBoundary";
import { installGlobalErrorHandlers } from "./errorHandling";
import "./styles.css";

installGlobalErrorHandlers();

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <AppErrorBoundary reloadApplication={() => window.location.reload()}>
      <App />
    </AppErrorBoundary>
  </React.StrictMode>,
);
