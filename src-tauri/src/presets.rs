use crate::jobs::{JobKind, Preset};
use crate::probe::MediaInfo;
use crate::settings::{Settings, WebCodec};

/// File extension of the encoder output for a given preset/media combination.
pub fn output_ext(preset: Preset, kind: JobKind, web_codec: WebCodec) -> &'static str {
    match (preset, kind) {
        (Preset::Telegram, JobKind::Video) => "mp4",
        (Preset::Telegram, JobKind::Image) => "jpg",
        (Preset::Web, JobKind::Video) => match web_codec {
            WebCodec::Vp9 => "webm",
            WebCodec::H264 => "mp4",
        },
        (Preset::Web, JobKind::Image) => "webp",
        (Preset::Extreme, JobKind::Video) => "mp4",
        (Preset::Extreme, JobKind::Image) => "webp",
    }
}

fn base_args(input: &str) -> Vec<String> {
    vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-nostdin".into(),
        "-y".into(),
        "-i".into(),
        input.into(),
    ]
}

fn strip_metadata_args(args: &mut Vec<String>) {
    args.push("-map_metadata".into());
    args.push("-1".into());
}

fn scale_filter(cap_w: u32, cap_h: u32) -> String {
    format!("scale=w={cap_w}:h={cap_h}:force_original_aspect_ratio=decrease:force_divisible_by=2")
}

fn video_filter(cap_w: u32, cap_h: u32, info: &MediaInfo) -> String {
    let mut filters: Vec<String> = Vec::new();
    // Only cap the frame rate when the source is actually higher.
    if let Some(fps) = info.fps {
        if fps > 31.0 {
            filters.push("fps=30".into());
        }
    }
    filters.push(scale_filter(cap_w, cap_h));
    filters.join(",")
}

fn audio_aac(args: &mut Vec<String>, kbps: u32, mono: bool) {
    args.extend(["-c:a".into(), "aac".into(), "-b:a".into(), format!("{kbps}k")]);
    if mono {
        args.extend(["-ac".into(), "1".into()]);
    }
}

fn audio_opus(args: &mut Vec<String>, kbps: u32) {
    args.extend(["-c:a".into(), "libopus".into(), "-b:a".into(), format!("{kbps}k")]);
}

/// Build the full ffmpeg argument list for a video job.
#[allow(clippy::too_many_arguments)]
pub fn build_video_args(
    preset: Preset,
    settings: &Settings,
    has_x265: bool,
    has_vp9: bool,
    input: &str,
    output: &str,
    info: &MediaInfo,
) -> Vec<String> {
    let mut args = base_args(input);
    if settings.strip_metadata {
        strip_metadata_args(&mut args);
    }

    match preset {
        // Telegram: H.264 + AAC + faststart, 1280px cap, ≤30fps → instant send & stream.
        Preset::Telegram => {
            args.extend(["-vf".into(), video_filter(1280, 1280, info)]);
            args.extend([
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "medium".into(),
                "-crf".into(),
                "24".into(),
                "-maxrate".into(),
                "2400k".into(),
                "-bufsize".into(),
                "4800k".into(),
                "-pix_fmt".into(),
                "yuv420p".into(),
            ]);
            audio_aac(&mut args, 128, false);
            args.extend(["-movflags".into(), "+faststart".into()]);
        }
        // Web: ≤1080p; H.264 (fast) or VP9/Opus (smaller), user choice.
        Preset::Web => {
            args.extend(["-vf".into(), video_filter(1920, 1080, info)]);
            match settings.web_codec {
                WebCodec::Vp9 if has_vp9 => {
                    args.extend([
                        "-c:v".into(),
                        "libvpx-vp9".into(),
                        "-crf".into(),
                        "32".into(),
                        "-b:v".into(),
                        "0".into(),
                        "-deadline".into(),
                        "good".into(),
                        "-cpu-used".into(),
                        "5".into(),
                        "-row-mt".into(),
                        "1".into(),
                    ]);
                    audio_opus(&mut args, 96);
                }
                _ => {
                    args.extend([
                        "-c:v".into(),
                        "libx264".into(),
                        "-preset".into(),
                        "medium".into(),
                        "-crf".into(),
                        "25".into(),
                        "-pix_fmt".into(),
                        "yuv420p".into(),
                    ]);
                    audio_aac(&mut args, 96, false);
                    args.extend(["-movflags".into(), "+faststart".into()]);
                }
            }
        }
        // Extreme: HEVC CRF 32 (fallback H.264 CRF 28), original resolution, mono audio.
        Preset::Extreme => {
            if has_x265 {
                args.extend([
                    "-c:v".into(),
                    "libx265".into(),
                    "-preset".into(),
                    "medium".into(),
                    "-crf".into(),
                    "32".into(),
                    "-tag:v".into(),
                    "hvc1".into(),
                    "-pix_fmt".into(),
                    "yuv420p".into(),
                ]);
            } else {
                args.extend([
                    "-c:v".into(),
                    "libx264".into(),
                    "-preset".into(),
                    "medium".into(),
                    "-crf".into(),
                    "28".into(),
                    "-pix_fmt".into(),
                    "yuv420p".into(),
                ]);
            }
            audio_aac(&mut args, 64, true);
        }
    }

    args.extend(["-progress".into(), "pipe:1".into(), "-nostats".into()]);
    args.push(output.into());
    args
}

/// Build the full ffmpeg argument list for an image job.
pub fn build_image_args(preset: Preset, input: &str, output: &str, strip_metadata: bool) -> Vec<String> {
    let mut args = base_args(input);
    if strip_metadata {
        strip_metadata_args(&mut args);
    }

    match preset {
        Preset::Telegram => {
            args.extend(["-vf".into(), scale_filter(1280, 1280)]);
            args.extend(["-q:v".into(), "3".into()]);
        }
        Preset::Web => {
            args.extend(["-vf".into(), scale_filter(1920, 1920)]);
            args.extend([
                "-c:v".into(),
                "libwebp".into(),
                "-q:v".into(),
                "78".into(),
                "-compression_level".into(),
                "5".into(),
            ]);
        }
        Preset::Extreme => {
            args.extend(["-vf".into(), scale_filter(1600, 1600)]);
            args.extend([
                "-c:v".into(),
                "libwebp".into(),
                "-q:v".into(),
                "55".into(),
                "-compression_level".into(),
                "5".into(),
            ]);
        }
    }

    args.push(output.into());
    args
}
