const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const pkg = require('./package.json');
const REPO = "junaidsultanxyz/easytodo";
const VERSION = pkg.version;

const PLATFORMS = {
  "win32-x64": "x86_64-windows",
  "darwin-x64": "x86_64-macos",
  "darwin-arm64": "aarch64-macos",
  "linux-x64": "x86_64-linux",
  "linux-arm64": "aarch64-linux"
};

const key = `${process.platform}-${process.arch}`;
const target = PLATFORMS[key];

if (!target) {
  console.error(`Unsupported platform/architecture: ${key}`);
  process.exit(1);
}

const url = `https://github.com/${REPO}/releases/download/v${VERSION}/easytodo-${target}.tar.gz`;
const vendorDir = path.join(__dirname, 'vendor');
const archivePath = path.join(__dirname, `easytodo-${target}.tar.gz`);

if (!fs.existsSync(vendorDir)) {
  fs.mkdirSync(vendorDir, { recursive: true });
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      if (res.statusCode === 301 || res.statusCode === 302) {
        return download(res.headers.location, dest).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`Failed to download: ${res.statusCode} ${res.statusMessage}`));
      }
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on('finish', () => {
        file.close();
        resolve();
      });
    }).on('error', (err) => {
      fs.unlink(dest, () => {});
      reject(err);
    });
  });
}

async function install() {
  try {
    console.log(`Downloading easytodo v${VERSION} for ${target}...`);
    await download(url, archivePath);
    
    console.log('Extracting archive...');
    execSync(`tar -xzf "${archivePath}" -C "${vendorDir}"`);
    
    fs.unlinkSync(archivePath);
    
    if (process.platform !== 'win32') {
      execSync(`chmod +x "${path.join(vendorDir, 'easytodo')}"`);
      execSync(`chmod +x "${path.join(vendorDir, 'easytodo-mcp')}"`);
    }
    
    console.log('Successfully installed easytodo!');
  } catch (err) {
    console.error('Installation failed:', err.message);
    process.exit(1);
  }
}

install();
