import { ShieldCheck, Zap } from 'lucide-react'

import { formatNumber } from '../lib/format'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

export function Footer() {
  const { t, lang } = useI18n()
  const ffmpeg = useAppStore((s) => s.ffmpeg)
  const jobs = useAppStore((s) => s.jobs)

  const healthy = ffmpeg != null && ffmpeg.source !== 'none'
  const done = jobs.filter((j) => j.status === 'done').length

  const engineLabel =
    ffmpeg != null && ffmpeg.version != null && ffmpeg.source !== 'none'
      ? `FFmpeg ${ffmpeg.version} · ${t(`footer.source.${ffmpeg.source}`)}`
      : t('footer.missing')

  const hw = ffmpeg?.hwAccel
  const hwTitle = hw?.available ? hw.encoders.join(', ') : undefined

  return (
    <footer className="mt-auto flex h-11 shrink-0 items-center justify-between gap-3 border-t border-white/5 px-4 text-xs text-zinc-500 sm:px-6">
      <div
        className="flex min-w-0 items-center gap-2"
        title={healthy ? undefined : t('footer.missingHint')}
      >
        <span
          aria-hidden="true"
          className={`h-2 w-2 shrink-0 rounded-full ${
            healthy ? 'animate-pulse bg-emerald-400' : 'bg-rose-400'
          }`}
        />
        <span className="truncate">
          {t('footer.engine')} · {engineLabel}
        </span>
        {hw?.available && (
          <span
            className="hidden shrink-0 items-center gap-1 rounded-full bg-accent-400/10 px-2 py-0.5 font-medium text-accent-300 sm:inline-flex"
            title={hwTitle}
          >
            <Zap className="h-3 w-3" aria-hidden="true" />
            {t('footer.hwChip')}
          </span>
        )}
      </div>

      <div className="flex shrink-0 items-center gap-3">
        <span className="hidden sm:inline">
          {t('footer.jobs', {
            done: formatNumber(done, lang),
            total: formatNumber(jobs.length, lang),
          })}
        </span>
        <span className="inline-flex items-center gap-1.5 text-emerald-300/80">
          <ShieldCheck className="h-3.5 w-3.5" aria-hidden="true" />
          {t('header.offline')}
        </span>
      </div>
    </footer>
  )
}
