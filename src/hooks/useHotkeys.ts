import { useEffect } from 'react'

import { useAppStore } from '../store/useAppStore'
import { useI18n } from '../i18n'

/** Window event name DropZone listens for to open the native file picker. */
export const PICK_FILES_EVENT = 'dropshrink:pick-files'

/**
 * Global keyboard shortcuts (all combos preventDefault):
 * - Ctrl/Cmd + O → open the native file picker (via `PICK_FILES_EVENT`)
 * - Ctrl/Cmd + , → open the settings dialog
 * - Ctrl/Cmd + L → toggle UI language (fa <-> en)
 */
export function useHotkeys(): void {
  const { lang, setLang } = useI18n()

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (!(event.ctrlKey || event.metaKey)) return

      const key = event.key.toLowerCase()
      if (key === 'o') {
        event.preventDefault()
        window.dispatchEvent(new CustomEvent(PICK_FILES_EVENT))
      } else if (event.key === ',') {
        event.preventDefault()
        useAppStore.getState().setSettingsOpen(true)
      } else if (key === 'l') {
        event.preventDefault()
        setLang(lang === 'fa' ? 'en' : 'fa')
      }
    }

    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [lang, setLang])
}
