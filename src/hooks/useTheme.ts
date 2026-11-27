import { useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

import type { ThemeMode } from '../types'
import { useAppStore } from '../store/useAppStore'

const LIGHT_QUERY = '(prefers-color-scheme: light)'

type ResolvedTheme = 'dark' | 'light'

/** Keeps the native title bar in step with the app. Outside Tauri (e.g. a
 *  plain browser during development) there is no window to update. */
async function setWindowTheme(theme: ResolvedTheme): Promise<void> {
  try {
    await getCurrentWindow().setTheme(theme)
  } catch {
    /* not running inside Tauri, or the window API is unavailable */
  }
}

/**
 * Applies the chosen theme to the document: `data-theme` on <html> (which the
 * CSS variables in index.css read), the browser colour-scheme, and the native
 * window theme. `system` follows the OS and updates when it changes.
 */
export function useTheme(): void {
  const mode: ThemeMode = useAppStore((s) => s.settings?.theme ?? 'dark')

  useEffect(() => {
    const root = document.documentElement
    const query = window.matchMedia(LIGHT_QUERY)

    const apply = () => {
      const resolved: ResolvedTheme =
        mode === 'system' ? (query.matches ? 'light' : 'dark') : mode
      root.dataset.theme = resolved
      root.style.colorScheme = resolved
      void setWindowTheme(resolved)
    }

    apply()
    if (mode !== 'system') return
    query.addEventListener('change', apply)
    return () => query.removeEventListener('change', apply)
  }, [mode])
}
