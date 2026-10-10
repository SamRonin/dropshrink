import { forwardRef } from 'react'
import { motion } from 'framer-motion'
import {
  AlertTriangle,
  ArrowRight,
  Ban,
  Check,
  Clock,
  Film,
  FolderOpen,
  Image as ImageIcon,
  Loader2,
  RotateCcw,
  X,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

import type { Job, JobStatus } from '../types'
import { api } from '../lib/tauri'
import {
  formatBytes,
  formatEta,
  formatNumber,
  formatPercent,
  reductionPercent,
} from '../lib/format'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

interface FileCardProps {
  job: Job
}

interface StatusVisual {
  icon: LucideIcon
  label: string
  spin: boolean
}

const PILL_CLASSES: Record<JobStatus, string> = {
  queued: 'bg-white/5 text-zinc-400',
  probing: 'bg-white/5 text-zinc-400',
  processing: 'bg-accent-400/10 text-accent-300',
  done: 'bg-emerald-400/10 text-emerald-300',
  error: 'bg-rose-400/10 text-rose-300',
  canceled: 'bg-white/5 text-zinc-400',
}

function statusVisual(status: JobStatus, t: (key: string) => string): StatusVisual {
  switch (status) {
    case 'queued':
      return { icon: Clock, label: t('status.queued'), spin: false }
    case 'probing':
      return { icon: Loader2, label: t('status.probing'), spin: true }
    case 'processing':
      return { icon: Loader2, label: t('status.processing'), spin: true }
    case 'done':
      return { icon: Check, label: t('status.done'), spin: false }
    case 'error':
      return { icon: AlertTriangle, label: t('status.error'), spin: false }
    case 'canceled':
      return { icon: Ban, label: t('status.canceled'), spin: false }
  }
}

const clampPercent = (value: number): number => Math.max(0, Math.min(100, value))

export const FileCard = forwardRef<HTMLLIElement, FileCardProps>(function FileCard(
  { job },
  ref,
) {
  const { t, lang } = useI18n()
  const applyUpdate = useAppStore((s) => s.applyUpdate)
  const removeJob = useAppStore((s) => s.removeJob)
  const toast = useAppStore((s) => s.toast)

  const visual = statusVisual(job.status, t)
  const StatusIcon = visual.icon
  const percentSuffix = lang === 'fa' ? '٪' : '%'

  const canCancel = job.status === 'queued' || job.status === 'processing'
  const canReveal = job.status === 'done'
  const canRetry = job.status === 'error' || job.status === 'canceled'
  const canRemove =
    job.status === 'done' || job.status === 'error' || job.status === 'canceled'

  const onCancel = () => {
    api.cancelJob(job.id).catch((err: unknown) => toast('error', String(err)))
  }

  const onReveal = () => {
    api.revealPath(job.outputPath ?? job.inputPath).catch(() => undefined)
  }

  const onRetry = () => {
    api
      .retryJob(job.id)
      .then(() => {
        applyUpdate({ id: job.id, status: 'queued', progress: null, error: null })
        toast('success', t('toast.retried'))
      })
      .catch((err: unknown) => toast('error', String(err)))
  }

  return (
    <motion.li
      ref={ref}
      layout
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, scale: 0.97 }}
      transition={{ duration: 0.18, ease: 'easeOut' }}
      className="list-none"
    >
      <div className="panel p-4">
        {/* Row 1 — identity */}
        <div className="flex items-start gap-3">
          <div
            className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl ${
              job.kind === 'video'
                ? 'bg-accent-400/10 text-accent-300'
                : 'bg-amber-400/10 text-amber-300'
            }`}
          >
            {job.kind === 'video' ? (
              <Film className="h-5 w-5" aria-hidden="true" />
            ) : (
              <ImageIcon className="h-5 w-5" aria-hidden="true" />
            )}
          </div>
          <div className="min-w-0 flex-1 pt-0.5">
            <p
              className="truncate text-sm font-medium text-zinc-100"
              dir="auto"
              title={job.inputPath}
            >
              {job.fileName}
            </p>
            <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-zinc-500">
              <span>{formatBytes(job.inputSize, lang)}</span>
              {job.status === 'done' && job.outputSize != null && (
                <>
                  <ArrowRight
                    className="h-3 w-3 text-zinc-600 rtl:rotate-180"
                    aria-hidden="true"
                  />
                  <span>{formatBytes(job.outputSize, lang)}</span>
                  <span className="rounded-full bg-emerald-400/10 px-2 py-0.5 font-medium text-emerald-300">
                    {t('card.reduction', {
                      percent: formatNumber(
                        reductionPercent(job.inputSize, job.outputSize),
                        lang,
                      ),
                    })}
                  </span>
                </>
              )}
            </div>
          </div>
        </div>

        {/* Row 2 — progress zone */}
        <div className="mt-3 flex items-center gap-2">
          <span
            className={`inline-flex items-center gap-1.5 rounded-full px-2 py-0.5 text-[11px] font-medium ${PILL_CLASSES[job.status]}`}
          >
            <StatusIcon
              className={`h-3 w-3 ${visual.spin ? 'animate-spin' : ''}`}
              aria-hidden="true"
            />
            {visual.label}
          </span>
        </div>

        {job.status === 'processing' && (
          <>
            <div className="mt-2.5 h-1.5 overflow-hidden rounded-full bg-white/[0.06]">
              {job.progress != null ? (
                <motion.div
                  className="h-full rounded-full bg-gradient-to-r from-accent-400 to-emerald-400"
                  initial={false}
                  animate={{ width: `${clampPercent(job.progress)}%` }}
                  transition={{ duration: 0.3, ease: 'easeOut' }}
                />
              ) : (
                <div className="h-full w-full overflow-hidden rounded-full">
                  <div className="h-full w-1/3 animate-shimmer rounded-full bg-gradient-to-r from-accent-400/70 via-emerald-400/70 to-accent-400/70" />
                </div>
              )}
            </div>
            <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-zinc-500">
              {job.etaSecs != null && (
                <span>{t('card.eta', { time: formatEta(job.etaSecs, lang) })}</span>
              )}
              {job.fps != null && (
                <span>
                  {t('card.processing', {
                    fps: formatNumber(Math.round(job.fps), lang),
                    kbps: formatNumber(job.kbps ?? 0, lang),
                  })}
                </span>
              )}
              {job.progress != null && (
                <span className="font-medium text-zinc-400">
                  {formatPercent(job.progress, lang)}
                  {percentSuffix}
                </span>
              )}
            </div>
          </>
        )}

        {job.error && (
          <p className="mt-2 break-words text-xs text-rose-300/90" dir="auto">
            {job.error.startsWith('error.') ? (
              t(job.error)
            ) : (
              <span className="font-mono text-[11px]">{job.error}</span>
            )}
          </p>
        )}

        {/* Row 3 — actions */}
        <div className="mt-3 flex items-center justify-end gap-1 border-t border-white/[0.04] pt-2.5">
          {canCancel && (
            <button
              type="button"
              className="icon-btn"
              aria-label={t('card.cancel')}
              onClick={onCancel}
            >
              <X className="h-4 w-4" aria-hidden="true" />
            </button>
          )}
          {canReveal && (
            <button
              type="button"
              className="icon-btn"
              aria-label={t('card.showInFolder')}
              onClick={onReveal}
            >
              <FolderOpen className="h-4 w-4" aria-hidden="true" />
            </button>
          )}
          {canRetry && (
            <button
              type="button"
              className="icon-btn"
              aria-label={t('card.retry')}
              onClick={onRetry}
            >
              <RotateCcw className="h-4 w-4" aria-hidden="true" />
            </button>
          )}
          {canRemove && (
            <button
              type="button"
              className="icon-btn"
              aria-label={t('card.remove')}
              onClick={() => removeJob(job.id)}
            >
              <X className="h-4 w-4" aria-hidden="true" />
            </button>
          )}
        </div>
      </div>
    </motion.li>
  )
})
