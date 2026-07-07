# DucDrop

**Thả link. Bấm tải. Xong.**

DucDrop là ứng dụng desktop tải video / nhạc / ảnh cực đơn giản cho người bình thường:
dán link → bấm một nút → xong. Không thuật ngữ codec/bitrate, không menu rối.

## Tính năng
- **Tải thông minh**: tự chọn chất lượng tốt nhất.
- **Đẹp nhất**: video + audio chất lượng cao (MP4).
- **Chỉ nghe (MP3)**: tách audio ra MP3.
- **Nhẹ máy**: bản 720p gọn nhẹ.
- Danh sách tải có tiến độ %, tốc độ, nút *Mở thư mục* khi xong.
- File lưu vào `Downloads/DucDrop`.

## Công nghệ lõi
- **Tauri v2** (Rust) + **SvelteKit** (TypeScript) — EXE gọn nhẹ.
- **yt-dlp** + **ffmpeg** đóng gói sẵn dạng sidecar → chạy là tải được ngay, không cần cài thêm.

## Phát triển
```bash
pnpm install
pnpm check        # svelte-check
pnpm build        # vite build
pnpm tauri dev    # chạy thử app
pnpm tauri build --bundles nsis   # đóng gói EXE cài đặt
```

Đặt engine vào `src-tauri/binaries/` trước khi build (xem README trong đó).

## Bản quyền
Sản phẩm cá nhân của Cường Đức. yt-dlp và ffmpeg giữ giấy phép gốc của chúng.
