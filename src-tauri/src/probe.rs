use serde_json::Value;
use std::path::Path;

/// Metadata extracted with ffprobe; drives scaling caps and progress math.
#[derive(Debug, Clone)]
pub struct MediaInfo {
    pub duration_secs: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: Option<f32>,
    pub has_audio: bool,
    pub video_codec: Option<String>,
}

fn parse_rational(raw: &str) -> Option<f32> {
    let mut parts = raw.split('/');
    let num: f32 = parts.next()?.trim().parse().ok()?;
    let den: f32 = match parts.next() {
        Some(d) => d.trim().parse().ok()?,
        None => 1.0,
    };
    if den == 0.0 || !num.is_finite() || num <= 0.0 {
        return None;
    }
    Some(num / den)
}

pub async fn probe(ffprobe_path: &str, input: &Path) -> Result<MediaInfo, String> {
    let output = tokio::process::Command::new(ffprobe_path)
        .args(["-v", "error", "-print_format", "json", "-show_format", "-show_streams"])
        .arg(input)
        .output()
        .await
        .map_err(|e| format!("probe_spawn_failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let last = stderr.lines().last().unwrap_or("unknown error").trim();
        return Err(format!("probe_failed: {last}"));
    }

    let json: Value =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("probe_parse_failed: {e}"))?;

    let mut info = MediaInfo {
        duration_secs: None,
        width: None,
        height: None,
        fps: None,
        has_audio: false,
        video_codec: None,
    };

    if let Some(duration) = json
        .pointer("/format/duration")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
    {
        info.duration_secs = Some(duration);
    }

    if let Some(streams) = json.get("streams").and_then(|v| v.as_array()) {
        for stream in streams {
            match stream.get("codec_type").and_then(|v| v.as_str()) {
                Some("video") => {
                    if info.video_codec.is_none() {
                        info.video_codec = stream.get("codec_name").and_then(|v| v.as_str()).map(String::from);
                        info.width = stream.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
                        info.height = stream.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);
                        info.fps = stream
                            .get("avg_frame_rate")
                            .and_then(|v| v.as_str())
                            .and_then(parse_rational)
                            .or_else(|| {
                                stream
                                    .get("r_frame_rate")
                                    .and_then(|v| v.as_str())
                                    .and_then(parse_rational)
                            });
                    }
                }
                Some("audio") => info.has_audio = true,
                _ => {}
            }
        }
    }

    Ok(info)
}
