// Vietnamese-first product copy for DucDrop
// Keep UI simple: paste link, choose result, download.

export const hero = {
  brand: "DucDrop",
  tagline: "Thả link. Bắt file. Xong.",
  dropTitle: "Thả link vào đây",
  dropHint: "Dán link video, playlist hoặc nhiều link. Chọn thứ anh muốn tải rồi bấm tải.",
  platforms: "YouTube, TikTok, Facebook, Instagram và nhiều nền tảng qua yt-dlp.",
};

export const ctas = {
  paste: "Dán",
  catch: "Bắt link",
  smart: "Tải nhanh",
  download: "Tải xuống",
  openFolder: "Mở kho tải",
  advanced: "Tùy chọn thêm",
  installEngine: "Cài engine",
  likeLastTime: "Tải giống lần trước",
};

export const modes = {
  video: { label: "Video", icon: "🎬", desc: "Tải video" },
  audio: { label: "Audio", icon: "🎧", desc: "Tách nhạc" },
  thumbnail: { label: "Thumbnail", icon: "🖼", desc: "Ảnh bìa" },
  subtitle: { label: "Subtitle", icon: "📝", desc: "Phụ đề" },
  channel: { label: "Tất cả", icon: "📚", desc: "Kênh/playlist" },
};

export const states = {
  empty: "Chưa có gì tải cả",
  emptySub: "Dán link ở trên, chọn chế độ rồi tải.",
  queued: "Đang chờ…",
  running: "Đang tải…",
  completed: "Đã tải xong",
  completedSub: "File nằm trong kho tải DucDrop.",
  failed: "Tải chưa thành công",
  failedSub: "Anh mở Tùy chọn thêm để xem log.",
  previewUnsupported:
    "Preview chưa hỗ trợ cho link này, vẫn có thể tải trực tiếp.",
};

export const loading = {
  catch: "Đang bắt link…",
  engine: "Đang kiểm tra engine…",
  best: "Đang chuẩn bị tải…",
};

export const engine = {
  title: "Engine",
  checking: "Đang kiểm tra engine…",
  ytdlpOk: "yt-dlp sẵn sàng",
  ytdlpMissing: "Chưa tìm thấy engine tải.",
  ffmpegOk: "FFmpeg sẵn sàng",
  ffmpegMissing: "Chưa tìm thấy FFmpeg nên chưa ghép/chuyển định dạng được.",
};

export function humanError(err: string): string {
  const e = err.toLowerCase();
  if (e.includes("ffmpeg") || e.includes("merger")) {
    return `Chưa tìm thấy FFmpeg nên chưa ghép/chuyển định dạng được. ${ctas.installEngine} hoặc build lại bản có sidecar.`;
  }
  if (e.includes("yt-dlp") || e.includes("sidecar") || e.includes("command")) {
    return `Chưa tìm thấy engine tải. Anh ${ctas.installEngine} hoặc build lại bản có sidecar.`;
  }
  if (e.includes("cookie") || e.includes("login") || e.includes("sign in")) {
    return `Nền tảng này cần đăng nhập. Anh thêm cookie rồi thử lại.`;
  }
  if (e.includes("unsupported")) {
    return `DucDrop chưa bắt được link này. Anh thử link gốc hoặc nguồn khác.`;
  }
  return `Không tải được vì ${err}. Anh mở Tùy chọn thêm để xem log.`;
}

export const hints = {
  badUrl: "Link chưa đúng. Cần bắt đầu bằng http…",
  noUrl: "Anh dán ít nhất một link vào đã nhé",
  pasted: "Đã dán link từ clipboard",
  added: "Đã thêm vào hàng chờ tải",
  multi: "Đã nhận nhiều link.",
};
