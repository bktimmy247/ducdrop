# DucDrop Clean-Room Boundary

This project is a clean DucDrop implementation. OmniGet may be used only as a product/architecture reference, not as a source-code donor.

## Why this exists

The public OmniGet repository declares `GPL-3.0`. GPL is a strong copyleft license. If DucDrop copies or derives source code from OmniGet and is distributed as an app, that can create obligations that conflict with a closed/commercial product plan.

DucDrop must therefore be built as an independent implementation.

## Allowed reference use

It is OK to study OmniGet at a high level for:

- Feature inventory: what categories of downloader features exist.
- Architecture ideas: desktop app + download engine + queue + settings.
- Common open tools: yt-dlp, FFmpeg, Tauri/Svelte/Rust ecosystem.
- User problems: cookies, quality choice, queue, subtitle/audio workflows.
- UX principles: preview before download, visible progress, recoverable failures.

## Not allowed

Do not copy from OmniGet:

- Source files, functions, modules, class names, exact folder structure.
- UI layout, component hierarchy, CSS, icons, copy, or translations.
- Rust command implementations, queue logic, plugin loader code, cookie parsers, Bilibili/platform-specific code.
- Test files, build scripts, docs text, README text, or screenshots as assets.
- Any code with small renames or superficial changes.

## Implementation rule for Builder

For every DucDrop feature, implement from first principles using public documentation for the underlying tools:

- `yt-dlp --help` / official docs for extraction and formats.
- FFmpeg docs for muxing/conversion.
- Tauri 2 docs for command bridge, sidecars, filesystem, shell/process.
- Svelte docs for UI state and components.

If a feature was inspired by OmniGet, describe the inspiration in a task note, then write new code without looking at the original implementation while coding.

## Recommended source hygiene

- Keep this file in the repo.
- Do not paste OmniGet snippets into issues, docs, prompts, comments, or code.
- Name DucDrop modules in DucDrop language, e.g. `drop-engine`, `link-catcher`, `download-queue`, not OmniGet names.
- Use original UI copy from `docs/soul.md`.
- Add tests/evidence for DucDrop behavior, not comparisons against OmniGet internals.

## Safe feature map

| Product need | Safe DucDrop implementation path |
|---|---|
| Download public video/audio | Spawn bundled `yt-dlp` sidecar with DucDrop-owned argument builder |
| Convert/mux media | Spawn bundled FFmpeg sidecar with minimal, documented commands |
| Queue | Write DucDrop task model: queued/running/done/failed/cancelled |
| Progress | Parse stdout/stderr from tools into DucDrop progress events |
| Cookie support | Start with manual cookie file path; later add import helpers from docs |
| Settings | Store DucDrop settings in app config JSON |
| History | Store DucDrop download records in JSON/SQLite designed by us |
| UI | Build original drop-zone/dashboard using `docs/soul.md` |

## Reviewer checklist

Reviewer should fail a PR if it:

- Contains copied OmniGet source or exact UI text.
- Uses OmniGet file/module names unnecessarily.
- Adds GPL text or OmniGet assets into DucDrop.
- Claims license safety without this boundary being followed.
- Implements advanced platform-specific logic by transplanting code instead of using public APIs/docs.

## Current stance

DucDrop can use the same open ecosystem (`yt-dlp`, FFmpeg, Tauri, Svelte) because those are independent tools/libraries. DucDrop must not use OmniGet GPL code as its own implementation.
