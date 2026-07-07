use std::path::PathBuf;
use std::process::Stdio;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

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
            format!("/select,{}", p.to_string_lossy())
        } else {
            p.to_string_lossy().to_string()
        };
        std::process::Command::new("explorer")
            .arg(arg)
            .spawn()
            .map_err(|e| e.to_string())?;
    }
    #[cfg(not(target_os = "windows"))]
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
    Ok(())
}

fn mode_args(mode: &str) -> Vec<String> {
    match mode {
        "audio" => vec![
            "-x".into(),
            "--audio-format".into(),
            "mp3".into(),
        ],
        "small" => vec![
            "-f".into(),
            "bv*[height<=720]+ba/b[height<=720]".into(),
            "--merge-output-format".into(),
            "mp4".into(),
        ],
        // smart + best behave the same
        _ => vec![
            "-f".into(),
            "bv*+ba/b".into(),
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

#[tauri::command]
async fn start_download(app: AppHandle, url: String, mode: String) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let out_dir = downloads_dir();
    std::fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;

    let yt = resolve_sidecar(&app, "yt-dlp")?;
    let ffmpeg_dir = resolve_sidecar(&app, "ffmpeg")
        .ok()
        .and_then(|p| p.parent().map(|x| x.to_path_buf()));

    let out_tpl = out_dir.join("%(title).120s.%(ext)s");

    let mut args: Vec<String> = vec![
        "--newline".into(),
        "--no-playlist".into(),
        "--no-mtime".into(),
        "-o".into(),
        out_tpl.to_string_lossy().to_string(),
    ];
    if let Some(dir) = &ffmpeg_dir {
        args.push("--ffmpeg-location".into());
        args.push(dir.to_string_lossy().to_string());
    }
    args.extend(mode_args(&mode));
    args.push(url.clone());

    let emit_status = |status: &str, percent: f64, speed: &str, title: &str, msg: Option<String>| {
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

    let mut child = Command::new(&yt)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Không chạy được yt-dlp: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let mut reader = BufReader::new(stdout).lines();

    let mut title = String::new();
    let mut last_file: Option<String> = None;

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
        }

        if line.contains("[download]") && line.contains('%') {
            let pct = parse_percent(&line).unwrap_or(0.0);
            let spd = parse_speed(&line).unwrap_or_default();
            emit_status("downloading", pct, &spd, &title, None);
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;

    if status.success() {
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            start_download,
            open_folder,
            get_downloads_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running DucDrop");
}
