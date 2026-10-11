import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
  type SyntheticEvent,
} from 'react'
import { convertFileSrc } from '@tauri-apps/api/core'
import { AnimatePresence, motion } from 'framer-motion'
import {
  Columns2,
  Film,
  Image as ImageIcon,
  Loader2,
  Pause,
  Play,
  SlidersHorizontal,
  X,
} from 'lucide-react'

import type { Job } from '../types'
import { api } from '../lib/tauri'
import {
  formatBytes,
  formatDuration,
  formatNumber,
  reductionPercent,
} from '../lib/format'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

type CompareMode = 'slider' | 'split'
type FrameTab = 'first' | 'middle'

const clampPercent = (value: number): number => Math.max(0, Math.min(100, value))

/** Label pill used on top of both compare panes. */
function PaneLabel({ text, tone }: { text: string; tone: 'original' | 'compressed' }) {
  return (
    <span
      className={`pointer-events-none absolute top-2 z-10 rounded-full px-2.5 py-1 text-[11px] font-medium backdrop-blur-sm ${
        tone === 'original'
          ? 'start-2 bg-black/55 text-zinc-100'
          : 'end-2 bg-emerald-500/25 text-emerald-100'
      }`}
    >
      {text}
    </span>
  )
}

interface CompareViewProps {
  beforeUrl: string
  afterUrl: string | null
  mode: CompareMode
  beforeLabel: string
  afterLabel: string
  beforeAlt: string
  afterAlt: string
  onBeforeLoad?: (event: SyntheticEvent<HTMLImageElement>) => void
  onAfterLoad?: (event: SyntheticEvent<HTMLImageElement>) => void
}

/**
 * The visual diff: a draggable before/after wipe, or two panes side by side.
 * The container is forced to LTR so pointer math stays simple in RTL locales.
 */
function CompareView({
  beforeUrl,
  afterUrl,
  mode,
  beforeLabel,
  afterLabel,
  beforeAlt,
  afterAlt,
  onBeforeLoad,
  onAfterLoad,
}: CompareViewProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [position, setPosition] = useState(50)
  const draggingRef = useRef(false)

  const updateFromPointer = useCallback((clientX: number) => {
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect || rect.width === 0) return
    setPosition(clampPercent(((clientX - rect.left) / rect.width) * 100))
  }, [])

  const onPointerDown = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!afterUrl) return
    draggingRef.current = true
    event.currentTarget.setPointerCapture(event.pointerId)
    updateFromPointer(event.clientX)
  }

  const onPointerMove = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (!draggingRef.current) return
    updateFromPointer(event.clientX)
  }

  const stopDragging = () => {
    draggingRef.current = false
  }

  if (!afterUrl || mode === 'split') {
    return (
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2" dir="ltr">
        <div className="relative overflow-hidden rounded-xl bg-ink-900">
          <PaneLabel text={beforeLabel} tone="original" />
          <img
            src={beforeUrl}
            alt={beforeAlt}
            draggable={false}
            onLoad={onBeforeLoad}
            className="mx-auto max-h-[42vh] w-full object-contain"
          />
        </div>
        {afterUrl && (
          <div className="relative overflow-hidden rounded-xl bg-ink-900">
            <PaneLabel text={afterLabel} tone="compressed" />
            <img
              src={afterUrl}
              alt={afterAlt}
              draggable={false}
              onLoad={onAfterLoad}
              className="mx-auto max-h-[42vh] w-full object-contain"
            />
          </div>
        )}
      </div>
    )
  }

  return (
    <div
      ref={containerRef}
      dir="ltr"
      className="relative touch-none select-none overflow-hidden rounded-xl bg-ink-900"
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onPointerUp={stopDragging}
      onPointerCancel={stopDragging}
      role="slider"
      aria-label={afterLabel}
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(position)}
      tabIndex={0}
      onKeyDown={(event) => {
        if (event.key === 'ArrowLeft') setPosition((p) => clampPercent(p - 2))
        if (event.key === 'ArrowRight') setPosition((p) => clampPercent(p + 2))
      }}
    >
      <PaneLabel text={beforeLabel} tone="original" />
      <PaneLabel text={afterLabel} tone="compressed" />
      <img
        src={beforeUrl}
        alt={beforeAlt}
        draggable={false}
        onLoad={onBeforeLoad}
        className="mx-auto block max-h-[42vh] w-full object-contain"
      />
      {/* "after" layer — clipped from the right edge, sized to the full box */}
      <div
        className="absolute inset-0"
        style={{ clipPath: `inset(0 ${100 - position}% 0 0)` }}
      >
        <img
          src={afterUrl}
          alt={afterAlt}
          draggable={false}
          onLoad={onAfterLoad}
          className="absolute inset-0 h-full w-full object-contain"
        />
      </div>
      {/* handle */}
      <div
        className="pointer-events-none absolute inset-y-0 z-10 w-0.5 bg-white/90 shadow-[0_0_8px_rgba(0,0,0,0.6)]"
        style={{ left: `${position}%` }}
      >
        <span className="absolute top-1/2 left-1/2 flex h-8 w-8 -translate-x-1/2 -translate-y-1/2 items-center justify-center rounded-full bg-white text-ink-950 shadow-lg">
          <SlidersHorizontal className="h-4 w-4" aria-hidden="true" />
        </span>
      </div>
    </div>
  )
}

interface DualVideoPlayerProps {
  originalUrl: string
  compressedUrl: string
  originalLabel: string
  compressedLabel: string
}

/** Two <video> elements kept in lockstep by a shared transport. */
function DualVideoPlayer({
  originalUrl,
  compressedUrl,
  originalLabel,
  compressedLabel,
}: DualVideoPlayerProps) {
  const { t, lang } = useI18n()
  const originalRef = useRef<HTMLVideoElement>(null)
  const compressedRef = useRef<HTMLVideoElement>(null)
  const [playing, setPlaying] = useState(false)
  const [time, setTime] = useState(0)
  const [duration, setDuration] = useState(0)

  const toggle = () => {
    const a = originalRef.current
    const b = compressedRef.current
    if (!a || !b) return
    if (playing) {
      a.pause()
      b.pause()
      setPlaying(false)
    } else {
      b.currentTime = a.currentTime
      void a.play()
      void b.play()
      setPlaying(true)
    }
  }

  const seek = (value: number) => {
    const a = originalRef.current
    const b = compressedRef.current
    if (a) a.currentTime = value
    if (b) b.currentTime = value
    setTime(value)
  }

  // Drift correction + time display while playing.
  useEffect(() => {
    if (!playing) return
    const interval = window.setInterval(() => {
      const a = originalRef.current
      const b = compressedRef.current
      if (!a || !b) return
      if (Math.abs(a.currentTime - b.currentTime) > 0.25) {
        b.currentTime = a.currentTime
      }
      setTime(a.currentTime)
      if (a.duration && Number.isFinite(a.duration)) setDuration(a.duration)
    }, 250)
    return () => window.clearInterval(interval)
  }, [playing])

  return (
    <div dir="ltr">
      <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
        <div className="relative overflow-hidden rounded-xl bg-ink-900">
          <PaneLabel text={originalLabel} tone="original" />
          <video
            ref={originalRef}
            src={originalUrl}
            muted
            playsInline
            preload="metadata"
            className="max-h-[34vh] w-full bg-black object-contain"
            onLoadedMetadata={(e) => {
              const d = e.currentTarget.duration
              if (Number.isFinite(d)) setDuration(d)
            }}
            onPlay={() => setPlaying(true)}
            onPause={() => setPlaying(false)}
          />
        </div>
        <div className="relative overflow-hidden rounded-xl bg-ink-900">
          <PaneLabel text={compressedLabel} tone="compressed" />
          <video
            ref={compressedRef}
            src={compressedUrl}
            muted
            playsInline
            preload="metadata"
            className="max-h-[34vh] w-full bg-black object-contain"
          />
        </div>
      </div>

      {/* shared transport */}
      <div className="mt-3 flex items-center gap-3">
        <button
          type="button"
          onClick={toggle}
          aria-label={playing ? t('preview.pause') : t('preview.play')}
          className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-accent-500 text-ink-950 transition-all hover:bg-accent-400 active:scale-95"
        >
          {playing ? (
            <Pause className="h-5 w-5" aria-hidden="true" />
          ) : (
            <Play className="h-5 w-5 translate-x-px" aria-hidden="true" />
          )}
        </button>
        <input
          type="range"
          min={0}
          max={duration || 0}
          step={0.05}
          value={Math.min(time, duration || 0)}
          onChange={(event) => seek(Number(event.target.value))}
          aria-label={originalLabel}
          className="h-1.5 w-full cursor-pointer accent-accent-500"
        />
        <span className="w-24 shrink-0 text-end font-mono text-xs text-zinc-400">
          {formatDuration(time, lang)} / {formatDuration(duration, lang)}
        </span>
      </div>
    </div>
  )
}

/** Stats line shared by both media kinds. */
function StatsRow({
  job,
  originalDims,
  outputDims,
  durationSecs,
}: {
  job: Job
  originalDims: string | null
  outputDims: string | null
  durationSecs: number | null
}) {
  const { t, lang } = useI18n()
  const { outputSize, inputSize } = job

  return (
    <div className="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-xl bg-white/[0.04] px-4 py-3 text-xs text-zinc-300">
      <span className="flex items-center gap-1.5">
        {formatBytes(inputSize, lang)}
        {outputSize != null && (
          <>
            <span className="text-zinc-600">→</span>
            <strong className="font-semibold text-emerald-300">
              {formatBytes(outputSize, lang)}
            </strong>
            <span className="rounded-full bg-emerald-400/10 px-2 py-0.5 font-medium text-emerald-300">
              {t('card.reduction', {
                percent: formatNumber(
                  reductionPercent(inputSize, outputSize),
                  lang,
                ),
              })}
            </span>
          </>
        )}
      </span>
      {durationSecs != null && (
        <span className="text-zinc-500">
          {t('preview.duration')}: {formatDuration(durationSecs, lang)}
        </span>
      )}
      {originalDims != null && (
        <span className="text-zinc-500" dir="ltr">
          {t('preview.dimensions')}:{' '}
          <span className="font-mono">
            {originalDims}
            {outputDims != null && outputDims !== originalDims && (
              <> → {outputDims}</>
            )}
          </span>
        </span>
      )}
    </div>
  )
}

export function PreviewDialog() {
  const { t } = useI18n()
  const previewJobId = useAppStore((s) => s.previewJobId)
  const setPreviewJobId = useAppStore((s) => s.setPreviewJobId)
  const jobs = useAppStore((s) => s.jobs)

  const job: Job | null = previewJobId
    ? (jobs.find((j) => j.id === previewJobId) ?? null)
    : null

  // Primitive snapshots keep the effects stable while the store mutates `job`.
  const jobId = job?.id ?? null
  const kind = job?.kind ?? null
  const inputPath = job?.inputPath ?? null
  const outputPath = job?.outputPath ?? null

  const [mode, setMode] = useState<CompareMode>('slider')
  const [frameTab, setFrameTab] = useState<FrameTab>('first')
  const [loading, setLoading] = useState(false)
  const [failed, setFailed] = useState(false)
  const [durationSecs, setDurationSecs] = useState<number | null>(null)
  const [videoUrls, setVideoUrls] = useState<{
    inputFirst: string
    inputMiddle: string
    outputFirst: string | null
    outputMiddle: string | null
  } | null>(null)
  const [dims, setDims] = useState<{ original: string | null; output: string | null }>({
    original: null,
    output: null,
  })

  const isVideo = kind === 'video'
  const open = Boolean(job && job.status === 'done' && outputPath)

  // Reset transient state whenever a different job opens.
  useEffect(() => {
    setMode('slider')
    setFrameTab('first')
    setVideoUrls(null)
    setFailed(false)
    setDurationSecs(null)
    setDims({ original: null, output: null })
  }, [jobId])

  // Extract frames for videos once per open (primitive deps → no re-runs on
  // unrelated store updates).
  useEffect(() => {
    if (!jobId || kind !== 'video' || !inputPath) return
    let active = true
    setLoading(true)
    setFailed(false)
    api
      .videoPreview(inputPath, outputPath)
      .then((preview) => {
        if (!active) return
        setVideoUrls({
          inputFirst: convertFileSrc(preview.inputFirst),
          inputMiddle: convertFileSrc(preview.inputMiddle),
          outputFirst: preview.outputFirst
            ? convertFileSrc(preview.outputFirst)
            : null,
          outputMiddle: preview.outputMiddle
            ? convertFileSrc(preview.outputMiddle)
            : null,
        })
        setDurationSecs(preview.durationSecs)
        if (preview.width && preview.height) {
          setDims((d) => ({
            ...d,
            original: `${preview.width}×${preview.height}`,
          }))
        }
      })
      .catch(() => {
        if (active) setFailed(true)
      })
      .finally(() => {
        if (active) setLoading(false)
      })
    return () => {
      active = false
    }
  }, [jobId, kind, inputPath, outputPath])

  // Esc closes the dialog.
  useEffect(() => {
    if (!previewJobId) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setPreviewJobId(null)
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [previewJobId, setPreviewJobId])

  const captureDims = useCallback(
    (which: 'original' | 'output') =>
      (event: SyntheticEvent<HTMLImageElement>) => {
        const img = event.currentTarget
        if (!img.naturalWidth) return
        setDims((d) => ({
          ...d,
          [which]: `${img.naturalWidth}×${img.naturalHeight}`,
        }))
      },
    [],
  )

  let body: ReactNode = null

  if (job && open) {
    if (isVideo) {
      const beforeUrl = videoUrls
        ? frameTab === 'first'
          ? videoUrls.inputFirst
          : videoUrls.inputMiddle
        : null
      const afterUrl = videoUrls
        ? frameTab === 'first'
          ? videoUrls.outputFirst
          : videoUrls.outputMiddle
        : null
      body = (
        <div className="flex flex-col gap-4">
          {loading && (
            <div className="flex h-48 items-center justify-center gap-2 text-sm text-zinc-400">
              <Loader2 className="h-4 w-4 animate-spin" aria-hidden="true" />
              {t('preview.loading')}
            </div>
          )}
          {!loading && !failed && beforeUrl && (
            <>
              <CompareView
                key={`${jobId}-${frameTab}`}
                beforeUrl={beforeUrl}
                afterUrl={afterUrl}
                mode={mode}
                beforeLabel={t('preview.original')}
                afterLabel={t('preview.compressed')}
                beforeAlt={t('preview.original')}
                afterAlt={t('preview.compressed')}
                onAfterLoad={captureDims('output')}
              />
              {!afterUrl && (
                <p className="text-center text-xs text-zinc-500">
                  {t('preview.outputMissing')}
                </p>
              )}
            </>
          )}
          {!loading && failed && (
            <div className="flex h-24 items-center justify-center text-sm text-rose-300">
              {t('preview.failed')}
            </div>
          )}

          <div className="grid grid-cols-2 gap-2">
            {(
              [
                ['first', 'preview.frame.first'],
                ['middle', 'preview.frame.middle'],
              ] as const
            ).map(([tab, key]) => (
              <button
                key={tab}
                type="button"
                aria-pressed={frameTab === tab}
                onClick={() => setFrameTab(tab)}
                className={`inline-flex h-9 items-center justify-center rounded-xl border px-3 text-xs font-medium transition-colors ${
                  frameTab === tab
                    ? 'border-accent-400/60 bg-accent-400/10 text-accent-200'
                    : 'border-white/[0.06] bg-ink-900 text-zinc-400 hover:border-white/15 hover:text-zinc-200'
                }`}
              >
                {t(key)}
              </button>
            ))}
          </div>

          {inputPath && outputPath && !loading && !failed && (
            <DualVideoPlayer
              originalUrl={convertFileSrc(inputPath)}
              compressedUrl={convertFileSrc(outputPath)}
              originalLabel={t('preview.original')}
              compressedLabel={t('preview.compressed')}
            />
          )}
        </div>
      )
    } else if (inputPath) {
      body = (
        <CompareView
          key={jobId ?? 'image'}
          beforeUrl={convertFileSrc(inputPath)}
          afterUrl={outputPath ? convertFileSrc(outputPath) : null}
          mode={mode}
          beforeLabel={t('preview.original')}
          afterLabel={t('preview.compressed')}
          beforeAlt={t('preview.original')}
          afterAlt={t('preview.compressed')}
          onBeforeLoad={captureDims('original')}
          onAfterLoad={captureDims('output')}
        />
      )
    }
  }

  return (
    <AnimatePresence>
      {open && job && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          onClick={() => setPreviewJobId(null)}
        >
          <motion.div
            role="dialog"
            aria-modal="true"
            aria-label={t('preview.title')}
            tabIndex={-1}
            className="panel max-h-[calc(100vh-3rem)] w-full max-w-3xl overflow-y-auto p-6 outline-none"
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: 4 }}
            transition={{ duration: 0.18, ease: 'easeOut' }}
            onClick={(event) => event.stopPropagation()}
          >
            {/* header */}
            <div className="flex items-center justify-between gap-3">
              <h2 className="flex min-w-0 items-center gap-2 text-base font-semibold text-zinc-100">
                {isVideo ? (
                  <Film className="h-4 w-4 shrink-0 text-accent-300" aria-hidden="true" />
                ) : (
                  <ImageIcon className="h-4 w-4 shrink-0 text-amber-300" aria-hidden="true" />
                )}
                <span className="truncate" dir="auto" title={job.fileName}>
                  {job.fileName}
                </span>
              </h2>
              <div className="flex shrink-0 items-center gap-2">
                {/* mode switch */}
                <div
                  role="group"
                  aria-label={t('preview.title')}
                  className="flex items-center rounded-xl border border-white/[0.06] bg-ink-900 p-1"
                >
                  <button
                    type="button"
                    aria-pressed={mode === 'slider'}
                    aria-label={t('preview.mode.slider')}
                    onClick={() => setMode('slider')}
                    className={`inline-flex h-7 items-center gap-1.5 rounded-lg px-2.5 text-xs font-medium transition-colors ${
                      mode === 'slider'
                        ? 'bg-accent-500/15 text-accent-300'
                        : 'text-zinc-400 hover:text-zinc-200'
                    }`}
                  >
                    <SlidersHorizontal className="h-3.5 w-3.5" aria-hidden="true" />
                    <span className="hidden sm:inline">{t('preview.mode.slider')}</span>
                  </button>
                  <button
                    type="button"
                    aria-pressed={mode === 'split'}
                    aria-label={t('preview.mode.split')}
                    onClick={() => setMode('split')}
                    className={`inline-flex h-7 items-center gap-1.5 rounded-lg px-2.5 text-xs font-medium transition-colors ${
                      mode === 'split'
                        ? 'bg-accent-500/15 text-accent-300'
                        : 'text-zinc-400 hover:text-zinc-200'
                    }`}
                  >
                    <Columns2 className="h-3.5 w-3.5" aria-hidden="true" />
                    <span className="hidden sm:inline">{t('preview.mode.split')}</span>
                  </button>
                </div>
                <button
                  type="button"
                  className="icon-btn"
                  aria-label={t('common.close')}
                  onClick={() => setPreviewJobId(null)}
                >
                  <X className="h-4 w-4" aria-hidden="true" />
                </button>
              </div>
            </div>

            <div className="mt-4 flex flex-col gap-4">
              {body}
              <StatsRow
                job={job}
                originalDims={dims.original}
                outputDims={dims.output}
                durationSecs={isVideo ? durationSecs : null}
              />
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
