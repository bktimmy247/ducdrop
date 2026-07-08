use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
fn hide_child_window(cmd: &mut Command) {
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_child_window(_cmd: &mut Command) {}


static ACTIVE_DOWNLOADS: OnceLock<Mutex<HashMap<String, u32>>> = OnceLock::new();
static CANCELLED_DOWNLOADS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn active_downloads() -> &'static Mutex<HashMap<String, u32>> {
    ACTIVE_DOWNLOADS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cancelled_downloads() -> &'static Mutex<HashSet<String>> {
    CANCELLED_DOWNLOADS.get_or_init(|| Mutex::new(HashSet::new()))
}

#[cfg(target_os = "windows")]
fn kill_process_tree(pid: u32) -> Result<(), String> {
    std::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn kill_process_tree(pid: u32) -> Result<(), String> {
    std::process::Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn cancel_download(id: String) -> Result<(), String> {
    cancelled_downloads()
        .lock()
        .map_err(|e| e.to_string())?
        .insert(id.clone());

    let pid = active_downloads()
        .lock()
        .map_err(|e| e.to_string())?
        .get(&id)
        .copied();

    if let Some(pid) = pid {
        kill_process_tree(pid)?;
    }
    Ok(())
}

#[tauri::command]
fn cancel_all_downloads() -> Result<(), String> {
    let ids_and_pids: Vec<(String, u32)> = active_downloads()
        .lock()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|(id, pid)| (id.clone(), *pid))
        .collect();

    {
        let mut cancelled = cancelled_downloads().lock().map_err(|e| e.to_string())?;
        for (id, _) in &ids_and_pids {
            cancelled.insert(id.clone());
        }
    }

    for (_, pid) in ids_and_pids {
        let _ = kill_process_tree(pid);
    }
    Ok(())
}


#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Serialize)]
struct UpdateResult {
    current_version: String,
    latest_version: String,
    update_available: bool,
    message: String,
}

fn normalize_version(v: &str) -> String {
    v.trim().trim_start_matches('v').to_string()
}

fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        normalize_version(v)
            .split('.')
            .map(|x| x.parse::<u32>().unwrap_or(0))
            .collect()
    };
    let a = parse(latest);
    let b = parse(current);
    for i in 0..a.len().max(b.len()) {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        if av > bv { return true; }
        if av < bv { return false; }
    }
    false
}

#[tauri::command]
async fn update_to_latest() -> Result<UpdateResult, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let client = reqwest::Client::builder()
        .user_agent("DucDrop updater")
        .build()
        .map_err(|e| e.to_string())?;

    let release: GitHubRelease = client
        .get("https://api.github.com/repos/bktimmy247/ducdrop/releases/latest")
        .send()
        .await
        .map_err(|e| format!("Không kiểm tra được bản cập nhật: {e}"))?
        .error_for_status()
        .map_err(|e| format!("GitHub trả lỗi khi kiểm tra cập nhật: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Không đọc được thông tin bản cập nhật: {e}"))?;

    let latest = normalize_version(&release.tag_name);
    if !is_newer_version(&latest, &current) {
        return Ok(UpdateResult {
            current_version: current.clone(),
            latest_version: latest,
            update_available: false,
            message: format!("DucDrop đang ở bản mới nhất ({current})."),
        });
    }

    let asset = release
        .assets
        .iter()
        .find(|a| a.name.to_lowercase().ends_with(".exe") && a.name.to_lowercase().contains("setup"))
        .or_else(|| release.assets.iter().find(|a| a.name.to_lowercase().ends_with(".exe")))
        .ok_or_else(|| format!("Có bản {latest} nhưng chưa thấy file cài Windows. Mở release: {}", release.html_url))?;

    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| format!("Không tải được bản cập nhật: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Link tải cập nhật bị lỗi: {e}"))?
        .bytes()
        .await
        .map_err(|e| format!("Không đọc được file cập nhật: {e}"))?;

    let dir = std::env::temp_dir().join("DucDropUpdates");
    tokio::fs::create_dir_all(&dir).await.map_err(|e| e.to_string())?;
    let installer_path = dir.join(&asset.name);
    tokio::fs::write(&installer_path, bytes).await.map_err(|e| e.to_string())?;

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(&installer_path)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Đã tải xong nhưng không chạy được installer: {e}"))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(&installer_path)
            .spawn()
            .map_err(|e| format!("Đã tải xong nhưng không chạy được installer: {e}"))?;
    }

    Ok(UpdateResult {
        current_version: current,
        latest_version: latest.clone(),
        update_available: true,
        message: format!("Đã tải DucDrop {latest}. Trình cài đặt đang mở, anh bấm theo hướng dẫn để cập nhật."),
    })
}

#[derive(Clone, Serialize)]
struct Progress {
    id: String,
    url: String,
    title: String,
    percent: f64,
    speed: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    filepath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Clone, Serialize)]
struct PreviewInfo {
    title: String,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<String>,
    modes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    note: Option<String>,
}

#[derive(Clone, Serialize)]
struct EngineHealth {
    name: String,
    ready: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    message: String,
}

fn downloads_dir() -> PathBuf {
    // ...\Users\<name>\Downloads\DucDrop
    let base = dirs_download();
    base.join("DucDrop")
}

fn dirs_download() -> PathBuf {
    if let Some(user) = std::env::var_os("USERPROFILE") {
        PathBuf::from(user).join("Downloads")
    } else if let Some(home) = std::env::var_os("HOME") {
        PathBuf::from(home).join("Downloads")
    } else {
        PathBuf::from(".")
    }
}

#[tauri::command]
fn get_downloads_dir() -> String {
    downloads_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    #[cfg(target_os = "windows")]
    {
        let arg = if p.is_file() {
            format!("/select,\"{}\"", p.to_string_lossy())
        } else {
            p.to_string_lossy().to_string()
        };
        std::process::Command::new("explorer")
            .arg(arg)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(target_os = "macos")]
    {
        let target = if p.is_file() {
            p.parent().map(|x| x.to_path_buf()).unwrap_or(p.clone())
        } else {
            p
        };
        std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if p.is_file() {
            p.parent().map(|x| x.to_path_buf()).unwrap_or(p.clone())
        } else {
            p
        };
        std::process::Command::new("xdg-open")
            .arg(target)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn mode_args(mode: &str) -> Vec<String> {
    match mode {
        "audio" => vec!["-x".into(), "--audio-format".into(), "mp3".into()],
        "thumbnail" => vec![
            "--skip-download".into(),
            "--write-thumbnail".into(),
            "--convert-thumbnails".into(),
            "jpg".into(),
        ],
        "subtitle" => vec![
            "--skip-download".into(),
            "--write-subs".into(),
            "--write-auto-subs".into(),
            "--sub-lang".into(),
            "en".into(),
            "--convert-subs".into(),
            "srt".into(),
        ],
        "channel" | _ => vec![
            "-f".into(),
            // Prefer MP4/H.264 + M4A/AAC first so downloaded videos open in common players.
            "bv*[ext=mp4][vcodec^=avc1]+ba[ext=m4a]/b[ext=mp4]/bv*+ba/b".into(),
            "--merge-output-format".into(),
            "mp4".into(),
        ],
    }
}

fn parse_percent(line: &str) -> Option<f64> {
    // yt-dlp --newline line: "[download]  12.3% of ~10.00MiB at  1.20MiB/s ETA 00:07"
    let idx = line.find('%')?;
    let start = line[..idx]
        .rfind(|c: char| c == ' ' || c == '[')
        .map(|i| i + 1)
        .unwrap_or(0);
    line[start..idx].trim().parse::<f64>().ok()
}

fn parse_speed(line: &str) -> Option<String> {
    let at = line.find(" at ")?;
    let rest = &line[at + 4..];
    let end = rest.find(" ETA").unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn current_triple() -> String {
    // Matches Tauri externalBin naming.
    let arch = std::env::consts::ARCH; // x86_64
    if cfg!(target_os = "windows") {
        format!("{arch}-pc-windows-msvc")
    } else if cfg!(target_os = "macos") {
        format!("{arch}-apple-darwin")
    } else {
        format!("{arch}-unknown-linux-gnu")
    }
}

fn resolve_sidecar(app: &AppHandle, name: &str) -> Result<PathBuf, String> {
    let exe = if cfg!(target_os = "windows") {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    // 1) Next to the app executable (bundled sidecar location in release).
    if let Ok(dir) = std::env::current_exe() {
        if let Some(parent) = dir.parent() {
            let cand = parent.join(&exe);
            if cand.exists() {
                return Ok(cand);
            }
        }
    }

    // 2) Tauri resource dir.
    if let Ok(res) = app.path().resource_dir() {
        let cand = res.join(&exe);
        if cand.exists() {
            return Ok(cand);
        }
    }

    // 3) Dev: src-tauri/binaries with plain name or target-triple suffix.
    let bin_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    let dev_plain = bin_dir.join(&exe);
    if dev_plain.exists() {
        return Ok(dev_plain);
    }
    let triple = current_triple();
    let suffix = if cfg!(target_os = "windows") {
        format!("{name}-{triple}.exe")
    } else {
        format!("{name}-{triple}")
    };
    let dev_triple = bin_dir.join(&suffix);
    if dev_triple.exists() {
        return Ok(dev_triple);
    }

    // 4) PATH fallback.
    Ok(PathBuf::from(&exe))
}

#[tauri::command]
async fn get_engine_health(app: AppHandle) -> Result<Vec<EngineHealth>, String> {
    let mut out = Vec::with_capacity(2);

    out.push(check_engine(&app, "yt-dlp", &["--version"]).await);
    out.push(check_engine(&app, "ffmpeg", &["-version"]).await);

    Ok(out)
}

async fn check_engine(app: &AppHandle, name: &str, args: &[&str]) -> EngineHealth {
    let path = resolve_sidecar(app, name);
    let exe = match path {
        Ok(p) => p,
        Err(_) => {
            return EngineHealth {
                name: name.into(),
                ready: false,
                version: None,
                message: format!("Chưa tìm thấy {name}."),
            }
        }
    };

    let mut cmd = Command::new(&exe);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    hide_child_window(&mut cmd);

    let Ok(mut child) = cmd.spawn() else {
        return EngineHealth {
            name: name.into(),
            ready: false,
            version: None,
            message: format!("Không chạy được {name}."),
        };
    };

    let stdout = child.stdout.take();
    let mut version: Option<String> = None;
    if let Some(stdout) = stdout {
        let mut reader = BufReader::new(stdout).lines();
        if let Ok(Ok(Some(line))) = timeout(Duration::from_secs(4), reader.next_line()).await {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                version = Some(trimmed.split_whitespace().next().unwrap_or(trimmed).to_string());
            }
        }
    }

    let _ = child.wait().await;

    let ready = version.is_some();
    let message = if ready {
        format!("{name} sẵn sàng")
    } else {
        format!("Chưa tìm thấy {name}.")
    };

    EngineHealth {
        name: name.into(),
        ready,
        version,
        message,
    }
}

#[tauri::command]
async fn catch_preview(app: AppHandle, url: String) -> Result<Option<PreviewInfo>, String> {
    let yt = resolve_sidecar(&app, "yt-dlp")?;

    let mut preview_cmd = Command::new(&yt);
    preview_cmd
        .args([
            "--dump-json",
            "--no-download",
            "--no-playlist",
            "--quiet",
            "--",
            &url,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_child_window(&mut preview_cmd);

    let mut child = preview_cmd
        .spawn()
        .map_err(|e| format!("Không chạy được yt-dlp: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let mut reader = BufReader::new(stdout).lines();
    let mut json_line = String::new();
    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim_start().starts_with('{') {
            json_line = line;
            break;
        }
    }

    // Drain stderr in case of error.
    let stderr = child.stderr.take();
    let mut err_lines: Vec<String> = Vec::new();
    if let Some(stderr) = stderr {
        let mut er = BufReader::new(stderr).lines();
        while let Ok(Some(l)) = er.next_line().await {
            err_lines.push(l);
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() || json_line.is_empty() {
        // Unsupported/private URL: still allow direct download.
        return Ok(Some(PreviewInfo {
            title: url.clone(),
            source: guess_source(&url),
            thumbnail: None,
            duration: None,
            modes: vec!["video".into(), "audio".into(), "thumbnail".into(), "subtitle".into(), "channel".into()],
            note: Some("Preview chưa hỗ trợ cho link này, vẫn có thể tải trực tiếp.".into()),
        }));
    }

    let info: Value = serde_json::from_str(&json_line).unwrap_or(Value::Null);
    let title = info["title"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| url.clone());
    let source = info["extractor"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| guess_source(&url));
    let thumbnail = info["thumbnail"].as_str().map(|s| s.to_string());
    let duration = info["duration"].as_f64().map(fmt_duration);

    // Decide modes from entry type/flags.
    let mut modes = vec!["video".into(), "audio".into(), "thumbnail".into(), "channel".into()];
    if info["subtitles"].as_object().map(|m| !m.is_empty()).unwrap_or(false)
        || info["automatic_captions"].as_object().map(|m| !m.is_empty()).unwrap_or(false)
    {
        modes.push("subtitle".into());
    }

    Ok(Some(PreviewInfo {
        title,
        source,
        thumbnail,
        duration,
        modes,
        note: None,
    }))
}

fn guess_source(url: &str) -> String {
    let lower = url.to_lowercase();
    if lower.contains("youtube") || lower.contains("youtu.be") {
        "YouTube".into()
    } else if lower.contains("tiktok") {
        "TikTok".into()
    } else if lower.contains("facebook") || lower.contains("fb.watch") {
        "Facebook".into()
    } else if lower.contains("instagram") {
        "Instagram".into()
    } else {
        "Web".into()
    }
}

fn fmt_duration(secs: f64) -> String {
    let total = secs as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

#[tauri::command]
async fn start_download(app: AppHandle, url: String, mode: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let out_dir = downloads_dir();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let yt = resolve_sidecar(&app, "yt-dlp")?;
    let ffmpeg_path = resolve_sidecar(&app, "ffmpeg").ok();

    // Keep downloads tidy: Downloads/DucDrop/<platform>/<channel-or-author>/<number - title.ext>
    let out_tpl = out_dir.join("%(extractor_key|Web)s/%(uploader|Unknown)s/%(playlist_index|000)s - %(title).120s.%(ext)s");

    let mut args: Vec<String> = vec![
        "--newline".into(),
        "--no-mtime".into(),
        "--restrict-filenames".into(),
        "-o".into(),
        out_tpl.to_string_lossy().to_string(),
    ];
    if mode == "channel" {
        args.push("--yes-playlist".into());
    } else {
        args.push("--no-playlist".into());
    }
    if let Some(ffmpeg) = &ffmpeg_path {
        args.push("--ffmpeg-location".into());
        args.push(ffmpeg.to_string_lossy().to_string());
    }
    args.extend(mode_args(&mode));
    args.push(url.clone());

    let emit_status =
        |status: &str, percent: f64, speed: &str, title: &str, msg: Option<String>| {
            let _ = app.emit(
                "download://progress",
                Progress {
                    id: id.clone(),
                    url: url.clone(),
                    title: title.to_string(),
                    percent,
                    speed: speed.to_string(),
                    status: status.to_string(),
                    filepath: None,
                    thumbnail: None,
                    message: msg,
                },
            );
        };

    emit_status("queued", 0.0, "", &url, None);

    let mut download_cmd = Command::new(&yt);
    download_cmd
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_child_window(&mut download_cmd);

    let mut child = download_cmd
        .spawn()
        .map_err(|e| format!("Không chạy được yt-dlp: {e}"))?;

    if let Some(pid) = child.id() {
        let _ = active_downloads()
            .lock()
            .map(|mut active| active.insert(id.clone(), pid));
    }

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut title = String::new();
    let mut last_file: Option<String> = None;
    let mut no_subtitles = false;

    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(rest) = line.strip_prefix("[download] Destination: ") {
            last_file = Some(rest.trim().to_string());
            if title.is_empty() {
                if let Some(name) = PathBuf::from(rest.trim()).file_stem() {
                    title = name.to_string_lossy().to_string();
                }
            }
        } else if let Some(rest) = line.strip_prefix("[Merger] Merging formats into ") {
            let f = rest.trim().trim_matches('"').to_string();
            last_file = Some(f);
        } else if line.contains("There are no subtitles") || line.contains("has no subtitles") {
            no_subtitles = true;
        }

        if line.contains("[download]") && line.contains('%') {
            let pct = parse_percent(&line).unwrap_or(0.0);
            let spd = parse_speed(&line).unwrap_or_default();
            emit_status("downloading", pct, &spd, &title, None);
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    let _ = active_downloads().lock().map(|mut active| active.remove(&id));
    let was_cancelled = cancelled_downloads()
        .lock()
        .map(|mut cancelled| cancelled.remove(&id))
        .unwrap_or(false);

    if was_cancelled {
        emit_status("cancelled", 0.0, "", &title, Some("Đã dừng lệnh tải.".into()));
    } else if status.success() {
        if no_subtitles && mode == "subtitle" && last_file.is_none() {
            emit_status("error", 0.0, "", &title, Some("Nguồn này không có phụ đề để tải.".into()));
            return Ok(id);
        }
        let _ = app.emit(
            "download://progress",
            Progress {
                id: id.clone(),
                url: url.clone(),
                title: if title.is_empty() { url.clone() } else { title.clone() },
                percent: 100.0,
                speed: String::new(),
                status: "completed".into(),
                filepath: last_file.or_else(|| Some(out_dir.to_string_lossy().to_string())),
                thumbnail: None,
                message: None,
            },
        );
    } else {
        // capture stderr tail
        let mut err_txt = String::from("yt-dlp lỗi");
        if let Some(stderr) = child.stderr.take() {
            let mut er = BufReader::new(stderr).lines();
            let mut lines: Vec<String> = Vec::new();
            while let Ok(Some(l)) = er.next_line().await {
                lines.push(l);
            }
            if let Some(last) = lines.last() {
                err_txt = last.clone();
            }
        }
        emit_status("error", 0.0, "", &title, Some(err_txt));
    }

    Ok(id)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            start_download,
            open_folder,
            get_downloads_dir,
            get_engine_health,
            catch_preview,
            cancel_download,
            cancel_all_downloads,
            update_to_latest,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DucDrop");
}
