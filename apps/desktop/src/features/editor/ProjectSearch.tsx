import {
  useEffect,
  useRef,
  useState,
  type FormEvent,
  type KeyboardEvent as ReactKeyboardEvent,
} from "react";

import {
  searchProject,
  type ProjectSearchInput,
  type ProjectSearchMatch,
  type ProjectSearchResults,
} from "../../application/projectSearch";

import "./ProjectSearch.css";

interface ProjectSearchProps {
  projectPath: string | null;
  onNavigate: (match: ProjectSearchMatch) => void;
  search?: (input: ProjectSearchInput) => Promise<ProjectSearchResults>;
}

type SearchState =
  | { status: "idle" }
  | { status: "loading" }
  | { status: "success"; results: ProjectSearchResults }
  | { status: "error"; message: string };

function errorMessage(error: unknown): string {
  if (
    typeof error === "object" &&
    error !== null &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return error.message;
  }
  return "Não foi possível buscar no projeto.";
}

export function ProjectSearch({
  projectPath,
  onNavigate,
  search = searchProject,
}: ProjectSearchProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [query, setQuery] = useState("");
  const [searchState, setSearchState] = useState<SearchState>({
    status: "idle",
  });
  const inputRef = useRef<HTMLInputElement>(null);
  const requestVersion = useRef(0);

  useEffect(() => {
    const openWithShortcut = (event: KeyboardEvent): void => {
      if (
        projectPath !== null &&
        (event.ctrlKey || event.metaKey) &&
        event.shiftKey &&
        event.key.toLowerCase() === "f"
      ) {
        event.preventDefault();
        setIsOpen(true);
      }
    };

    window.addEventListener("keydown", openWithShortcut);
    return () => {
      window.removeEventListener("keydown", openWithShortcut);
    };
  }, [projectPath]);

  useEffect(() => {
    if (isOpen) {
      inputRef.current?.focus();
      inputRef.current?.select();
    }
  }, [isOpen]);

  useEffect(
    () => () => {
      requestVersion.current += 1;
    },
    [],
  );

  const closeSearch = (): void => {
    requestVersion.current += 1;
    setIsOpen(false);
  };

  const handleSubmit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const normalizedQuery = query.trim();
    if (projectPath === null || normalizedQuery.length === 0) {
      setSearchState({ status: "idle" });
      return;
    }

    const currentRequest = ++requestVersion.current;
    setSearchState({ status: "loading" });
    try {
      const results = await search({
        projectPath,
        query: normalizedQuery,
      });
      if (requestVersion.current === currentRequest) {
        setSearchState({ status: "success", results });
      }
    } catch (error: unknown) {
      if (requestVersion.current === currentRequest) {
        setSearchState({ status: "error", message: errorMessage(error) });
      }
    }
  };

  const handleInputKeyDown = (
    event: ReactKeyboardEvent<HTMLInputElement>,
  ): void => {
    if (event.key === "Escape") {
      event.preventDefault();
      closeSearch();
    }
  };

  return (
    <div className="project-search">
      <button
        aria-expanded={isOpen}
        aria-keyshortcuts="Control+Shift+F Meta+Shift+F"
        className="project-search__open"
        disabled={projectPath === null}
        onClick={() => setIsOpen(true)}
        title="Buscar no projeto (Ctrl+Shift+F)"
        type="button"
      >
        Buscar no projeto
      </button>

      {isOpen && (
        <section
          aria-label="Busca no projeto"
          className="project-search__panel"
        >
          <form
            aria-label="Buscar em todos os documentos"
            className="project-search__form"
            onSubmit={handleSubmit}
            role="search"
          >
            <label htmlFor="project-search-query">Buscar no projeto</label>
            <div className="project-search__controls">
              <input
                autoComplete="off"
                id="project-search-query"
                onChange={(event) => setQuery(event.target.value)}
                onKeyDown={handleInputKeyDown}
                ref={inputRef}
                type="search"
                value={query}
              />
              <button
                disabled={
                  query.trim().length === 0 || searchState.status === "loading"
                }
                type="submit"
              >
                {searchState.status === "loading" ? "Buscando…" : "Buscar"}
              </button>
              <button
                aria-label="Fechar busca"
                onClick={closeSearch}
                type="button"
              >
                ×
              </button>
            </div>
          </form>

          <div aria-live="polite" className="project-search__results">
            {searchState.status === "error" && (
              <p className="project-search__error" role="alert">
                {searchState.message}
              </p>
            )}
            {searchState.status === "success" &&
              searchState.results.matches.length === 0 && (
                <p>Nenhum resultado encontrado.</p>
              )}
            {searchState.status === "success" &&
              searchState.results.matches.length > 0 && (
                <>
                  <p className="project-search__summary">
                    {searchState.results.matches.length}{" "}
                    {searchState.results.matches.length === 1
                      ? "resultado"
                      : "resultados"}
                    {searchState.results.truncated &&
                      " — refine a busca para ver ocorrências adicionais"}
                  </p>
                  <ol className="project-search__list">
                    {searchState.results.matches.map((match, index) => (
                      <li key={`${match.documentId}:${match.from}:${match.to}`}>
                        <button
                          aria-label={`Abrir ocorrência ${index + 1} em ${match.documentTitle}`}
                          onClick={() => {
                            closeSearch();
                            onNavigate(match);
                          }}
                          type="button"
                        >
                          <strong>{match.documentTitle}</strong>
                          <span>{match.snippet}</span>
                        </button>
                      </li>
                    ))}
                  </ol>
                </>
              )}
          </div>
        </section>
      )}
    </div>
  );
}
