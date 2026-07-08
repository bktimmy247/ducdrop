// Shared DucDrop types

export type DownloadMode = "video" | "audio" | "thumbnail" | "subtitle" | "channel";

export type DownloadStatus =
  | "queued"
  | "downloading"
  | "completed"
  | "error"
  | "cancelled";

export interface DownloadProgress {
  id: string;
  url: string;
  title: string;
  percent: number;
  speed: string;
  status: DownloadStatus;
  filepath?: string;
  thumbnail?: string;
  message?: string;
}

export interface PreviewInfo {
  title: string;
  source: string;
  thumbnail?: string;
  duration?: string;
  modes: DownloadMode[];
  note?: string;
}

export interface EngineHealth {
  name: "yt-dlp" | "ffmpeg";
  ready: boolean;
  version?: string;
  message: string;
}
