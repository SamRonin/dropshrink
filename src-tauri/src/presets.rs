use crate::detect::{FfmpegStatus, HwAccelStatus};
use crate::jobs::{JobKind, Preset};
use crate::probe::MediaInfo;
use crate::settings::{Settings, WebCodec};

/// File extension of the encoder output for a given preset/media combination.
/// `has_vp9` guards the Web preset: without libvpx-vp9 the fallback is H.264,
/// which cannot be muxed into a WebM container.
pub fn output_ext(
    preset: Preset,
    kind: JobKind,
    web_codec: WebCodec,
    has_vp9: bool,
) -> &'static str {
    match (preset, kind) {
        (Preset::Telegram, JobKind::Video) => "mp4",
        (Preset::Telegram, JobKind::Image) => "jpg",
        (Preset::Web, JobKind::Video) => match web_codec {
            WebCodec::Vp9 if has_vp9 => "webm",
            _ => "mp4",
        },
        (Preset::Web, JobKind::Image) => "webp",
        (Preset::Extreme, JobKind::Video) => "mp4",
        (Preset::Extreme, JobKind::Image) => "webp",
    }
}

/// The full ffmpeg invocation for a job, plus whether a hardware encoder is
/// used (so the runner can fall back to CPU when the encoder fails).
pub struct EncodePlan {
    pub args: Vec<String>,
    pub hw_encoder: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwCodec {
    H264,
    Hevc,
}

fn hw_candidates(codec: HwCodec) -> &'static [&'static str] {
    match codec {
        HwCodec::H264 => &["h264_nvenc", "h264_qsv", "h264_vaapi", "h264_videotoolbox"],
        HwCodec::Hevc => &["hevc_nvenc", "hevc_qsv", "hevc_vaapi", "hevc_videotoolbox"],
    }
}

/// Pick the first *verified* hardware encoder for a codec class.
pub fn pick_hw_encoder(codec: HwCodec, hw: &HwAccelStatus) -> Option<&'static str> {
    if !hw.available {
        return None;
    }
    hw_candidates(codec)
        .iter()
        .find(|e| hw.encoders.iter().any(|have| have == *e))
        .copied()
}

fn base_args(input: &str, pre_input: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = [
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-y",
    ]
    .iter()
    .map(|s| (*s).to_string())
    .collect();
    for flag in pre_input {
        args.push((*flag).to_string());
    }
    args.push("-i".into());
    args.push(input.into());
    args
}

fn keep_metadata_args(args: &mut Vec<String>) {
    // Carry dates/tags and other container metadata of input 0 into the output.
    args.push("-map_metadata".into());
    args.push("0".into());
}

fn strip_metadata_args(args: &mut Vec<String>) {
    args.push("-map_metadata".into());
    args.push("-1".into());
}

fn scale_filter(cap_w: u32, cap_h: u32) -> String {
    format!("scale=w={cap_w}:h={cap_h}:force_original_aspect_ratio=decrease:force_divisible_by=2")
}

/// Video filter chain; appends the VAAPI upload chain when encoding on VAAPI.
fn video_filter(cap_w: u32, cap_h: u32, info: &MediaInfo, vaapi: bool) -> String {
    let mut filters: Vec<String> = Vec::new();
    // Only cap the frame rate when the source is actually higher.
    if let Some(fps) = info.fps {
        if fps > 31.0 {
            filters.push("fps=30".into());
        }
    }
    filters.push(scale_filter(cap_w, cap_h));
    if vaapi {
        // Frames are in system memory (software decode) → upload to the GPU.
        filters.push("format=nv12".into());
        filters.push("hwupload=extra_hw_frames=8".into());
    }
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

/// Encoder-specific quality flags. Values roughly mirror the CRF numbers of
/// the CPU paths so presets keep their character across encoders.
fn hw_encoder_flags(args: &mut Vec<String>, encoder: &'static str) {
    match encoder {
        "h264_nvenc" => args.extend([
            "-c:v".into(),
            "h264_nvenc".into(),
            "-preset".into(),
            "p5".into(),
            "-rc".into(),
            "vbr".into(),
            "-cq".into(),
            "26".into(),
            "-b:v".into(),
            "0".into(),
            "-spatial-aq".into(),
            "1".into(),
        ]),
        "hevc_nvenc" => args.extend([
            "-c:v".into(),
            "hevc_nvenc".into(),
            "-preset".into(),
            "p6".into(),
            "-rc".into(),
            "vbr".into(),
            "-cq".into(),
            "30".into(),
            "-b:v".into(),
            "0".into(),
            "-spatial-aq".into(),
            "1".into(),
            "-tag:v".into(),
            "hvc1".into(),
        ]),
        "h264_qsv" => args.extend([
            "-c:v".into(),
            "h264_qsv".into(),
            "-preset".into(),
            "medium".into(),
            "-global_quality".into(),
            "26".into(),
        ]),
        "hevc_qsv" => args.extend([
            "-c:v".into(),
            "hevc_qsv".into(),
            "-preset".into(),
            "medium".into(),
            "-global_quality".into(),
            "30".into(),
            "-tag:v".into(),
            "hvc1".into(),
        ]),
        "h264_vaapi" => args.extend(["-c:v".into(), "h264_vaapi".into(), "-qp".into(), "26".into()]),
        "hevc_vaapi" => args.extend([
            "-c:v".into(),
            "hevc_vaapi".into(),
            "-qp".into(),
            "30".into(),
            "-tag:v".into(),
            "hvc1".into(),
        ]),
        "h264_videotoolbox" => args.extend([
            "-c:v".into(),
            "h264_videotoolbox".into(),
            "-q:v".into(),
            "55".into(),
        ]),
        "hevc_videotoolbox" => args.extend([
            "-c:v".into(),
            "hevc_videotoolbox".into(),
            "-q:v".into(),
            "50".into(),
            "-tag:v".into(),
            "hvc1".into(),
        ]),
        _ => {}
    }
    // VAAPI consumes hardware frames; the others take system-memory frames.
    if !encoder.ends_with("vaapi") {
        args.extend(["-pix_fmt".into(), "yuv420p".into()]);
    }
}

/// Build the full ffmpeg plan for a video job.
pub fn plan_video(
    preset: Preset,
    settings: &Settings,
    status: &FfmpegStatus,
    input: &str,
    output: &str,
    info: &MediaInfo,
) -> EncodePlan {
    let has_x265 = status.has_x265;
    let has_vp9 = status.has_vp9;

    // Which hardware encoder (if any) applies to this preset?
    let hw_encoder = if settings.hw_accel {
        match (preset, settings.web_codec) {
            (Preset::Web, WebCodec::Vp9) => None, // no HW VP9 in the candidate set → CPU
            (Preset::Extreme, _) => pick_hw_encoder(HwCodec::Hevc, &status.hw_accel),
            _ => pick_hw_encoder(HwCodec::H264, &status.hw_accel),
        }
    } else {
        None
    };

    // Input options: enable hardware decode where safe. VAAPI gets its own
    // device pipeline instead of `-hwaccel auto`.
    let mut pre_input: Vec<&str> = Vec::new();
    if hw_encoder.is_some() && !hw_encoder.is_some_and(|e| e.ends_with("vaapi")) {
        pre_input.extend(["-hwaccel", "auto"]);
    }
    if hw_encoder == Some("h264_vaapi") || hw_encoder == Some("hevc_vaapi") {
        pre_input.extend(["-vaapi_device", "/dev/dri/renderD128"]);
    }

    let vaapi = hw_encoder.is_some_and(|e| e.ends_with("vaapi"));
    let mut args = base_args(input, &pre_input);
    if settings.strip_metadata {
        strip_metadata_args(&mut args);
    } else {
        keep_metadata_args(&mut args);
    }

    match preset {
        // Telegram: H.264 + AAC + faststart, 1280px cap, ≤30fps → instant send & stream.
        Preset::Telegram => {
            args.extend(["-vf".into(), video_filter(1280, 1280, info, vaapi)]);
            match hw_encoder {
                Some(encoder) => {
                    hw_encoder_flags(&mut args, encoder);
                    args.extend(["-maxrate".into(), "2400k".into(), "-bufsize".into(), "4800k".into()]);
                }
                None => args.extend([
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
                ]),
            }
            audio_aac(&mut args, 128, false);
            args.extend(["-movflags".into(), "+faststart".into()]);
        }
        // Web: ≤1080p; H.264 (fast) or VP9/Opus (smaller), user choice.
        Preset::Web => {
            args.extend(["-vf".into(), video_filter(1920, 1080, info, vaapi)]);
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
                    match hw_encoder {
                        Some(encoder) => hw_encoder_flags(&mut args, encoder),
                        None => args.extend([
                            "-c:v".into(),
                            "libx264".into(),
                            "-preset".into(),
                            "medium".into(),
                            "-crf".into(),
                            "25".into(),
                            "-pix_fmt".into(),
                            "yuv420p".into(),
                        ]),
                    }
                    audio_aac(&mut args, 96, false);
                    args.extend(["-movflags".into(), "+faststart".into()]);
                }
            }
        }
        // Extreme: HEVC (HW → libx265 → libx264), original resolution, mono audio.
        Preset::Extreme => {
            match hw_encoder {
                Some(encoder) => hw_encoder_flags(&mut args, encoder),
                None => {
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
                }
            }
            audio_aac(&mut args, 64, true);
        }
    }

    args.extend(["-progress".into(), "pipe:1".into(), "-nostats".into()]);
    args.push(output.into());
    EncodePlan { args, hw_encoder }
}

/// Build the full ffmpeg argument list for an image job.
pub fn build_image_args(preset: Preset, input: &str, output: &str, strip_metadata: bool) -> Vec<String> {
    let mut args = base_args(input, &[]);
    if strip_metadata {
        strip_metadata_args(&mut args);
    } else {
        keep_metadata_args(&mut args);
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
