#!/usr/bin/env node
const { spawn } = require('child_process');
const path = require('path');

// Path to the project root
const projectRoot = 'c:\\ORION';

// Find electron executable in node_modules
const electronPath = path.join(projectRoot, 'node_modules', '.bin', process.platform === 'win32' ? 'electron.cmd' : 'electron');

// Spawn electron
const child = spawn(electronPath, ['.'], {
  detached: true,
  stdio: 'ignore',
  cwd: projectRoot,
  shell: true // Required for Windows .cmd files
});

child.unref();
process.exit(0);
