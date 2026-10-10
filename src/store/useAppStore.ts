import { create } from 'zustand'

import type { AppSettings, FfmpegStatus, Job, JobUpdate, Preset } from '../types'
import type { Lang } from '../i18n'

export type ToastKind = 'success' | 'error' | 'info'

export interface ToastItem {
  id: number
  kind: ToastKind
  message: string
}

let toastSeq = 1

interface AppState {
  // Preferences
  lang: Lang
  setLang: (lang: Lang) => void

  preset: Preset
  setPreset: (preset: Preset) => void

  // Backend state
  ffmpeg: FfmpegStatus | null
  setFfmpeg: (status: FfmpegStatus | null) => void

  settings: AppSettings | null
  setSettings: (settings: AppSettings) => void

  settingsOpen: boolean
  setSettingsOpen: (open: boolean) => void

  // Queue
  jobs: Job[]
  addJobs: (jobs: Job[]) => void
  applyUpdate: (update: JobUpdate) => void
  removeJob: (id: string) => void
  clearFinished: () => void

  // Dropzone
  isDragging: boolean
  setDragging: (dragging: boolean) => void

  // Toasts
  toasts: ToastItem[]
  toast: (kind: ToastKind, message: string) => void
  dismissToast: (id: number) => void
}

const isFinished = (status: Job['status']) =>
  status === 'done' || status === 'error' || status === 'canceled'

export const useAppStore = create<AppState>((set) => ({
  lang: 'fa',
  setLang: (lang) => set({ lang }),

  preset: 'telegram',
  setPreset: (preset) => set({ preset }),

  ffmpeg: null,
  setFfmpeg: (ffmpeg) => set({ ffmpeg }),

  settings: null,
  setSettings: (settings) => set({ settings }),

  settingsOpen: false,
  setSettingsOpen: (settingsOpen) => set({ settingsOpen }),

  jobs: [],
  addJobs: (incoming) =>
    set((state) => {
      const known = new Set(state.jobs.map((j) => j.id))
      const fresh = incoming.filter((j) => !known.has(j.id))
      return { jobs: [...state.jobs, ...fresh] }
    }),
  applyUpdate: (update) =>
    set((state) => ({
      jobs: state.jobs.map((job) =>
        job.id === update.id
          ? {
              ...job,
              status: update.status ?? job.status,
              progress:
                update.progress === undefined ? job.progress : update.progress,
              etaSecs: update.etaSecs === undefined ? job.etaSecs : update.etaSecs,
              fps: update.fps === undefined ? job.fps : update.fps,
              kbps: update.kbps === undefined ? job.kbps : update.kbps,
              outputPath:
                update.outputPath === undefined
                  ? job.outputPath
                  : update.outputPath,
              outputSize:
                update.outputSize === undefined ? job.outputSize : update.outputSize,
              error: update.error === undefined ? job.error : update.error,
            }
          : job,
      ),
    })),
  removeJob: (id) => set((state) => ({ jobs: state.jobs.filter((j) => j.id !== id) })),
  clearFinished: () => set((state) => ({ jobs: state.jobs.filter((j) => !isFinished(j.status)) })),

  isDragging: false,
  setDragging: (isDragging) => set({ isDragging }),

  toasts: [],
  toast: (kind, message) =>
    set((state) => ({
      toasts: [...state.toasts, { id: toastSeq++, kind, message }].slice(-4),
    })),
  dismissToast: (id) =>
    set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) })),
}))

/** Selector helper: number of jobs currently queued/processing. */
export function activeJobCount(jobs: Job[]): number {
  return jobs.filter((j) => j.status === 'queued' || j.status === 'probing' || j.status === 'processing').length
}

/** Selector helper: number of finished jobs (done/error/canceled). */
export function finishedJobCount(jobs: Job[]): number {
  return jobs.filter((j) => isFinished(j.status)).length
}
