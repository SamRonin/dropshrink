use serde::Serialize;
use std::path::{Path, PathBuf};

use tauri::Manager;

/// Hardware encoders that were detected **and** verified to actually work
/// (a driver can expose the encoder while the GPU is absent or blocked).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HwAccelStatus {
    pub available: bool,
    /// Verified encoder names, e.g. `["h264_nvenc", "hevc_nvenc"]`.
    pub encoders: Vec<String>,
    /// "nvidia" | "intel" | "vaapi" | "apple"
    pub vendor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FfmpegStatus {
    pub ffmpeg_path: Option<String>,
    pub ffprobe_path: Option<String>,
    pub version: Option<String>,
    /// "sidecar" | "bundled" | "system" | "custom" | "none"
    pub source: String,
    pub has_x265: bool,
    pub has_vp9: bool,
    pub hw_accel: HwAccelStatus,
}

/// Candidate hardware encoders per platform. Detection verifies each one with
/// a tiny real encode, so only working entries end up in `HwAccelStatus`.
const HW_CANDIDATES: &[&str] = &[
    // NVIDIA
    "h264_nvenc",
    "hevc_nvenc",
    "av1_nvenc",
    // Intel QuickSync
    "h264_qsv",
    "hevc_qsv",
    // Linux VAAPI
    "h264_vaapi",
    "hevc_vaapi",
    // macOS VideoToolbox
    "h264_videotoolbox",
    "hevc_videotoolbox",
];

fn vendor_of(encoder: &str) -> Option<&'static str> {
    if encoder.ends_with("nvenc") {
        Some("nvidia")
    } else if encoder.ends_with("qsv") {
        Some("intel")
    } else if encoder.ends_with("vaapi") {
        Some("vaapi")
    } else if encoder.ends_with("videotoolbox") {
        Some("apple")
    } else {
        None
    }
}

/// Hardware encoders make no sense on foreign platforms even if a build
/// somehow lists them.
fn encoder_supported_on_platform(encoder: &str) -> bool {
    if encoder.ends_with("videotoolbox") {
        cfg!(target_os = "macos")
    } else if encoder.ends_with("vaapi") {
        cfg!(target_os = "linux")
    } else {
        !cfg!(target_os = "macos")
    }
}

fn exe_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// Tauri `externalBin` sidecars are bundled next to the main executable using
/// the Rust target triple, e.g. `ffmpeg-x86_64-pc-windows-msvc.exe`.
fn sidecar_name(name: &str) -> Option<String> {
    let triple = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        ("windows", "aarch64") => "aarch64-pc-windows-msvc",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => return None,
    };
    let ext = if cfg!(windows) { ".exe" } else { "" };
    Some(format!("{name}-{triple}{ext}"))
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

/// Binaries from a user-chosen directory (settings → `ffmpegDir`).
fn custom_candidates(custom_dir: &str, name: &str) -> Vec<PathBuf> {
    let target = exe_name(name);
    let dir = PathBuf::from(custom_dir);
    vec![dir.join(&target), dir.join("bin").join(&target)]
}

async fn binary_version(ffprobe_path: &Path) -> Option<String> {
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

/// Parse the encoder names out of `ffmpeg -hide_banner -encoders` output.
async fn list_encoders(ffmpeg_path: &Path) -> Vec<String> {
    let output = tokio::process::Command::new(ffmpeg_path)
        .args(["-hide_banner", "-encoders"])
        .output()
        .await;
    let Ok(output) = output else {
        return Vec::new();
    };
    let mut names = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        // Format: " V..... h264_nvenc    NVIDIA NVENC H.264 encoder ..."
        let mut parts = line.split_whitespace();
        let _flags = parts.next();
        if let Some(name) = parts.next() {
            names.push(name.to_string());
        }
    }
    names
}

/// Verify an encoder with a one-frame test encode — the encoder can appear in
/// `-encoders` yet fail at runtime (missing GPU, disabled driver, no DRI
/// device, ...). Guarded with a timeout so a wedged driver can't block startup.
async fn encoder_works(ffmpeg_path: &Path, encoder: &str) -> bool {
    let mut cmd = tokio::process::Command::new(ffmpeg_path);
    cmd.args(["-hide_banner", "-v", "error", "-nostdin"]);
    if encoder.ends_with("vaapi") {
        cmd.args([
            "-init_hw_device",
            "vaapi=va:/dev/dri/renderD128",
            "-filter_hw_device",
            "va",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=64x64:d=0.04",
            "-vf",
            "format=nv12,hwupload=extra_hw_frames=8",
        ]);
    } else {
        cmd.args(["-f", "lavfi", "-i", "color=c=black:s=64x64:d=0.04"]);
    }
    cmd.args(["-c:v", encoder, "-frames:v", "1", "-f", "null", "-"]);

    let attempt = tokio::time::timeout(std::time::Duration::from_secs(20), cmd.output()).await;
    match attempt {
        Ok(Ok(output)) => output.status.success(),
        _ => false,
    }
}

async fn detect_hw_accel(ffmpeg_path: &Path) -> HwAccelStatus {
    let listed = list_encoders(ffmpeg_path).await;
    let mut encoders: Vec<String> = Vec::new();
    let mut vendor = None;

    for candidate in HW_CANDIDATES {
        if !encoder_supported_on_platform(candidate) || !listed.iter().any(|e| e == candidate) {
            continue;
        }
        if encoder_works(ffmpeg_path, candidate).await {
            if vendor.is_none() {
                vendor = vendor_of(candidate).map(String::from);
            }
            encoders.push((*candidate).to_string());
        }
    }

    if encoders.is_empty() {
        HwAccelStatus::default()
    } else {
        HwAccelStatus {
            available: true,
            encoders,
            vendor,
        }
    }
}

/// Locate FFmpeg/FFprobe:
/// sidecar → bundled/portable → system (PATH + common locations) → custom dir.
pub async fn detect(app: &tauri::AppHandle, custom_dir: Option<&str>) -> FfmpegStatus {
    let sidecar: Vec<PathBuf> = std::env::current_exe()
        .ok()
        .and_then(|exe| {
            let dir = exe.parent()?.to_path_buf();
            Some(vec![dir.join(sidecar_name("ffmpeg")?)])
        })
        .unwrap_or_default();

    let mut system: Vec<PathBuf> = Vec::new();
    if let Some(p) = find_in_path("ffmpeg") {
        system.push(p);
    }
    system.extend(common_locations("ffmpeg"));

    let custom: Vec<PathBuf> = custom_dir
        .map(|dir| custom_candidates(dir, "ffmpeg"))
        .unwrap_or_default();

    let mut ffmpeg: Option<PathBuf> = None;
    let mut source = "none";
    for (kind, candidates) in [
        ("sidecar", &sidecar),
        ("bundled", &bundled_candidates(app, "ffmpeg")),
        ("system", &system),
        ("custom", &custom),
    ] {
        for candidate in candidates {
            if candidate.is_file() {
                ffmpeg = Some(candidate.clone());
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
        // Prefer an ffprobe sitting next to the resolved ffmpeg (covers both
        // the sidecar triple naming and plain folder layouts).
        let mut same_dir: Vec<PathBuf> = Vec::new();
        if let Some(parent) = ff.parent() {
            if let Some(triple_name) = sidecar_name("ffprobe") {
                same_dir.push(parent.join(triple_name));
            }
            same_dir.push(parent.join(exe_name("ffprobe")));
        }
        if let Some(found) = same_dir.into_iter().find(|p| p.is_file()) {
            ffprobe = Some(found);
        }
    }
    if ffprobe.is_none() && ffmpeg.is_some() {
        let mut candidates = find_in_path("ffprobe").into_iter().collect::<Vec<_>>();
        candidates.extend(common_locations("ffprobe"));
        if let Some(dir) = custom_dir {
            candidates.extend(custom_candidates(dir, "ffprobe"));
        }
        ffprobe = candidates.into_iter().find(|p| p.is_file());
    }

    let version = if let Some(probe_bin) = &ffprobe {
        binary_version(probe_bin).await
    } else {
        None
    };

    let (listed, hw_accel) = if let Some(ff) = &ffmpeg {
        let listed = list_encoders(ff).await;
        let has_x265 = listed.iter().any(|e| e == "libx265");
        let has_vp9 = listed.iter().any(|e| e == "libvpx-vp9");
        let hw = detect_hw_accel(ff).await;
        (Some((has_x265, has_vp9)), hw)
    } else {
        (None, HwAccelStatus::default())
    };
    let (has_x265, has_vp9) = listed.unwrap_or((false, false));

    FfmpegStatus {
        ffmpeg_path: ffmpeg.map(|p| p.to_string_lossy().to_string()),
        ffprobe_path: ffprobe.map(|p| p.to_string_lossy().to_string()),
        version,
        source: source.to_string(),
        has_x265,
        has_vp9,
        hw_accel,
    }
}
