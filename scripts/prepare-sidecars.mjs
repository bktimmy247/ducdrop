#!/usr/bin/env node
import { createWriteStream } from 'node:fs';
import { chmod, mkdir, rename, rm, stat } from 'node:fs/promises';
import { dirname, join, resolve } from 'node:path';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';
import { createGunzip } from 'node:zlib';
import { spawnSync } from 'node:child_process';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const binDir = join(root, 'src-tauri', 'binaries');

const platform = process.env.DUCDROP_TARGET_PLATFORM || process.platform;
const arch = process.env.DUCDROP_TARGET_ARCH || process.arch;
const version = process.env.YTDLP_VERSION || '2026.07.04';

const triples = {
  win32: { x64: { triple: 'x86_64-pc-windows-msvc', ext: '.exe' } },
  darwin: { x64: { triple: 'x86_64-apple-darwin', ext: '' }, arm64: { triple: 'aarch64-apple-darwin', ext: '' } },
  linux: { x64: { triple: 'x86_64-unknown-linux-gnu', ext: '' }, arm64: { triple: 'aarch64-unknown-linux-gnu', ext: '' } },
};

const ffmpegUrls = {
  win32: {
    x64: 'https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip',
  },
  darwin: {
    x64: 'https://evermeet.cx/ffmpeg/getrelease/zip',
    arm64: 'https://evermeet.cx/ffmpeg/getrelease/zip',
  },
  linux: {
    x64: 'https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz',
    arm64: 'https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz',
  },
};

const ytDlpNames = {
  win32: { x64: 'yt-dlp.exe' },
  darwin: { x64: 'yt-dlp_macos', arm64: 'yt-dlp_macos' },
  linux: { x64: 'yt-dlp_linux', arm64: 'yt-dlp_linux_aarch64' },
};

function targetInfo() {
  const p = triples[platform]?.[arch];
  if (!p) throw new Error(`Unsupported platform/arch: ${platform}/${arch}`);
  return p;
}

async function exists(path) {
  try { await stat(path); return true; } catch { return false; }
}

async function download(url, dest) {
  console.log(`download ${url}`);
  const res = await fetch(url, { redirect: 'follow' });
  if (!res.ok) throw new Error(`Download failed ${res.status}: ${url}`);
  await mkdir(dirname(dest), { recursive: true });
  await pipeline(res.body, createWriteStream(dest));
}

function run(cmd, args, opts = {}) {
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: process.platform === 'win32', ...opts });
  if (res.status !== 0) throw new Error(`${cmd} ${args.join(' ')} failed`);
}

async function prepareYtDlp(triple, ext) {
  const out = join(binDir, `yt-dlp-${triple}${ext}`);
  if (await exists(out)) return console.log(`exists ${out}`);
  const asset = ytDlpNames[platform]?.[arch];
  if (!asset) throw new Error(`No yt-dlp asset for ${platform}/${arch}`);
  const url = `https://github.com/yt-dlp/yt-dlp/releases/download/${version}/${asset}`;
  await download(url, out);
  if (!ext) await chmod(out, 0o755);
  console.log(`ready ${out}`);
}

async function prepareFfmpeg(triple, ext) {
  const out = join(binDir, `ffmpeg-${triple}${ext}`);
  if (await exists(out)) return console.log(`exists ${out}`);
  const url = ffmpegUrls[platform]?.[arch];
  if (!url) throw new Error(`No ffmpeg URL for ${platform}/${arch}`);
  const tmp = join(binDir, `.tmp-ffmpeg-${platform}-${arch}`);
  await rm(tmp, { recursive: true, force: true });
  await mkdir(tmp, { recursive: true });
  const archive = join(tmp, platform === 'linux' ? 'ffmpeg.tar.xz' : 'ffmpeg.zip');
  await download(url, archive);
  if (platform === 'linux') {
    run('tar', ['-xf', archive, '-C', tmp]);
    const find = spawnSync('bash', ['-lc', `find '${tmp.replaceAll("'", "'\\''")}' -type f -name ffmpeg | head -n 1`], { encoding: 'utf8' });
    const src = find.stdout.trim();
    if (!src) throw new Error('ffmpeg binary not found in linux archive');
    await rename(src, out);
  } else {
    if (process.platform === 'win32') {
      run('powershell', ['-NoProfile', '-Command', `Expand-Archive -Force ${JSON.stringify(archive)} ${JSON.stringify(tmp)}`]);
      const find = spawnSync('powershell', ['-NoProfile', '-Command', `Get-ChildItem -Recurse -File ${JSON.stringify(tmp)} | Where-Object {$_.Name -eq 'ffmpeg.exe' -or $_.Name -eq 'ffmpeg'} | Select-Object -First 1 -ExpandProperty FullName`], { encoding: 'utf8' });
      const src = find.stdout.trim();
      if (!src) throw new Error('ffmpeg binary not found in archive');
      await rename(src, out);
    } else {
      run('unzip', ['-q', archive, '-d', tmp]);
      const find = spawnSync('bash', ['-lc', `find '${tmp.replaceAll("'", "'\\''")}' -type f \\( -name ffmpeg -o -name ffmpeg.exe \\) | head -n 1`], { encoding: 'utf8' });
      const src = find.stdout.trim();
      if (!src) throw new Error('ffmpeg binary not found in archive');
      await rename(src, out);
    }
  }
  if (!ext) await chmod(out, 0o755);
  await rm(tmp, { recursive: true, force: true });
  console.log(`ready ${out}`);
}

const { triple, ext } = targetInfo();
await mkdir(binDir, { recursive: true });
await prepareYtDlp(triple, ext);
await prepareFfmpeg(triple, ext);
