import { useEffect, useRef, useState } from 'react'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { AnimatePresence, motion } from 'framer-motion'
import { FolderOpen } from 'lucide-react'

import type { AppSettings, WebCodec } from '../types'
import { api } from '../lib/tauri'
import { formatNumber } from '../lib/format'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

const MAX_CONCURRENT_RANGE = 8

const CODEC_LABEL_KEYS: Record<WebCodec, string> = {
  h264: 'settings.webCodec.h264',
  vp9: 'settings.webCodec.vp9',
}

const DEFAULT_SETTINGS: AppSettings = {
  maxConcurrent: 2,
  webCodec: 'h264',
  outputDir: null,
  stripMetadata: true,
}

export function SettingsDialog() {
  const { t, lang } = useI18n()
  const settingsOpen = useAppStore((s) => s.settingsOpen)
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen)
  const settings = useAppStore((s) => s.settings)
  const setSettings = useAppStore((s) => s.setSettings)
  const toast = useAppStore((s) => s.toast)

  const [draft, setDraft] = useState<AppSettings | null>(null)
  const panelRef = useRef<HTMLDivElement>(null)

  // Re-seed the draft from the store every time the dialog opens.
  useEffect(() => {
    if (settingsOpen) {
      setDraft(settings ?? DEFAULT_SETTINGS)
    }
  }, [settingsOpen, settings])

  // Move keyboard focus into the dialog when it opens.
  useEffect(() => {
    if (!settingsOpen) return
    const raf = requestAnimationFrame(() => panelRef.current?.focus())
    return () => cancelAnimationFrame(raf)
  }, [settingsOpen])

  // Esc closes the dialog.
  useEffect(() => {
    if (!settingsOpen) return
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') setSettingsOpen(false)
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [settingsOpen, setSettingsOpen])

  const pickOutputDir = async () => {
    try {
      const result = await openDialog({ directory: true })
      if (typeof result === 'string') {
        setDraft((current) =>
          current ? { ...current, outputDir: result } : current,
        )
      }
      // Canceled (null) → keep the previous value.
    } catch (err) {
      toast('error', String(err))
    }
  }

  const save = async () => {
    if (!draft) return
    try {
      await api.updateSettings(draft)
      setSettings(draft)
      toast('success', t('toast.settingsSaved'))
      setSettingsOpen(false)
    } catch (err) {
      toast('error', String(err))
    }
  }

  return (
    <AnimatePresence>
      {settingsOpen && draft && (
        <motion.div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          onClick={() => setSettingsOpen(false)}
        >
          <motion.div
            ref={panelRef}
            role="dialog"
            aria-modal="true"
            aria-label={t('settings.title')}
            tabIndex={-1}
            className="panel w-full max-w-md p-6 outline-none"
            initial={{ opacity: 0, scale: 0.96, y: 8 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.97, y: 4 }}
            transition={{ duration: 0.18, ease: 'easeOut' }}
            onClick={(event) => event.stopPropagation()}
          >
            <h2 className="text-base font-semibold text-zinc-100">
              {t('settings.title')}
            </h2>

            <div className="mt-5 flex flex-col gap-5">
              {/* Max concurrent jobs */}
              <div>
                <div className="flex items-center justify-between gap-3">
                  <label
                    htmlFor="ds-max-concurrent"
                    className="text-sm font-medium text-zinc-200"
                  >
                    {t('settings.maxConcurrent')}
                  </label>
                  <span className="rounded-full bg-accent-400/10 px-2.5 py-0.5 text-xs font-semibold text-accent-300">
                    {formatNumber(draft.maxConcurrent, lang)}
                  </span>
                </div>
                <input
                  id="ds-max-concurrent"
                  type="range"
                  min={1}
                  max={MAX_CONCURRENT_RANGE}
                  step={1}
                  value={draft.maxConcurrent}
                  onChange={(event) =>
                    setDraft((current) =>
                      current
                        ? { ...current, maxConcurrent: Number(event.target.value) }
                        : current,
                    )
                  }
                  className="mt-3 w-full cursor-pointer accent-accent-500"
                />
                <p className="mt-1.5 text-xs text-zinc-500">
                  {t('settings.maxConcurrentHint')}
                </p>
              </div>

              {/* Web preset codec */}
              <div>
                <p className="text-sm font-medium text-zinc-200">
                  {t('settings.webCodec')}
                </p>
                <div className="mt-2 grid grid-cols-2 gap-2">
                  {(Object.keys(CODEC_LABEL_KEYS) as WebCodec[]).map((codec) => {
                    const selected = draft.webCodec === codec
                    return (
                      <button
                        key={codec}
                        type="button"
                        aria-pressed={selected}
                        onClick={() =>
                          setDraft((current) =>
                            current ? { ...current, webCodec: codec } : current,
                          )
                        }
                        className={`inline-flex h-9 items-center justify-center rounded-xl border px-3 text-xs font-medium transition-colors ${
                          selected
                            ? 'border-accent-400/60 bg-accent-400/10 text-accent-200'
                            : 'border-white/[0.06] bg-ink-900 text-zinc-400 hover:border-white/15 hover:text-zinc-200'
                        }`}
                      >
                        {t(CODEC_LABEL_KEYS[codec])}
                      </button>
                    )
                  })}
                </div>
                <p className="mt-1.5 text-xs text-zinc-500">
                  {t('settings.webCodecHint')}
                </p>
              </div>

              {/* Output directory */}
              <div>
                <p className="text-sm font-medium text-zinc-200">
                  {t('settings.outputDir')}
                </p>
                <div className="mt-2 flex items-center gap-2">
                  <p
                    dir="ltr"
                    className="min-w-0 flex-1 truncate rounded-lg bg-ink-900 px-3 py-2 font-mono text-xs text-zinc-400"
                  >
                    {draft.outputDir ?? t('settings.outputDirAuto')}
                  </p>
                  <button
                    type="button"
                    onClick={() => void pickOutputDir()}
                    className="inline-flex h-9 shrink-0 items-center gap-1.5 rounded-lg bg-white/5 px-3 text-xs font-medium text-zinc-200 transition-colors hover:bg-white/10"
                  >
                    <FolderOpen className="h-3.5 w-3.5" aria-hidden="true" />
                    {t('settings.outputDirPick')}
                  </button>
                </div>
                <p className="mt-1.5 text-xs text-zinc-500">
                  {t('settings.outputDirHint')}
                </p>
              </div>

              {/* Strip metadata switch */}
              <div className="flex items-center justify-between gap-4">
                <div className="min-w-0">
                  <p className="text-sm font-medium text-zinc-200">
                    {t('settings.stripMetadata')}
                  </p>
                  <p className="mt-0.5 text-xs leading-relaxed text-zinc-500">
                    {t('settings.stripMetadataHint')}
                  </p>
                </div>
                <button
                  type="button"
                  role="switch"
                  aria-checked={draft.stripMetadata}
                  aria-label={t('settings.stripMetadata')}
                  onClick={() =>
                    setDraft((current) =>
                      current
                        ? { ...current, stripMetadata: !current.stripMetadata }
                        : current,
                    )
                  }
                  className={`relative h-6 w-10 shrink-0 rounded-full transition-colors ${
                    draft.stripMetadata ? 'bg-accent-500' : 'bg-white/10'
                  }`}
                >
                  <span
                    className={`absolute top-0.5 start-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform ${
                      draft.stripMetadata
                        ? 'translate-x-4 rtl:-translate-x-4'
                        : 'translate-x-0'
                    }`}
                  />
                </button>
              </div>
            </div>

            {/* Footer */}
            <div className="mt-6 flex items-center justify-end gap-2">
              <button
                type="button"
                onClick={() => setSettingsOpen(false)}
                className="rounded-xl px-4 py-2 text-sm text-zinc-400 transition-colors hover:text-zinc-200"
              >
                {t('settings.cancel')}
              </button>
              <button
                type="button"
                onClick={() => void save()}
                className="rounded-xl bg-accent-500 px-4 py-2 text-sm font-semibold text-ink-950 transition-all hover:bg-accent-400 active:scale-95"
              >
                {t('settings.save')}
              </button>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
