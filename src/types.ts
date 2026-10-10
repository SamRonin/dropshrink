/**
 * Shared contract between the React frontend and the Rust (Tauri) backend.
 * The Rust side mirrors these shapes with `#[serde(rename_all = "camelCase")]`.
 */

export type Preset = 'telegram' | 'web' | 'extreme'

export type JobKind = 'video' | 'image'

export type JobStatus =
  | 'queued'
  | 'probing'
  | 'processing'
  | 'done'
  | 'error'
  | 'canceled'

export interface Job {
  id: string
  inputPath: string
  outputPath: string | null
  fileName: string
  kind: JobKind
  preset: Preset
  inputSize: number
  status: JobStatus
  /** 0..100; `null` means indeterminate (e.g. images). */
  progress: number | null
  etaSecs: number | null
  fps: number | null
  kbps: number | null
  outputSize: number | null
  /** Either an i18n error code (`error.*`) or a raw ffmpeg message. */
  error: string | null
}

/** Payload of the `job-update` event emitted by the Rust backend. */
export interface JobUpdate {
  id: string
  status?: JobStatus
  progress?: number | null
  etaSecs?: number | null
  fps?: number | null
  kbps?: number | null
  outputPath?: string | null
  outputSize?: number | null
  error?: string | null
}

export type FfmpegSource = 'bundled' | 'system' | 'none'

export interface FfmpegStatus {
  ffmpegPath: string | null
  ffprobePath: string | null
  version: string | null
  source: FfmpegSource
  hasX265: boolean
  hasVp9: boolean
}

export type WebCodec = 'h264' | 'vp9'

export interface AppSettings {
  maxConcurrent: number
  webCodec: WebCodec
  outputDir: string | null
  stripMetadata: boolean
}
