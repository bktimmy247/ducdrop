# DucDrop v1.1 Upgrade Brief

## Goal

Upgrade DucDrop from a working MVP into a distinctive, trustworthy desktop product for Vietnamese creators.

Current v1.0.0 status:

- Clean project at `projects/ducdrop`.
- Tauri + Svelte app.
- Bundled `yt-dlp` and FFmpeg sidecars.
- NSIS installer built.
- UI is functional but too simple/generic.

v1.1 should not add every advanced downloader feature. It should make the product feel real, branded, and easier to trust.

## North Star

> DucDrop: Thả link. Bắt file. Xong.

A user should open DucDrop, paste a link, understand the result, and get a useful file without seeing developer noise.

## Non-negotiables

1. **Clean-room:** follow `docs/CLEAN_ROOM_BOUNDARY.md`.
2. **Product soul first:** use `docs/soul.md` for colors, tone, microcopy.
3. **No fake readiness:** if engine/format/cookie is missing, say it clearly.
4. **No piracy framing:** neutral creator utility, not bypass/steal messaging.
5. **Evidence-based release:** Builder must report changed files + commands; Reviewer must run check/build and inspect UI.

## v1.1 Scope

### 1. Main screen redesign

Replace plain form feel with a small dashboard:

- Left/center hero:
  - `DucDrop`
  - `Thả link. Bắt file. Xong.`
  - Big paste/drop zone.
  - Primary CTA: `Bắt link`
- Mode cards:
  - `Video`
  - `Audio`
  - `Thumbnail`
  - `Subtitle` (can be disabled if not implemented)
- Queue/status panel:
  - Waiting/Running/Done/Failed rows.
  - File destination and `Mở thư mục` action.
- Engine health strip:
  - yt-dlp status/version if available.
  - FFmpeg status/version if available.
  - Keep detailed logs behind `Advanced`.

### 2. Preview-first flow

When possible, split flow into two steps:

1. Paste link -> `Bắt link`
2. Show preview -> choose mode/quality -> `Tải xuống`

Minimum preview fields:

- Title or fallback domain/source.
- Platform/source guess.
- Thumbnail if available.
- Duration if available.
- Available output modes.

If full preview is not available yet, show a clear `Preview chưa hỗ trợ cho link này, vẫn có thể tải trực tiếp` state.

### 3. Human microcopy

Use Vietnamese labels from `docs/soul.md`.

Suggested labels:

- Empty: `Thả link vào đây`
- CTA parse: `Bắt link`
- CTA download: `Tải xuống`
- Audio: `Tách nhạc`
- Best video: `Tải bản đẹp`
- Folder: `Mở kho tải`
- Advanced: `Chi tiết kỹ thuật`
- Installing/checking engines: `Đang kiểm tra engine...`

### 4. Visual direction

- Dark premium background with cyan/violet accent.
- Rounded cards, clear hierarchy.
- A simple drop icon/mark in CSS/SVG, not copied assets.
- Avoid clutter: one main action, advanced details collapsed.

### 5. Error handling polish

Convert raw errors into actionable messages:

| Situation | User-facing message |
|---|---|
| yt-dlp missing | `Chưa tìm thấy engine tải. Anh bấm Cài engine hoặc build lại bản có sidecar.` |
| FFmpeg missing | `Chưa tìm thấy FFmpeg nên chưa ghép/chuyển định dạng được.` |
| Login/cookie needed | `Nền tảng này cần đăng nhập. Anh thêm cookie rồi thử lại.` |
| Unsupported URL | `DucDrop chưa bắt được link này. Anh thử link gốc hoặc nền tảng khác.` |
| Download failed | `Tải chưa thành công. Anh mở Chi tiết kỹ thuật để xem log.` |

### 6. Minimal product analytics/logging locally

No cloud analytics in v1.1. If logging is needed:

- Local-only `logs/` or app data log.
- No tokens/cookies in logs.
- Show recent technical log only in Advanced.

## Suggested file/module plan

Builder can adapt to current repo, but recommended shape:

```text
src/
  routes/+page.svelte              # main screen composition
  lib/brand.ts                     # colors/copy constants if useful
  lib/ducdrop-copy.ts              # UI copy keys
  lib/tauri.ts                     # existing bridge
  lib/types.ts                     # task/preview/engine types
src-tauri/src/
  lib.rs / commands                # keep minimal commands
  engine.rs                        # DucDrop-owned ytdlp/ffmpeg wrapper if needed
```

Do not over-architect if the current app is small. v1.1 is polish + clarity first.

## Acceptance Criteria for Builder

Builder is done only when:

- `pnpm check` passes.
- `pnpm build` passes.
- App main screen uses DucDrop soul: tagline, cyan/violet dark theme, drop-zone, mode cards, queue/status area.
- No OmniGet code/text/assets copied.
- UI labels are Vietnamese-first and useful.
- Engine status is visible but technical logs are not the main UI.
- Failure states show next actions.
- Builder report includes changed files and screenshots or a short screen recording if possible.

## Reviewer Checklist

Reviewer should verify:

### Product/UX

- Can a non-technical user understand what to do in 3 seconds?
- Does it feel like DucDrop, not a generic link form?
- Are primary/secondary actions clear?
- Are empty/loading/success/error states present?

### Technical

- `pnpm check` pass.
- `pnpm build` pass.
- Tauri build not broken if touched.
- Sidecar paths still work.
- No secrets/log-sensitive data exposed.

### Legal boundary

- No OmniGet copied source, UI text, folder names, assets, or docs.
- `docs/CLEAN_ROOM_BOUNDARY.md` remains in repo.

## v1.2 Ideas (not required now)

- Cookie manager with clear safe UX.
- Batch links queue.
- Download history.
- Quality picker with recommended default.
- Subtitle extraction.
- Audio-only presets.
- Auto-update yt-dlp.
- Browser extension / clipboard watcher.

## Task prompt for Cò 1 / Builder

Use this when assigning implementation:

> Cò 1, upgrade DucDrop v1.0.0 to v1.1 product UI. Read `docs/soul.md`, `docs/CLEAN_ROOM_BOUNDARY.md`, and `docs/DUCDROP_UPGRADE_BRIEF.md`. Do not copy OmniGet code/assets/text. Focus on main screen redesign, Vietnamese microcopy, engine status, mode cards, queue/status states, and actionable errors. Run `pnpm check` and `pnpm build`. Report files changed, commands, evidence, blockers.

## Task prompt for Cò 2 / Reviewer

Use this after Builder finishes:

> Cò 2, review DucDrop v1.1 changes. Read `docs/CLEAN_ROOM_BOUNDARY.md` and `docs/DUCDROP_UPGRADE_BRIEF.md`. Verify no OmniGet copying, run `pnpm check` and `pnpm build`, inspect the UI against the product soul, and return PASS / PASS WITH NOTES / FAIL with evidence and action items.
