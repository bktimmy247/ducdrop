# DucDrop v1.2 Product Direction

## Positioning

DucDrop is **the easiest way to download anything**.

Not an AI chat downloader. Not a workflow builder. Not a power-user format picker.

The main flow is:

```text
Open app -> paste/copy link -> choose desired result -> download
```

The difference is not more controls. The difference is smarter buttons.

## Product principle

Users should choose the result they want, not the technical format.

Do not expose first:

- MP4 / WebM / MKV
- H.264 / AV1
- bitrate / codec / container
- raw yt-dlp flags

Expose first:

- Smart Download
- Best
- Audio
- Small
- Offline
- Batch

Technical details live in **More options** only.

## Main screen target

```text
[DucDrop]
The easiest way to download anything

+------------------------------------------------+
| Paste one link or many links                   |
+------------------------------------------------+

[ SMART DOWNLOAD ]

[ Best ] [ Audio ] [ Small ] [ Offline ] [ Batch ]

Recent downloads...

More options
```

## Modes

### Smart

Default primary action.

DucDrop chooses the best reasonable behavior:

- Single video -> good quality video + audio.
- Playlist/multiple links -> offer batch/download all.
- Audio/podcast-like link -> audio-first.
- Direct file/PDF/image -> direct file download.
- Unsupported preview -> still allow direct download with a clear warning.

v1.2 implementation can be simple: map to best video and use link count/context hints. The UI should already teach the right behavior.

### Best

Highest quality reasonable video + audio.

User-facing copy: `Đẹp nhất`.

### Audio

Extract audio.

User-facing copy: `Chỉ nghe`.

### Small

Prefer smaller file size while still watchable.

User-facing copy: `Nhẹ máy`.

### Offline

Download video plus useful extras when available: subtitles, thumbnail, metadata.

User-facing copy: `Xem offline`.

### Batch

Download multiple links or all items from a playlist/page when possible.

User-facing copy: `Tải tất cả`.

## Batch links

v1.2 core app should support pasting multiple links separated by new lines.

Behavior:

- One link -> normal mode.
- Multiple links -> show count and make Batch mode prominent.
- Each link should become its own queue item.
- Failure of one link must not stop the whole batch.

## Clipboard download

Roadmap, not required in first v1.2 patch.

Desired future behavior:

- User copies a link.
- DucDrop detects it.
- Small popup/tray action: `Video found — Download`.

Must be opt-in. Do not annoy users by default.

## Download button everywhere

Roadmap:

- Browser extension: `Download with DucDrop`.
- Mobile share sheet: `Share -> DucDrop -> Download`.

This is vNext after the core desktop app feels right.

## Copy voice

Use direct Vietnamese copy:

- `Dán link vào đây`
- `Dán một link hoặc nhiều link, mỗi dòng một link`
- `Tải thông minh`
- `Đẹp nhất`
- `Chỉ nghe`
- `Nhẹ máy`
- `Xem offline`
- `Tải tất cả`
- `Tùy chọn thêm`
- `Tải giống lần trước`

Avoid:

- `Agent`
- `Prompt`
- `Workflow`
- `Pipeline`
- Technical flags on the main screen.

## v1.2 acceptance criteria

- Main screen feels consumer-friendly, not developer/power-user.
- Smart Download is the biggest button.
- Mode buttons use outcome language, not format language.
- Multi-link paste is visible and understandable.
- Recent downloads/queue remains simple.
- Advanced options are collapsed.
- No installer build without anh approving.
