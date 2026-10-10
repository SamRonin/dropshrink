import { Settings } from 'lucide-react'

import { BrandMark } from './BrandMark'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

export function Header() {
  const { t, lang, setLang } = useI18n()
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen)

  const langButton = (active: boolean) =>
    `inline-flex h-9 items-center justify-center rounded-lg px-3 text-xs font-semibold transition-colors ${
      active ? 'bg-accent-500/15 text-accent-300' : 'text-zinc-400 hover:text-zinc-200'
    }`

  return (
    <header className="sticky top-0 z-40 flex h-14 shrink-0 items-center justify-between gap-4 border-b border-white/5 bg-ink-950/80 px-4 backdrop-blur-md sm:px-6">
      <div className="flex min-w-0 items-center gap-3">
        <BrandMark />
        <span className="hidden truncate text-xs text-zinc-500 sm:inline">
          {t('app.tagline')}
        </span>
      </div>

      <div className="flex shrink-0 items-center gap-2">
        <div
          role="group"
          aria-label={t('header.language')}
          className="flex items-center rounded-xl border border-white/[0.06] bg-ink-850 p-1"
        >
          <button
            type="button"
            onClick={() => setLang('fa')}
            aria-pressed={lang === 'fa'}
            className={langButton(lang === 'fa')}
          >
            فا
          </button>
          <button
            type="button"
            onClick={() => setLang('en')}
            aria-pressed={lang === 'en'}
            className={langButton(lang === 'en')}
          >
            EN
          </button>
        </div>

        <button
          type="button"
          className="icon-btn"
          aria-label={t('header.settings')}
          onClick={() => setSettingsOpen(true)}
        >
          <Settings className="h-[18px] w-[18px]" />
        </button>
      </div>
    </header>
  )
}
