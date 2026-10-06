import { useState } from "react";

import type {
  WritingProject,
  WritingProjectGateway,
} from "./application/writingProject";
import { WritingWorkspace } from "./features/editor/WritingWorkspace";
import { ProjectLauncher } from "./features/project/ProjectLauncher";

interface AppProps {
  gateway?: WritingProjectGateway;
}

function App({ gateway }: AppProps) {
  const [project, setProject] = useState<WritingProject | null>(null);

  if (project === null) {
    return <ProjectLauncher gateway={gateway} onOpen={setProject} />;
  }

  return (
    <WritingWorkspace
      gateway={gateway}
      key={project.projectId}
      onCloseProject={() => setProject(null)}
      project={project}
    />
  );
}

export default App;
