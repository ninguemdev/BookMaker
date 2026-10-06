import { useState, type FormEvent } from "react";

import {
  nativeProjectLauncherActions,
  type ProjectLauncherActions,
} from "../../application/projectLauncher";
import {
  nativeWritingProjectGateway,
  type WritingProject,
  type WritingProjectGateway,
} from "../../application/writingProject";

import "./ProjectLauncher.css";

interface ProjectLauncherProps {
  gateway?: WritingProjectGateway;
  launcherActions?: ProjectLauncherActions;
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
  return "Não foi possível concluir a operação.";
}

export function ProjectLauncher({
  gateway = nativeWritingProjectGateway,
  launcherActions = nativeProjectLauncherActions,
  onOpen,
}: ProjectLauncherProps) {
  const [title, setTitle] = useState("");
  const [parentDirectory, setParentDirectory] = useState("");
  const [projectPath, setProjectPath] = useState("");
  const [pendingAction, setPendingAction] = useState<
    "creating" | "opening" | "selecting" | null
  >(null);
  const [error, setError] = useState<string | null>(null);

  const chooseNewProjectParent = async () => {
    setPendingAction("selecting");
    setError(null);
    try {
      const selectedDirectory = await launcherActions.selectNewProjectParent();
      if (selectedDirectory !== null) setParentDirectory(selectedDirectory);
    } catch (selectionError: unknown) {
      setError(projectErrorMessage(selectionError));
    } finally {
      setPendingAction(null);
    }
  };

  const createProject = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedTitle = title.trim();
    if (normalizedTitle.length === 0 || parentDirectory.length === 0) return;

    setPendingAction("creating");
    setError(null);
    try {
      onOpen(
        await launcherActions.createProject({
          parentDirectory,
          title: normalizedTitle,
          language: "pt-BR",
        }),
      );
    } catch (createError: unknown) {
      setError(projectErrorMessage(createError));
      setPendingAction(null);
    }
  };

  const chooseExistingProject = async () => {
    setPendingAction("selecting");
    setError(null);
    try {
      const selectedDirectory = await launcherActions.selectExistingProject();
      if (selectedDirectory !== null) setProjectPath(selectedDirectory);
    } catch (selectionError: unknown) {
      setError(projectErrorMessage(selectionError));
    } finally {
      setPendingAction(null);
    }
  };

  const openProject = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedPath = projectPath.trim();
    if (normalizedPath.length === 0) return;

    setPendingAction("opening");
    setError(null);
    try {
      onOpen(await gateway.loadProject(normalizedPath));
    } catch (openError: unknown) {
      setError(projectErrorMessage(openError));
      setPendingAction(null);
    }
  };

  return (
    <main className="project-launcher">
      <section className="project-launcher__card">
        <p className="project-launcher__eyebrow">
          Engine editorial local-first
        </p>
        <h1>BookMaker</h1>
        <p>Crie seu primeiro livro ou continue um projeto existente.</p>

        <form onSubmit={(event) => void createProject(event)}>
          <h2>Novo livro</h2>
          <label htmlFor="project-title">Título do livro</label>
          <input
            autoComplete="off"
            id="project-title"
            onChange={(event) => setTitle(event.target.value)}
            placeholder="Meu livro"
            type="text"
            value={title}
          />
          <label htmlFor="parent-directory">Onde salvar</label>
          <div className="project-launcher__path-field">
            <input
              id="parent-directory"
              placeholder="Escolha uma pasta no seu computador"
              readOnly
              type="text"
              value={parentDirectory}
            />
            <button
              className="project-launcher__secondary-button"
              disabled={pendingAction !== null}
              onClick={() => void chooseNewProjectParent()}
              type="button"
            >
              Escolher pasta…
            </button>
          </div>
          <p className="project-launcher__field-hint">
            O BookMaker criará uma nova pasta .bookmaker dentro do local
            escolhido.
          </p>
          <button
            disabled={
              pendingAction !== null ||
              title.trim() === "" ||
              parentDirectory === ""
            }
            type="submit"
          >
            {pendingAction === "creating" ? "Criando…" : "Criar livro"}
          </button>
        </form>

        <div className="project-launcher__divider" role="separator" />

        <form onSubmit={(event) => void openProject(event)}>
          <h2>Abrir projeto existente</h2>
          <label htmlFor="project-path">Caminho da pasta .bookmaker</label>
          <div className="project-launcher__path-field">
            <input
              autoComplete="off"
              id="project-path"
              onChange={(event) => setProjectPath(event.target.value)}
              placeholder="C:\Livros\MeuLivro.bookmaker"
              type="text"
              value={projectPath}
            />
            <button
              className="project-launcher__secondary-button"
              disabled={pendingAction !== null}
              onClick={() => void chooseExistingProject()}
              type="button"
            >
              Procurar…
            </button>
          </div>
          <button
            disabled={pendingAction !== null || projectPath.trim() === ""}
            type="submit"
          >
            {pendingAction === "opening" ? "Abrindo…" : "Abrir projeto"}
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
