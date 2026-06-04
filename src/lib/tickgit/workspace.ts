import type {
  CommitFileDiffResult,
  WorkspaceChangeSection,
  WorkspaceFileChange,
  WorkspaceStatus,
} from "$lib/types";
import { EMPTY_DIFF_RESULT } from "$lib/tickgit/page-data";

export const EMPTY_WORKSPACE_STATUS: WorkspaceStatus = {
  staged: [],
  unstaged: [],
};

export type WorkspaceSelection = {
  section: WorkspaceChangeSection;
  path: string;
};

export type WorkspaceApi = {
  getWorkspaceStatus: (repoPath: string) => Promise<WorkspaceStatus>;
  getWorkspaceFileDiff: (
    repoPath: string,
    section: WorkspaceChangeSection,
    filePath: string,
    ignoreWhitespace?: boolean,
    previousPath?: string | null,
  ) => Promise<CommitFileDiffResult>;
};

export type WorkspaceSnapshot = {
  status: WorkspaceStatus;
  selectedFile: WorkspaceFileChange | null;
  selectedSection: WorkspaceChangeSection | null;
  selectedFilePath: string | null;
  diffResult: CommitFileDiffResult;
};

export type WorkspaceCommitEffect = {
  nextCommitMessage: string;
  refreshWorkspace: boolean;
  refreshRepository: boolean;
};

export function workspaceFileKey(
  selection: WorkspaceSelection | WorkspaceFileChange,
) {
  return `${selection.section}:${selection.path}`;
}

export function findWorkspaceFile(
  status: WorkspaceStatus,
  selection: WorkspaceSelection | null,
) {
  if (!selection) {
    return null;
  }

  return (
    [...status.staged, ...status.unstaged].find(
      (file) =>
        file.section === selection.section && file.path === selection.path,
    ) ?? null
  );
}

export function pickWorkspaceFile(
  status: WorkspaceStatus,
  previousSelection: WorkspaceSelection | null,
  keepSelection: boolean,
) {
  if (keepSelection) {
    const previousFile = findWorkspaceFile(status, previousSelection);

    if (previousFile) {
      return previousFile;
    }
  }

  return status.staged[0] ?? status.unstaged[0] ?? null;
}

export async function fetchWorkspaceSnapshot(
  api: WorkspaceApi,
  repoPath: string,
  keepSelection: boolean,
  previousSelection: WorkspaceSelection | null,
  ignoreWhitespace = false,
): Promise<WorkspaceSnapshot> {
  const status = await api.getWorkspaceStatus(repoPath);
  const selectedFile = pickWorkspaceFile(
    status,
    previousSelection,
    keepSelection,
  );

  if (!selectedFile) {
    return {
      status,
      selectedFile: null,
      selectedSection: null,
      selectedFilePath: null,
      diffResult: EMPTY_DIFF_RESULT,
    };
  }

  const diffResult = await api.getWorkspaceFileDiff(
    repoPath,
    selectedFile.section,
    selectedFile.path,
    ignoreWhitespace,
    selectedFile.previousPath,
  );

  return {
    status,
    selectedFile,
    selectedSection: selectedFile.section,
    selectedFilePath: selectedFile.path,
    diffResult,
  };
}

export function getWorkspaceCommitSuccessEffect(): WorkspaceCommitEffect {
  return {
    nextCommitMessage: "",
    refreshWorkspace: true,
    refreshRepository: true,
  };
}

export function getWorkspaceCommitFailureEffect(
  commitMessage: string,
): WorkspaceCommitEffect {
  return {
    nextCommitMessage: commitMessage,
    refreshWorkspace: false,
    refreshRepository: false,
  };
}
