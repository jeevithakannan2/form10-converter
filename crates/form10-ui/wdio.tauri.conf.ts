import { spawn, spawnSync, type ChildProcess } from 'node:child_process';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const directory = fileURLToPath(new URL('.', import.meta.url));
let driver: ChildProcess | undefined;
let shuttingDown = false;

function closeDriver() {
  shuttingDown = true;
  driver?.kill();
}

export const config = {
  hostname: '127.0.0.1',
  port: 4444,
  specs: ['./e2e/tauri/**/*.spec.ts'],
  maxInstances: 1,
  capabilities: [{
    maxInstances: 1,
    'tauri:options': {
      application: path.resolve(directory, 'src-tauri/target/debug/form10-converter')
    }
  }],
  autoCompileOpts: {
    autoCompile: true,
    tsNodeOpts: {
      project: './tsconfig.json',
      transpileOnly: true
    }
  },
  logLevel: 'warn',
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: {
    ui: 'bdd',
    timeout: 60_000
  },
  onPrepare: () => {
    const result = spawnSync('cargo', ['build', '-p', 'form10-ui'], {
      cwd: path.resolve(directory, '../..'),
      stdio: 'inherit'
    });
    if (result.status !== 0) throw new Error('Could not build the Tauri desktop application.');
  },
  beforeSession: () => {
    const executable = process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver';
    driver = spawn(path.join(os.homedir(), '.cargo', 'bin', executable), [], { stdio: 'inherit' });
    driver.on('error', (error) => {
      console.error('Could not start tauri-driver:', error);
      process.exit(1);
    });
    driver.on('exit', (code) => {
      if (!shuttingDown && code !== 0) {
        console.error(`tauri-driver exited unexpectedly with code ${code}.`);
        process.exit(1);
      }
    });
  },
  afterSession: closeDriver,
  onComplete: closeDriver
};
