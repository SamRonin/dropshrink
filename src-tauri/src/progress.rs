use std::time::{Duration, Instant};

/// One parsed progress snapshot from `ffmpeg -progress pipe:1`.
#[derive(Debug, Clone, Copy)]
pub struct ProgressTick {
    pub percent: Option<f32>,
    pub eta_secs: Option<u64>,
    pub fps: Option<f32>,
    pub kbps: Option<u64>,
    pub finished: bool,
}

/// Incrementally parses ffmpeg's key=value progress stream.
/// Emits a tick at most every 250ms (and always on `progress=end`).
pub struct ProgressParser {
    total_secs: Option<f64>,
    out_secs: Option<f64>,
    fps: Option<f32>,
    kbps: Option<u64>,
    speed: Option<f32>,
    last_emit: Instant,
}

impl ProgressParser {
    pub fn new(total_secs: Option<f64>) -> Self {
        Self {
            total_secs,
            out_secs: None,
            fps: None,
            kbps: None,
            speed: None,
            // Emit the very first tick immediately.
            last_emit: Instant::now() - Duration::from_millis(500),
        }
    }

    fn should_emit(&self) -> bool {
        self.last_emit.elapsed() >= Duration::from_millis(250)
    }

    /// Feed one line; returns `Some(tick)` when it is time to update the UI.
    pub fn feed(&mut self, line: &str) -> Option<ProgressTick> {
        let (key, value) = line.split_once('=')?;
        let value = value.trim();
        match key.trim() {
            // Despite the name, `out_time_ms` is reported in microseconds.
            "out_time_us" | "out_time_ms" => {
                if let Ok(micros) = value.parse::<f64>() {
                    self.out_secs = Some(micros / 1_000_000.0);
                }
            }
            "out_time" => {
                self.out_secs = parse_clock(value);
            }
            "fps" => {
                if let Ok(v) = value.parse::<f32>() {
                    if v.is_finite() && v > 0.0 {
                        self.fps = Some(v);
                    }
                }
            }
            "bitrate" => {
                self.kbps = parse_bitrate_kbps(value);
            }
            "speed" => {
                if let Ok(v) = value.trim_end_matches('x').parse::<f32>() {
                    if v.is_finite() && v > 0.0 {
                        self.speed = Some(v);
                    }
                }
            }
            "progress" => {
                let tick = self.build_tick(value == "end");
                self.last_emit = Instant::now();
                return Some(tick);
            }
            _ => {}
        }

        if self.should_emit() {
            self.last_emit = Instant::now();
            Some(self.build_tick(false))
        } else {
            None
        }
    }

    fn build_tick(&self, finished: bool) -> ProgressTick {
        let (percent, eta_secs) = match (self.total_secs, self.out_secs) {
            (Some(total), Some(done)) if total > 0.0 => {
                // Cap at 99 until the process really exits with code 0.
                let pct = ((done / total) * 100.0).clamp(0.0, 99.0) as f32;
                let eta = if finished {
                    Some(0)
                } else {
                    self.speed
                        .filter(|s| *s > 0.05)
                        .map(|s| ((total - done).max(0.0) / s as f64) as u64)
                };
                (Some(pct), eta)
            }
            _ => (None, None),
        };
        ProgressTick {
            percent,
            eta_secs,
            fps: self.fps,
            kbps: self.kbps,
            finished,
        }
    }
}

fn parse_clock(value: &str) -> Option<f64> {
    if value == "N/A" {
        return None;
    }
    let mut parts = value.split(':');
    let h: f64 = parts.next()?.parse().ok()?;
    let m: f64 = parts.next()?.parse().ok()?;
    let s: f64 = parts.next().unwrap_or("0").parse().ok()?;
    Some(h * 3600.0 + m * 60.0 + s)
}

fn parse_bitrate_kbps(value: &str) -> Option<u64> {
    if value == "N/A" {
        return None;
    }
    let num = value.split("kbits").next()?;
    num.trim().parse::<f64>().ok().map(|v| v as u64)
}
