import type { Lang } from '../i18n'

/** Locale-aware number formatting (Persian digits for fa). */
export function formatNumber(value: number, lang: Lang): string {
  return value.toLocaleString(lang === 'fa' ? 'fa-IR' : 'en-US')
}

export function formatBytes(
  bytes: number | null | undefined,
  lang: Lang,
): string {
  if (bytes == null || Number.isNaN(bytes)) return '—'
  const units = lang === 'fa'
    ? ['بایت', 'کیلوبایت', 'مگابایت', 'گیگابایت']
    : ['B', 'KB', 'MB', 'GB']
  let value = bytes
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  const digits = value >= 100 || unit === 0 ? 0 : value >= 10 ? 1 : 2
  const formatted = value.toLocaleString(lang === 'fa' ? 'fa-IR' : 'en-US', {
    maximumFractionDigits: digits,
    minimumFractionDigits: 0,
  })
  return lang === 'fa' ? `${formatted} ${units[unit]}` : `${formatted} ${units[unit]}`
}

function twoDigits(n: number, lang: Lang): string {
  return formatNumber(Math.floor(n), lang).padStart(2, lang === 'fa' ? '۰' : '0')
}

/** 125 → "2:05" (or Persian digits). Over an hour → "1:02:05". */
export function formatDuration(totalSeconds: number, lang: Lang): string {
  const secs = Math.max(0, Math.round(totalSeconds))
  const h = Math.floor(secs / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = secs % 60
  if (h > 0) return `${twoDigits(h, lang)}:${twoDigits(m, lang)}:${twoDigits(s, lang)}`
  return `${twoDigits(m, lang)}:${twoDigits(s, lang)}`
}

export function formatEta(etaSecs: number | null, lang: Lang): string {
  if (etaSecs == null || !Number.isFinite(etaSecs)) return '—'
  return formatDuration(etaSecs, lang)
}

export function formatPercent(p: number, lang: Lang): string {
  const clamped = Math.max(0, Math.min(100, Math.round(p)))
  return formatNumber(clamped, lang)
}

/** Percentage of size saved, 0..100 (rounded). */
export function reductionPercent(input: number, output: number): number {
  if (!input || output == null || output >= input) return 0
  return Math.round(((input - output) / input) * 100)
}

/** Extract a human-safe file name from a full path (both separators). */
export function baseName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path
}
