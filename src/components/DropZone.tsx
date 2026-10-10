import { useCallback, useEffect } from 'react'
import type {
  KeyboardEvent as ReactKeyboardEvent,
  MouseEvent as ReactMouseEvent,
} from 'react'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { motion } from 'framer-motion'
import { FolderOpen, UploadCloud } from 'lucide-react'

import { PICK_FILES_EVENT } from '../hooks/useHotkeys'
import { ALL_SUPPORTED_EXTENSIONS, api, fileExtension } from '../lib/tauri'
import { formatNumber } from '../lib/format'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

/**
 * Hero drag-and-drop area. Uses the native Tauri webview drag-drop events,
 * falls back to the file dialog via the browse button / Ctrl+O hotkey.
 */
export function DropZone() {
  const { t, lang } = useI18n()
  const preset = useAppStore((s) => s.preset)
  const jobs = useAppStore((s) => s.jobs)
  const isDragging = useAppStore((s) => s.isDragging)
  const setDragging = useAppStore((s) => s.setDragging)
  const addJobs = useAppStore((s) => s.addJobs)
  const toast = useAppStore((s) => s.toast)

  const isEmpty = jobs.length === 0

  const handlePaths = useCallback(
    (paths: string[]) => {
      if (paths.length === 0) return

      const supported = new Set<string>(ALL_SUPPORTED_EXTENSIONS)
      const validPaths = paths.filter((p) => supported.has(fileExtension(p)))

      if (validPaths.length === 0) {
        toast('error', t('toast.unsupported'))
        return
      }

      api
        .enqueueFiles(validPaths, preset)
        .then((newJobs) => {
          addJobs(newJobs)
          toast(
            'success',
            t('toast.enqueued', { count: formatNumber(newJobs.length, lang) }),
          )
        })
        .catch((err: unknown) => {
          const message = String(err)
          toast(
            'error',
            message.includes('ffmpeg_missing') ? t('error.ffmpeg_missing') : message,
          )
        })
    },
    [addJobs, lang, preset, t, toast],
  )

  // Native OS drag & drop (Tauri webview events).
  useEffect(() => {
    let unlisten: (() => void) | undefined
    let disposed = false

    try {
      void getCurrentWebview()
        .onDragDropEvent((event) => {
          const payload = event.payload
          if (payload.type === 'enter' || payload.type === 'over') {
            setDragging(true)
          } else if (payload.type === 'leave') {
            setDragging(false)
          } else if (payload.type === 'drop') {
            setDragging(false)
            handlePaths(payload.paths)
          }
        })
        .then((fn) => {
          if (disposed) fn()
          else unlisten = fn
        })
        .catch(() => undefined)
    } catch {
      // Not running inside Tauri (e.g. plain browser preview) — no native drag-drop.
    }

    return () => {
      disposed = true
      unlisten?.()
    }
  }, [handlePaths, setDragging])

  // Native file picker — browse button and Ctrl/Cmd+O hotkey both land here.
  const browse = useCallback(async () => {
    try {
      const result = await openDialog({
        multiple: true,
        title: t('drop.filter'),
        filters: [{ name: t('drop.filter'), extensions: ALL_SUPPORTED_EXTENSIONS }],
      })
      handlePaths(result ?? [])
    } catch (err) {
      toast('error', String(err))
    }
  }, [handlePaths, t, toast])

  useEffect(() => {
    const onPickFiles = () => {
      void browse()
    }
    window.addEventListener(PICK_FILES_EVENT, onPickFiles)
    return () => window.removeEventListener(PICK_FILES_EVENT, onPickFiles)
  }, [browse])

  const onZoneClick = (event: ReactMouseEvent<HTMLDivElement>) => {
    event.preventDefault()
    void browse()
  }

  const onZoneKeyDown = (event: ReactKeyboardEvent<HTMLDivElement>) => {
    if (
      (event.key === 'Enter' || event.key === ' ') &&
      event.target === event.currentTarget
    ) {
      event.preventDefault()
      void browse()
    }
  }

  const zoneClasses = [
    'relative flex w-full cursor-pointer select-none flex-col items-center justify-center rounded-3xl border-2 border-dashed text-center transition-colors duration-200',
    isDragging
      ? 'border-accent-400 bg-accent-400/[0.06] shadow-glow'
      : 'border-white/[0.08] bg-ink-900/40 hover:border-white/20 hover:bg-ink-900/70',
    isEmpty ? 'min-h-[280px] gap-4 px-6 py-10' : 'h-28 px-4 sm:px-6',
  ].join(' ')

  return (
    <motion.div
      layout
      transition={{ type: 'spring', stiffness: 320, damping: 34 }}
      animate={{ scale: isDragging ? 1.01 : 1 }}
      role="button"
      tabIndex={0}
      aria-label={t('drop.title')}
      onClick={onZoneClick}
      onKeyDown={onZoneKeyDown}
      className={zoneClasses}
    >
      {isDragging ? (
        <div className="flex flex-col items-center gap-2 text-accent-200">
          <UploadCloud className="h-7 w-7" aria-hidden="true" />
          <p className="text-base font-semibold">{t('drop.dragOver')}</p>
        </div>
      ) : isEmpty ? (
        <>
          <div className="flex h-16 w-16 items-center justify-center rounded-2xl bg-gradient-to-br from-accent-400/25 to-accent-600/10 text-accent-300 ring-1 ring-inset ring-white/10">
            <UploadCloud className="h-8 w-8" aria-hidden="true" />
          </div>
          <div className="space-y-1.5">
            <p className="text-lg font-semibold text-zinc-100">{t('drop.title')}</p>
            <p className="mx-auto max-w-md text-sm leading-relaxed text-zinc-500">
              {t('drop.subtitle')}
            </p>
          </div>
          <p className="rounded-full bg-white/[0.04] px-3.5 py-1.5 text-xs text-zinc-600">
            {t('drop.hint')}
          </p>
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation()
              void browse()
            }}
            className="mt-1 inline-flex h-9 items-center gap-2 rounded-full bg-white/5 px-4 text-sm font-medium text-zinc-200 transition-colors hover:bg-white/10"
          >
            <FolderOpen className="h-4 w-4" aria-hidden="true" />
            {t('drop.browse')}
          </button>
        </>
      ) : (
        <div className="flex w-full items-center justify-between gap-3">
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-white/[0.04] text-accent-300">
              <UploadCloud className="h-[18px] w-[18px]" aria-hidden="true" />
            </div>
            <div className="min-w-0 text-start">
              <p className="truncate text-sm font-medium text-zinc-200">
                {t('drop.title')}
              </p>
              <p className="truncate text-xs text-zinc-600">{t('drop.hint')}</p>
            </div>
          </div>
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation()
              void browse()
            }}
            className="inline-flex h-9 shrink-0 items-center gap-2 rounded-full bg-white/5 px-3.5 text-xs font-medium text-zinc-200 transition-colors hover:bg-white/10"
          >
            <FolderOpen className="h-3.5 w-3.5" aria-hidden="true" />
            {t('drop.browse')}
          </button>
        </div>
      )}
    </motion.div>
  )
}
