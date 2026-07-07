# Sidecar binaries

Tauri yêu cầu sidecar đặt tên kèm **target triple**. Trên Windows (x64):

```
yt-dlp-x86_64-pc-windows-msvc.exe
ffmpeg-x86_64-pc-windows-msvc.exe
```

Các file `.exe` này KHÔNG commit vào git (xem `.gitignore`). Cách lấy:

- **yt-dlp**: tải bản standalone mới nhất
  https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe
  rồi đổi tên thành `yt-dlp-x86_64-pc-windows-msvc.exe`.
- **ffmpeg**: lấy `ffmpeg.exe` (build static) và đổi tên thành
  `ffmpeg-x86_64-pc-windows-msvc.exe`.

`tauri.conf.json` khai báo chúng trong `bundle.externalBin`, nên khi
`pnpm tauri build` chúng được nhét vào bộ cài và giải nén cạnh app —
app tự tìm thấy khi chạy.
