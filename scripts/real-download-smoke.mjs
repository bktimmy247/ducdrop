import { spawn } from 'node:child_process';
import { mkdir, rm, readdir, stat, writeFile } from 'node:fs/promises';
import path from 'node:path';

const root = process.cwd();
const binDir = path.join(root, 'src-tauri', 'binaries');
const ytdlp = path.join(binDir, 'yt-dlp-x86_64-pc-windows-msvc.exe');
const ffmpeg = path.join(binDir, 'ffmpeg-x86_64-pc-windows-msvc.exe');
const ffprobe = ffmpeg; // ffmpeg bundle may not include ffprobe; use ffmpeg -i fallback.
const outRoot = path.join(root, 'tmp', 'real-download-smoke');

const urls = [
  { id: 'youtube-video', platform: 'youtube', mode: 'video', url: 'https://www.youtube.com/watch?v=kPa7bsKwL-c', expect: 'media' },
  { id: 'youtube-audio', platform: 'youtube', mode: 'audio', url: 'https://www.youtube.com/watch?v=kPa7bsKwL-c', expect: 'audio' },
  { id: 'youtube-thumbnail', platform: 'youtube', mode: 'thumbnail', url: 'https://www.youtube.com/watch?v=kPa7bsKwL-c', expect: 'image' },
  { id: 'youtube-subtitle', platform: 'youtube', mode: 'subtitle', url: 'https://www.youtube.com/watch?v=dQw4w9WgXcQ', expect: 'subtitle' },
  // Public YouTube playlist. Test only first 2 items to avoid burning bandwidth.
  { id: 'youtube-playlist', platform: 'youtube', mode: 'channel', url: 'https://www.youtube.com/playlist?list=PLMC9KNkIncKtPzgY-5rmhvj7fax8fdxoj', expect: 'media', extra: ['--playlist-items', '1:2'] },
  // Public URLs that may change; script records platform limitation instead of pretending pass.
  { id: 'tiktok-video', platform: 'tiktok', mode: 'video', url: 'https://www.tiktok.com/@tiktok/video/7106594312292453678', expect: 'media', optional: true },
  { id: 'facebook-video', platform: 'facebook', mode: 'video', url: 'https://www.facebook.com/facebook/videos/10153231379946729/', expect: 'media', optional: true },
];

function modeArgs(mode) {
  if (mode === 'audio') return ['-x', '--audio-format', 'mp3'];
  if (mode === 'thumbnail') return ['--skip-download', '--write-thumbnail', '--convert-thumbnails', 'jpg'];
  if (mode === 'subtitle') return ['--skip-download', '--write-subs', '--write-auto-subs', '--sub-lang', 'en', '--convert-subs', 'srt'];
  return ['-f', 'bv*[ext=mp4][vcodec^=avc1]+ba[ext=m4a]/b[ext=mp4]/bv*+ba/b', '--merge-output-format', 'mp4'];
}

function run(cmd, args, opts = {}) {
  return new Promise((resolve) => {
    const child = spawn(cmd, args, { cwd: root, windowsHide: true });
    let stdout = '', stderr = '';
    child.stdout.on('data', (d) => stdout += d.toString());
    child.stderr.on('data', (d) => stderr += d.toString());
    child.on('close', (code) => resolve({ code, stdout, stderr, cmd, args }));
  });
}

async function listFiles(dir) {
  const out = [];
  async function walk(d) {
    for (const name of await readdir(d)) {
      const p = path.join(d, name);
      const s = await stat(p);
      if (s.isDirectory()) await walk(p);
      else out.push({ path: p, size: s.size });
    }
  }
  await walk(dir).catch(() => {});
  return out;
}

async function main() {
  await rm(outRoot, { recursive: true, force: true });
  await mkdir(outRoot, { recursive: true });
  const results = [];

  for (const t of urls) {
    const caseDir = path.join(outRoot, t.id);
    await mkdir(caseDir, { recursive: true });
    const tpl = path.join(caseDir, '%(extractor_key|Web)s', '%(uploader|Unknown)s', '%(playlist_index|000)s - %(title).120s.%(ext)s');
    const args = ['--newline', '--no-mtime', '--restrict-filenames', '-o', tpl, '--ffmpeg-location', ffmpeg];
    if (t.mode === 'channel') args.push('--yes-playlist'); else args.push('--no-playlist');
    if (t.extra) args.push(...t.extra);
    args.push(...modeArgs(t.mode), t.url);

    const started = new Date().toISOString();
    const r = await run(ytdlp, args);
    const files = await listFiles(caseDir);
    const media = files.filter(f => f.size > 1024);
    let ok = r.code === 0 && media.length > 0;
    let codec = '';
    const firstVideo = media.find(f => /\.(mp4|mkv|webm)$/i.test(f.path));
    if (firstVideo) {
      const pr = await run(ffmpeg, ['-hide_banner', '-i', firstVideo.path]);
      codec = (pr.stderr || pr.stdout).split('\n').filter(l => /Video:|Audio:/.test(l)).join(' | ');
      if (t.mode === 'video' || t.mode === 'channel') {
        ok = ok && /Video:/.test(codec);
      }
    }
    results.push({
      id: t.id,
      platform: t.platform,
      mode: t.mode,
      url: t.url,
      optional: !!t.optional,
      started,
      code: r.code,
      ok,
      files: files.map(f => ({ path: path.relative(outRoot, f.path), size: f.size })),
      codec,
      stderrTail: r.stderr.split('\n').slice(-20).join('\n'),
      stdoutTail: r.stdout.split('\n').slice(-20).join('\n'),
    });
    console.log(`${ok ? 'PASS' : (t.optional ? 'LIMIT' : 'FAIL')} ${t.id} files=${files.length} code=${r.code}`);
  }

  const report = { generatedAt: new Date().toISOString(), outRoot, results };
  await writeFile(path.join(outRoot, 'report.json'), JSON.stringify(report, null, 2), 'utf8');
  const requiredFailed = results.filter(r => !r.ok && !r.optional);
  console.log(`REPORT ${path.join(outRoot, 'report.json')}`);
  if (requiredFailed.length) process.exit(1);
}

main().catch(e => { console.error(e); process.exit(1); });
