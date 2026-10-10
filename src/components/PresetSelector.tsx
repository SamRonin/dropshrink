import { Archive, Check, Globe, Send } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

import type { Preset } from '../types'
import { useI18n } from '../i18n'
import { useAppStore } from '../store/useAppStore'

interface PresetOption {
  id: Preset
  icon: LucideIcon
  titleKey: string
  subtitleKey: string
}

const PRESETS: PresetOption[] = [
  {
    id: 'telegram',
    icon: Send,
    titleKey: 'preset.telegram.title',
    subtitleKey: 'preset.telegram.subtitle',
  },
  {
    id: 'web',
    icon: Globe,
    titleKey: 'preset.web.title',
    subtitleKey: 'preset.web.subtitle',
  },
  {
    id: 'extreme',
    icon: Archive,
    titleKey: 'preset.extreme.title',
    subtitleKey: 'preset.extreme.subtitle',
  },
]

export function PresetSelector() {
  const { t } = useI18n()
  const preset = useAppStore((s) => s.preset)
  const setPreset = useAppStore((s) => s.setPreset)

  return (
    <section aria-label={t('preset.title')} className="flex flex-col gap-3">
      <h2 className="text-sm font-medium text-zinc-400">{t('preset.title')}</h2>
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
        {PRESETS.map((option) => {
          const selected = preset === option.id
          const Icon = option.icon
          return (
            <button
              key={option.id}
              type="button"
              onClick={() => setPreset(option.id)}
              aria-pressed={selected}
              className={`relative rounded-2xl border p-4 text-start transition-colors duration-150 ${
                selected
                  ? 'border-accent-400/60 bg-accent-400/10'
                  : 'border-white/[0.06] bg-ink-850 hover:border-white/15'
              }`}
            >
              {selected && (
                <span className="absolute end-3 top-3 flex h-5 w-5 items-center justify-center rounded-full bg-accent-400 text-ink-950">
                  <Check className="h-3 w-3" strokeWidth={3} aria-hidden="true" />
                </span>
              )}
              <span className="flex items-center gap-2">
                <Icon
                  className={`h-4 w-4 shrink-0 ${selected ? 'text-accent-300' : 'text-zinc-500'}`}
                  aria-hidden="true"
                />
                <span className="text-sm font-medium text-zinc-100">
                  {t(option.titleKey)}
                </span>
              </span>
              <span className="mt-1 block text-xs leading-relaxed text-zinc-500">
                {t(option.subtitleKey)}
              </span>
            </button>
          )
        })}
      </div>
    </section>
  )
}
