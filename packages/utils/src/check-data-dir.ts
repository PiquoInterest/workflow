import { stat } from 'node:fs/promises';
import { homedir } from 'node:os';
import { dirname, isAbsolute, join, resolve, sep } from 'node:path';

/**
 * Known paths where workflow data might be stored, relative to the project root.
 */
export const possibleWorkflowDataPaths = [
  '.next/workflow-data',
  '.workflow-data',
  'workflow-data',
] as const;

export interface WorkflowDataDirInfo {
  /** Absolute path to the workflow data directory, if found. Absence indicates that the folder does not point to a project or folder within a valid local world. */
  dataDir?: string;
  /** Absolute path to the project root (parent of the workflow data folder) */
  projectDir: string;
  /** Short name for display: up to last two folder names of projectDir */
  shortName: string;
  /** Error message if the given path couldn't be accessed (doesn't exist or is not a directory) */
  error?: string;
}

/**
 * Expands a standalone ~ or a ~/... path to use the user's home directory.
 * User-qualified forms such as ~alice remain ordinary relative paths.
 */
function expandTilde(path: string): string {
  if (path === '~') {
    return homedir();
  }
  if (path.startsWith('~/') || path.startsWith(`~${sep}`)) {
    return join(homedir(), path.slice(2));
  }
  return path;
}

/**
 * Normalizes a path to an absolute path.
 */
function toAbsolutePath(path: string, basePath?: string): string {
  const expanded = expandTilde(path);
  if (isAbsolute(expanded)) {
    return resolve(expanded);
  }
  return resolve(basePath || process.cwd(), expanded);
}

/**
 * Extracts up to the last two folder names from a path for a short display name.
 *
 * Examples:
 * - `/Users/peter/code/myproject` → `code/myproject`
 * - `/myproject` → `myproject`
 * - `/` → `/`
 */
export function getDirShortName(projectDir: string): string {
  const parts = projectDir.split(sep).filter(Boolean);
  if (parts.length === 0) {
    return '/';
  }
  if (parts.length === 1) {
    return parts[0];
  }
  return parts.slice(-2).join('/');
}

/**
 * Checks that a path resolves to a directory. Merely being accessible is not
 * enough because regular files and special files are not workflow stores.
 */
async function directoryExists(path: string): Promise<boolean> {
  try {
    return (await stat(path)).isDirectory();
  } catch {
    return false;
  }
}

/**
 * Checks if the given path is itself a workflow data directory.
 *
 * The comparison is made from complete path components. A raw string suffix
 * check would incorrectly classify names such as `not.workflow-data` and
 * `not-workflow-data` as trusted workflow stores.
 */
function checkIfPathIsWorkflowDataDir(
  absolutePath: string
): { suffix: string; projectDir: string } | null {
  for (const suffix of possibleWorkflowDataPaths) {
    const suffixParts = suffix.split('/');
    let projectDir = absolutePath;
    for (let index = 0; index < suffixParts.length; index++) {
      projectDir = dirname(projectDir);
    }

    if (resolve(projectDir, ...suffixParts) === absolutePath) {
      return { suffix, projectDir };
    }
  }
  return null;
}

/**
 * Finds the workflow data directory starting from the given path.
 *
 * This function handles several cases:
 * 1. The path itself is a workflow data directory
 * 2. The path contains one of the known workflow data directories
 * 3. The path is somewhere inside a project with workflow data
 *
 * @param cwd - The directory to start searching from (can be relative, absolute, or use ~)
 * @returns Information about the found workflow data directory, or an empty result if not found
 */
export async function findWorkflowDataDir(
  cwd: string
): Promise<WorkflowDataDirInfo> {
  const absoluteCwd = toAbsolutePath(cwd);

  if (!(await directoryExists(absoluteCwd))) {
    return {
      projectDir: absoluteCwd,
      dataDir: undefined,
      shortName: getDirShortName(absoluteCwd),
      error: 'Folder does not exist',
    };
  }

  const isDataDir = checkIfPathIsWorkflowDataDir(absoluteCwd);
  if (isDataDir) {
    return {
      projectDir: isDataDir.projectDir,
      dataDir: absoluteCwd,
      shortName: getDirShortName(isDataDir.projectDir),
    };
  }

  for (const path of possibleWorkflowDataPaths) {
    const fullPath = join(absoluteCwd, path);
    if (await directoryExists(fullPath)) {
      return {
        projectDir: absoluteCwd,
        dataDir: resolve(fullPath),
        shortName: getDirShortName(absoluteCwd),
      };
    }
  }

  let currentDir = absoluteCwd;
  while (true) {
    for (const path of possibleWorkflowDataPaths) {
      const fullPath = join(currentDir, path);
      if (await directoryExists(fullPath)) {
        return {
          projectDir: currentDir,
          dataDir: resolve(fullPath),
          shortName: getDirShortName(currentDir),
        };
      }
    }

    const parentDir = dirname(currentDir);
    if (parentDir === currentDir) {
      break;
    }
    currentDir = parentDir;
  }

  return {
    projectDir: absoluteCwd,
    dataDir: undefined,
    shortName: getDirShortName(absoluteCwd),
  };
}
