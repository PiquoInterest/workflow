import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { findWorkflowDataDir } from './check-data-dir';

async function withTempDir(
  run: (root: string) => Promise<void>
): Promise<void> {
  const root = await mkdtemp(join(tmpdir(), 'workflow-data-dir-security-'));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

describe('findWorkflowDataDir security boundaries', () => {
  it('rejects a regular file passed as a workflow data directory', async () => {
    await withTempDir(async (root) => {
      const candidate = join(root, '.workflow-data');
      await writeFile(candidate, 'not a directory');

      const result = await findWorkflowDataDir(candidate);

      expect(result.dataDir).toBeUndefined();
      expect(result.error).toBe('Folder does not exist');
    });
  });

  it('ignores a regular file stored at a workflow data candidate path', async () => {
    await withTempDir(async (root) => {
      const project = join(root, 'project');
      await mkdir(project, { recursive: true });
      await writeFile(join(project, '.workflow-data'), 'not a directory');

      const result = await findWorkflowDataDir(project);

      expect(result.projectDir).toBe(project);
      expect(result.dataDir).toBeUndefined();
      expect(result.error).toBeUndefined();
    });
  });

  it('does not classify a dot-suffix lookalike as .workflow-data', async () => {
    await withTempDir(async (root) => {
      const lookalike = join(root, 'not.workflow-data');
      await mkdir(lookalike, { recursive: true });

      const result = await findWorkflowDataDir(lookalike);

      expect(result.projectDir).toBe(lookalike);
      expect(result.dataDir).toBeUndefined();
    });
  });

  it('does not classify a name suffix lookalike as workflow-data', async () => {
    await withTempDir(async (root) => {
      const lookalike = join(root, 'not-workflow-data');
      await mkdir(lookalike, { recursive: true });

      const result = await findWorkflowDataDir(lookalike);

      expect(result.projectDir).toBe(lookalike);
      expect(result.dataDir).toBeUndefined();
    });
  });
});
