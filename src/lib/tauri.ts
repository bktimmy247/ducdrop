// Thin wrappers around Tauri APIs, with browser-dev fallbacks so `pnpm dev`
// in a normal browser doesn't crash before the desktop shell is attached.

export type DownloadMode = "smart" | "best" | "audio" | "small";

export type DownloadStatus =
  | "queued"
  | "downloading"
  | "completed"
  | "error";

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
  const { invoke } = await import("@tauri-apps/api/core");
  return invoke<string>("start_download", { url, mode });
}

export async function openFolder(path: string): Promise<void> {
  if (!hasTauri()) return;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke("open_folder", { path });
}

export async function getDownloadsDir(): Promise<string> {
  if (!hasTauri()) return "Downloads/DucDrop";
  const { invoke } = await import("@tauri-apps/api/core");
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
