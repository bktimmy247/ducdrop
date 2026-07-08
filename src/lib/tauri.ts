// Thin wrappers around Tauri APIs, with browser-dev fallbacks so `pnpm dev`
// in a normal browser doesn't crash before the desktop shell is attached.

import { invoke } from "@tauri-apps/api/core";
import type { DownloadMode, DownloadProgress, DownloadStatus, EngineHealth, PreviewInfo } from "./types";

export type { DownloadMode, DownloadProgress, DownloadStatus, EngineHealth, PreviewInfo };

function hasTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function startDownload(
  url: string,
  mode: DownloadMode,
): Promise<string> {
  if (!hasTauri()) {
    console.warn("[dev] startDownload noop", url, mode);
    return crypto.randomUUID();
  }
  return invoke<string>("start_download", { url, mode });
}

export async function cancelDownload(id: string): Promise<void> {
  if (!hasTauri()) {
    console.warn("[dev] cancelDownload noop", id);
    return;
  }
  await invoke("cancel_download", { id });
}

export async function cancelAllDownloads(): Promise<void> {
  if (!hasTauri()) {
    console.warn("[dev] cancelAllDownloads noop");
    return;
  }
  await invoke("cancel_all_downloads");
}

export async function catchPreview(url: string): Promise<PreviewInfo | null> {
  if (!hasTauri()) {
    console.warn("[dev] catchPreview noop", url);
    return null;
  }
  try {
    return await invoke<PreviewInfo | null>("catch_preview", { url });
  } catch (e) {
    console.warn("preview failed", e);
    return null;
  }
}

export async function openFolder(path: string): Promise<void> {
  if (!hasTauri()) return;
  await invoke("open_folder", { path });
}

export async function getDownloadsDir(): Promise<string> {
  if (!hasTauri()) return "Downloads/DucDrop";
  return invoke<string>("get_downloads_dir");
}

export async function readClipboard(): Promise<string> {
  if (!hasTauri()) {
    try {
      return await navigator.clipboard.readText();
    } catch {
      return "";
    }
  }
  const { readText } = await import("@tauri-apps/plugin-clipboard-manager");
  return (await readText()) ?? "";
}

export async function onProgress(
  cb: (p: DownloadProgress) => void,
): Promise<() => void> {
  if (!hasTauri()) return () => {};
  const { listen } = await import("@tauri-apps/api/event");
  const unlisten = await listen<DownloadProgress>(
    "download://progress",
    (e) => cb(e.payload),
  );
  return unlisten;
}

export async function getEngineHealth(): Promise<EngineHealth[]> {
  if (!hasTauri()) {
    return [
      { name: "yt-dlp", ready: false, message: "Không chạy trong môi trường Tauri." },
      { name: "ffmpeg", ready: false, message: "Không chạy trong môi trường Tauri." },
    ];
  }
  try {
    return await invoke<EngineHealth[]>("get_engine_health");
  } catch (e) {
    return [
      { name: "yt-dlp", ready: false, message: String(e) },
      { name: "ffmpeg", ready: false, message: String(e) },
    ];
  }
}
