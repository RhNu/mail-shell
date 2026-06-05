import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { spawnSync } from 'node:child_process';

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: resolve('.'),
    encoding: 'utf8',
    stdio: 'inherit',
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}

const workspaceRoot = resolve('.');
const tempDir = mkdtempSync(join(tmpdir(), 'mail-shell-openapi-'));
const openapiPath = join(tempDir, 'openapi.json');
const openapiTypescriptCli = join(
  workspaceRoot,
  'node_modules',
  'openapi-typescript',
  'bin',
  'cli.js',
);
const outputPath = join(workspaceRoot, 'client', 'src', 'api', 'generated', 'schema.d.ts');

try {
  run('cargo', ['run', '-p', 'mail-shell-server', '--bin', 'export_openapi', '--', openapiPath]);
  run(process.execPath, [openapiTypescriptCli, openapiPath, '-o', outputPath]);
} finally {
  rmSync(tempDir, { recursive: true, force: true });
}
