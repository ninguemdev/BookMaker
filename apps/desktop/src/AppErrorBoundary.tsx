import { Component, type ReactNode } from "react";

import { reportRenderError } from "./errorHandling";

type AppErrorBoundaryProps = Readonly<{
  children: ReactNode;
  reloadApplication: () => void;
}>;

type AppErrorBoundaryState = Readonly<{
  hasError: boolean;
}>;

export class AppErrorBoundary extends Component<
  AppErrorBoundaryProps,
  AppErrorBoundaryState
> {
  state: AppErrorBoundaryState = { hasError: false };

  static getDerivedStateFromError(): AppErrorBoundaryState {
    return { hasError: true };
  }

  componentDidCatch(): void {
    reportRenderError();
  }

  render(): ReactNode {
    if (this.state.hasError) {
      return (
        <main className="error-boundary" role="alert">
          <section className="error-boundary__content">
            <p className="error-boundary__eyebrow">Erro BM-UNEXPECTED-001</p>
            <h1>Algo deu errado</h1>
            <p>
              O BookMaker encontrou um erro inesperado. Recarregue a aplicação
              para continuar.
            </p>
            <button type="button" onClick={this.props.reloadApplication}>
              Recarregar aplicação
            </button>
          </section>
        </main>
      );
    }

    return this.props.children;
  }
}
