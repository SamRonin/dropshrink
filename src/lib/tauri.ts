import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

import type {
  AppSettings,
  FfmpegStatus,
  Job,
  JobUpdate,
  Preset,
} from '../types'

/**
 * Typed wrappers around the Tauri commands implemented in src-tauri.
 * Keep this file in sync with the Rust command signatures.
 */
export const api = {
  detectFfmpeg: () => invoke<FfmpegStatus>('detect_ffmpeg'),

  enqueueFiles: (paths: string[], preset: Preset) =>
    invoke<Job[]>('enqueue_files', { paths, preset }),

  cancelJob: (id: string) => invoke<void>('cancel_job', { id }),

  retryJob: (id: string) => invoke<Job | null>('retry_job', { id }),

  getSettings: () => invoke<AppSettings>('get_settings'),

  updateSettings: (settings: AppSettings) =>
    invoke<void>('update_settings', { settings }),

  revealPath: (path: string) => invoke<void>('reveal_path', { path }),
}

/** Subscribe to live job progress updates emitted by the Rust backend. */
export function listenJobUpdate(
  handler: (update: JobUpdate) => void,
): Promise<() => void> {
  return listen<JobUpdate>('job-update', (event) => handler(event.payload))
}

export const VIDEO_EXTENSIONS = [
  'mp4',
  'mkv',
  'mov',
  'webm',
  'm4v',
  'avi',
  'wmv',
  'flv',
  'ts',
  'mpg',
  'mpeg',
  '3gp',
]

export const IMAGE_EXTENSIONS = [
  'jpg',
  'jpeg',
  'png',
  'webp',
  'bmp',
  'tif',
  'tiff',
  'gif',
  'avif',
]

export const ALL_SUPPORTED_EXTENSIONS = [
  ...VIDEO_EXTENSIONS,
  ...IMAGE_EXTENSIONS,
]

export function fileExtension(path: string): string {
  const name = path.split(/[\\/]/).pop() ?? ''
  const dot = name.lastIndexOf('.')
  return dot === -1 ? '' : name.slice(dot + 1).toLowerCase()
}
