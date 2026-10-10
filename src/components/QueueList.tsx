import { AnimatePresence, motion } from 'framer-motion'
import { Trash2 } from 'lucide-react'

import { FileCard } from './FileCard'
import { formatNumber } from '../lib/format'
import { useI18n } from '../i18n'
import { finishedJobCount, useAppStore } from '../store/useAppStore'

export function QueueList() {
  const { t, lang } = useI18n()
  const jobs = useAppStore((s) => s.jobs)
  const clearFinished = useAppStore((s) => s.clearFinished)
  const finished = finishedJobCount(jobs)

  // The hero DropZone already covers the empty state.
  if (jobs.length === 0) return null

  return (
    <section aria-label={t('queue.title')}>
      <div className="mb-3 flex items-center justify-between gap-2">
        <div className="flex items-center gap-2">
          <h2 className="text-sm font-medium text-zinc-400">{t('queue.title')}</h2>
          <span className="rounded-full bg-white/5 px-2.5 py-0.5 text-xs text-zinc-400">
            {t('queue.count', { count: formatNumber(jobs.length, lang) })}
          </span>
        </div>
        {finished > 0 && (
          <button
            type="button"
            onClick={clearFinished}
            className="inline-flex h-9 items-center gap-1.5 rounded-lg px-2 text-xs text-zinc-400 transition-colors hover:text-rose-300"
          >
            <Trash2 className="h-3.5 w-3.5" aria-hidden="true" />
            {t('queue.clear')}
          </button>
        )}
      </div>

      <motion.ul layout className="flex flex-col gap-3">
        <AnimatePresence mode="popLayout" initial={false}>
          {jobs.map((job) => (
            <FileCard key={job.id} job={job} />
          ))}
        </AnimatePresence>
      </motion.ul>
    </section>
  )
}
