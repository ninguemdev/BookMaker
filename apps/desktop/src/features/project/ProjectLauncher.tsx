import { useState, type FormEvent } from "react";

import {
  nativeWritingProjectGateway,
  type WritingProject,
  type WritingProjectGateway,
} from "../../application/writingProject";

import "./ProjectLauncher.css";

interface ProjectLauncherProps {
  gateway?: WritingProjectGateway;
  onOpen: (project: WritingProject) => void;
}

function projectErrorMessage(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return "Não foi possível abrir o projeto.";
}

export function ProjectLauncher({
  gateway = nativeWritingProjectGateway,
  onOpen,
}: ProjectLauncherProps) {
  const [projectPath, setProjectPath] = useState("");
  const [isOpening, setIsOpening] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const openProject = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedPath = projectPath.trim();
    if (normalizedPath.length === 0) return;

    setIsOpening(true);
    setError(null);
    try {
      onOpen(await gateway.loadProject(normalizedPath));
    } catch (openError: unknown) {
      setError(projectErrorMessage(openError));
      setIsOpening(false);
    }
  };

  return (
    <main className="project-launcher">
      <section className="project-launcher__card">
        <p className="project-launcher__eyebrow">
          Engine editorial local-first
        </p>
        <h1>BookMaker</h1>
        <p>Abra uma pasta de projeto para continuar escrevendo.</p>
        <form onSubmit={(event) => void openProject(event)}>
          <label htmlFor="project-path">Caminho da pasta .bookmaker</label>
          <input
            autoComplete="off"
            id="project-path"
            onChange={(event) => setProjectPath(event.target.value)}
            placeholder="C:\Livros\MeuLivro.bookmaker"
            type="text"
            value={projectPath}
          />
          <button
            disabled={isOpening || projectPath.trim() === ""}
            type="submit"
          >
            {isOpening ? "Abrindo…" : "Abrir projeto"}
          </button>
        </form>
        {error && (
          <p className="project-launcher__error" role="alert">
            {error}
          </p>
        )}
        <p className="project-launcher__hint">
          O projeto permanece no seu computador e funciona sem conexão.
        </p>
      </section>
    </main>
  );
}
