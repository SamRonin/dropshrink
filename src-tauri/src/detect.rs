use serde::Serialize;
use std::path::PathBuf;

use tauri::Manager;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatus {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub version: Option<String>,
    /// "bundled" | "system" | "none"
    pub source: String,
    pub has_x265: bool,
    pub has_vp9: bool,
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let target = exe_name(name);
    let paths = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&paths) {
        let candidate = dir.join(&target);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn common_locations(name: &str) -> Vec<PathBuf> {
    let target = exe_name(name);
    let mut dirs: Vec<PathBuf> = Vec::new();
    if cfg!(windows) {
        dirs.push(PathBuf::from(r"C:\ffmpeg\bin"));
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            dirs.push(PathBuf::from(local).join("Programs").join("ffmpeg").join("bin"));
        }
        if let Some(pf) = std::env::var_os("ProgramFiles") {
            dirs.push(PathBuf::from(pf).join("ffmpeg").join("bin"));
        }
    } else if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    } else {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/usr/bin"));
        dirs.push(PathBuf::from("/snap/bin"));
    }
    dirs.into_iter().map(|d| d.join(&target)).collect()
}

/// Places a bundled/portable binary could live.
fn bundled_candidates(app: &tauri::AppHandle, name: &str) -> Vec<PathBuf> {
    let target = exe_name(name);
    let mut out = Vec::new();
    // Portable mode: `binaries/` folder next to the installed executable.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            out.push(dir.join("binaries").join(&target));
        }
    }
    // Bundled resource directory.
    if let Ok(res) = app.path().resource_dir() {
        out.push(res.join("binaries").join(&target));
        out.push(res.join(&target));
    }
    out
}

async fn binary_version(ffprobe_path: &PathBuf) -> Option<String> {
    let output = tokio::process::Command::new(ffprobe_path)
        .arg("-version")
        .output()
        .await
        .ok()?;
    let text = String::from_utf8_lossy(&output.stdout);
    let first = text.lines().next()?;
    let idx = first.find("version")?;
    let rest = first[idx + "version".len()..].trim();
    let version: String = rest.chars().take_while(|c| !c.is_whitespace()).collect();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

async fn has_encoder(ffmpeg_path: &PathBuf, encoder: &str) -> bool {
    let Ok(output) = tokio::process::Command::new(ffmpeg_path)
        .args(["-hide_banner", "-encoders"])
        .output()
        .await
    else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout).contains(encoder)
}

/// Locate FFmpeg/FFprobe: bundled → PATH → common install locations.
pub async fn detect(app: &tauri::AppHandle) -> FfmpegStatus {
    let mut ffmpeg: Option<PathBuf> = None;
    let mut source = "none";

    let bundled = bundled_candidates(app, "ffmpeg");
    let mut system: Vec<PathBuf> = Vec::new();
    if let Some(p) = find_in_path("ffmpeg") {
        system.push(p);
    }
    system.extend(common_locations("ffmpeg"));

    for (kind, candidates) in [("bundled", bundled), ("system", system)] {
        for candidate in candidates {
            if candidate.is_file() {
                ffmpeg = Some(candidate);
                source = kind;
                break;
            }
        }
        if ffmpeg.is_some() {
            break;
        }
    }

    let mut ffprobe: Option<PathBuf> = None;
    if let Some(ff) = &ffmpeg {
        let target = exe_name("ffprobe");
        if let Some(parent) = ff.parent() {
            let same_dir = parent.join(&target);
            if same_dir.is_file() {
                ffprobe = Some(same_dir);
            }
        }
        if ffprobe.is_none() {
            ffprobe = find_in_path("ffprobe")
                .or_else(|| common_locations("ffprobe").into_iter().find(|p| p.is_file()));
        }
    }

    let version = if let Some(probe_bin) = &ffprobe {
        binary_version(probe_bin).await
    } else {
        None
    };

    let (has_x265, has_vp9) = if let Some(ff) = &ffmpeg {
        tokio::join!(has_encoder(ff, "libx265"), has_encoder(ff, "libvpx-vp9"))
    } else {
        (false, false)
    };

    FfmpegStatus {
        ffmpeg_path: ffmpeg.map(|p| p.to_string_lossy().to_string()),
        ffprobe_path: ffprobe.map(|p| p.to_string_lossy().to_string()),
        version,
        source: source.to_string(),
        has_x265,
        has_vp9,
    }
}
