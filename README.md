# DucDrop

**Tha link. Bam tai. Xong.**

DucDrop is a desktop downloader for normal users: paste a public video link, choose what to download, and get the file in `Downloads/DucDrop`.

## Features

- Video download with MP4/H.264/AAC preference.
- Audio extraction to MP3.
- Thumbnail download and JPG conversion.
- Subtitle download to SRT when available.
- Playlist/channel mode with organized output folders.
- Cancel active downloads and clear the queue without leaving yt-dlp/ffmpeg running.
- Bundled `yt-dlp` + `ffmpeg` sidecars so users do not need CLI setup.

## Platforms

Current release focus:

- Windows: NSIS installer, tested locally.
- macOS: GitHub Actions builds `.dmg` / `.app` for Apple Silicon and Intel.
- Linux: GitHub Actions builds `.AppImage` / `.deb` for x64.

Note: macOS builds are unsigned unless Apple Developer signing/notarization secrets are added later, so Gatekeeper may warn on first open.

## Development

```bash
pnpm install
pnpm check
pnpm build
pnpm tauri dev
```

## Sidecar binaries

Tauri sidecars live in `src-tauri/binaries/` and must match the target triple naming convention:

```text
yt-dlp-x86_64-pc-windows-msvc.exe
ffmpeg-x86_64-pc-windows-msvc.exe
yt-dlp-aarch64-apple-darwin
ffmpeg-aarch64-apple-darwin
yt-dlp-x86_64-apple-darwin
ffmpeg-x86_64-apple-darwin
yt-dlp-x86_64-unknown-linux-gnu
ffmpeg-x86_64-unknown-linux-gnu
```

Prepare the current platform sidecars:

```bash
pnpm prepare:sidecars
```

Override platform/arch for CI or testing:

```bash
DUCDROP_TARGET_PLATFORM=darwin DUCDROP_TARGET_ARCH=arm64 pnpm prepare:sidecars
DUCDROP_TARGET_PLATFORM=linux DUCDROP_TARGET_ARCH=x64 pnpm prepare:sidecars
```

## Build desktop bundles

Windows local:

```bash
pnpm build:desktop -- --bundles nsis
```

macOS/Linux real artifacts are built by GitHub Actions:

```text
.github/workflows/desktop-release.yml
```

Run it manually with **Actions -> DucDrop desktop builds -> Run workflow**, or push a tag like `v1.1.4`.

## License notes

DucDrop is a clean-room personal product by Cuong Duc. `yt-dlp` and `ffmpeg` keep their original licenses.