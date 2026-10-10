import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'

import fa from './fa.json'
import en from './en.json'

export type Lang = 'fa' | 'en'

const LS_KEY = 'dropshrink.lang'

const dictionaries: Record<Lang, Record<string, string>> = {
  fa: fa as Record<string, string>,
  en: en as Record<string, string>,
}

interface I18nContextValue {
  lang: Lang
  dir: 'rtl' | 'ltr'
  t: (key: string, params?: Record<string, string | number>) => string
  setLang: (lang: Lang) => void
}

const I18nContext = createContext<I18nContextValue | null>(null)

function detectInitialLang(): Lang {
  try {
    const saved = localStorage.getItem(LS_KEY)
    if (saved === 'fa' || saved === 'en') return saved
  } catch {
    /* ignore */
  }
  const nav = typeof navigator !== 'undefined' ? navigator.language : 'en'
  return nav?.toLowerCase().startsWith('fa') ? 'fa' : 'en'
}

export function I18nProvider({ children }: { children: ReactNode }) {
  const [lang, setLangState] = useState<Lang>(detectInitialLang)

  useEffect(() => {
    try {
      localStorage.setItem(LS_KEY, lang)
    } catch {
      /* ignore */
    }
    document.documentElement.lang = lang
    document.documentElement.dir = lang === 'fa' ? 'rtl' : 'ltr'
  }, [lang])

  const setLang = useCallback((l: Lang) => setLangState(l), [])

  const t = useCallback(
    (key: string, params?: Record<string, string | number>) => {
      let str = dictionaries[lang][key] ?? dictionaries.en[key] ?? key
      if (params) {
        for (const [k, v] of Object.entries(params)) {
          str = str.replaceAll(`{${k}}`, String(v))
        }
      }
      return str
    },
    [lang],
  )

  const dir = lang === 'fa' ? 'rtl' : 'ltr'
  const value = useMemo<I18nContextValue>(
    () => ({ lang, dir, t, setLang }),
    [lang, dir, t, setLang],
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used inside <I18nProvider>')
  return ctx
}
