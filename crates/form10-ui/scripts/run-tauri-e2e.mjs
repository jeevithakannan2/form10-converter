import { spawnSync } from 'node:child_process';

if (process.platform === 'darwin') {
  console.log('Skipping native Tauri E2E: tauri-driver does not support macOS WKWebView.');
  process.exit(0);
}

const command = process.platform === 'win32' ? 'wdio.cmd' : 'wdio';
const result = spawnSync(command, ['run', 'wdio.tauri.conf.ts'], {
  cwd: new URL('..', import.meta.url),
  stdio: 'inherit',
  shell: process.platform === 'win32'
});

process.exit(result.status ?? 1);
