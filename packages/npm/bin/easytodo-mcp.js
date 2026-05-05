#!/usr/bin/env node
const { spawnSync } = require('child_process');
const path = require('path');

const ext = process.platform === 'win32' ? '.exe' : '';
const binPath = path.join(__dirname, '..', 'vendor', 'easytodo-mcp' + ext);

const result = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit' });

if (result.error) {
    console.error(result.error);
    process.exit(1);
}

process.exit(result.status === null ? 1 : result.status);
