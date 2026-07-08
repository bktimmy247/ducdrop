# DucDrop Product Soul

DucDrop is not a generic downloader. DucDrop is a small creator tool with one promise:

> Thả link. Bắt file. Xong.

It should feel like a fast, reliable Vietnamese creator assistant: practical, energetic, clean, and a little playful — but never messy.

## Positioning

**For:** Vietnamese creators, sellers, course makers, editors, and operators who collect public media for legitimate reuse, editing, archiving, or reference.

**Job to be done:** Paste a link, choose what they want, download the usable file fast, and know what happened if a platform blocks it.

**One-line pitch:** DucDrop turns messy media links into clean local files.

## Personality

- **Fast:** short flows, clear status, no heavy setup screen before the first download.
- **Street-smart:** understands real creator work: videos, audio, thumbnails, subtitles, batches.
- **Trustworthy:** shows engine health, file location, and errors honestly.
- **Friendly:** Vietnamese microcopy, light charm, no corporate stiffness.
- **Not pirate-coded:** does not encourage copyright abuse, paywall bypassing, or credential theft.

## Voice

Use Vietnamese-first copy. Keep it short and action-oriented.

Good:
- `Thả link vào đây`
- `Bắt link`
- `Tải bản đẹp`
- `Tách nhạc`
- `Mở kho tải`
- `Nền tảng này cần cookie. Anh nhập cookie rồi thử lại.`

Avoid:
- Long technical errors on the main screen.
- Fake success states.
- English-only developer labels for normal users.
- Jokes when download fails, credentials are involved, or files may be lost.

## Visual Identity

### Core motif

A **drop** carrying content home: link goes in, clean file drops out.

Possible mascot directions:
1. **DucDrop Drop:** a glossy electric-blue drop with a small lightning cut.
2. **Cò Ship File:** a stork/courier carrying a media file bundle; fits the Cò worker bot universe.
3. **Magnet Drop:** a magnet/drop hybrid that catches links.

Recommended for v1.1: **DucDrop Drop** as primary icon, `Cò` mascot only in onboarding/help moments.

### Color

Keep the product simple: 2 brand colors + neutrals.

- Primary: Electric cyan `#18C7FF`
- Accent: Deep violet `#7C3AED`
- Background dark: Ink `#07111F`
- Surface: Slate `#0F172A`
- Success: Mint `#2EE59D`
- Warning: Amber `#F59E0B`
- Error: Rose `#F43F5E`

### Typography feel

- Big confident hero title.
- Rounded, modern controls.
- Mono only for logs/advanced details.

## UX Principles

1. **One main action:** paste/drop link first.
2. **Preview before download:** show title, source, thumbnail if available, duration/quality when possible.
3. **Modes are human:** `Video`, `Audio`, `Thumbnail`, `Subtitle`, `Batch` — not raw flags.
4. **Queue is visible:** users should know what is downloading, done, failed, and where the file went.
5. **Advanced stays advanced:** logs, CLI args, engine versions, cookies live behind panels.
6. **Every error gives next step:** what happened + what to do.

## Empty / Loading / Success / Error Copy

### Empty

- Title: `Thả link vào đây`
- Body: `DucDrop sẽ bắt link, xem trước nội dung và tải về đúng định dạng anh chọn.`
- Hint: `Hỗ trợ YouTube, TikTok, Facebook, Instagram và nhiều nền tảng qua yt-dlp.`

### Loading

- `Đang bắt link...`
- `Đang hỏi engine tải file...`
- `Đang chuẩn bị bản đẹp nhất...`

### Success

- `Đã tải xong.`
- `File nằm trong kho tải DucDrop.`
- CTA: `Mở thư mục`

### Error

Format: `Không tải được vì <reason>. <next action>.`

Examples:
- `Không tải được vì nền tảng yêu cầu đăng nhập. Anh thêm cookie rồi thử lại.`
- `Không tìm thấy FFmpeg. Anh bấm Cài engine để DucDrop tự chuẩn bị.`
- `Link này không được yt-dlp hỗ trợ. Anh thử link gốc hoặc đổi nền tảng khác.`

## Product Boundaries

DucDrop should help users download content they have rights to access and use. It must not market itself as a piracy, paywall-bypass, or account-abuse tool.

## v1.1 Soul Checklist

- Main screen says `Thả link. Bắt file. Xong.`
- User can understand the next action within 3 seconds.
- Normal UI is Vietnamese-first.
- Engine/log details are available but not central.
- Failures are calm and actionable.
- Visual identity uses drop/cyan/violet consistently.
