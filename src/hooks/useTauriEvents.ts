import { useEffect, useRef } from 'react'

import { api, listenJobUpdate } from '../lib/tauri'
import { useAppStore } from '../store/useAppStore'
import { useI18n } from '../i18n'

/**
 * Single bridge between the Tauri backend and the zustand store.
 * On mount: load persisted settings, detect FFmpeg, then keep the queue
 * in sync via `job-update` events. All handlers are stable — listeners
 * are registered exactly once and never resubscribe on re-render.
 */
export function useTauriEvents(): void {
  const { t } = useI18n()

  // Keep a stable translator reference so the effect below can stay dep-free.
  const tRef = useRef(t)
  useEffect(() => {
    tRef.current = t
  }, [t])

  useEffect(() => {
    let active = true
    // Store actions are created once inside `create()` — grabbing them via
    // getState() keeps this effect dependency-free and the subscription stable.
    const { setSettings, setFfmpeg, applyUpdate, toast } = useAppStore.getState()

    api
      .getSettings()
      .then((settings) => {
        if (active) setSettings(settings)
      })
      .catch((err: unknown) => {
        console.error('[dropshrink] get_settings failed:', err)
      })

    api
      .detectFfmpeg()
      .then((status) => {
        if (active) setFfmpeg(status)
      })
      .catch((err: unknown) => {
        console.error('[dropshrink] detect_ffmpeg failed:', err)
        if (active) toast('error', tRef.current('ffmpeg.missing.toast'))
      })

    const unlistenPromise = listenJobUpdate((update) => {
      applyUpdate(update)
    })

    return () => {
      active = false
      unlistenPromise
        .then((unlisten) => unlisten())
        .catch(() => undefined)
    }
  }, [])
}
