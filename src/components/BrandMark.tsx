import { useI18n } from '../i18n'

interface BrandMarkProps {
  /** Side length of the logo tile in px. */
  size?: number
}

/**
 * Inline SVG logo: rounded gradient tile (accent-400 → accent-600) with a
 * white droplet and a small chevron-down, plus the app name.
 */
export function BrandMark({ size = 28 }: BrandMarkProps) {
  const { t } = useI18n()

  return (
    <div className="flex shrink-0 items-center gap-2.5">
      <svg
        width={size}
        height={size}
        viewBox="0 0 24 24"
        role="img"
        aria-hidden="true"
        className="shrink-0 drop-shadow-[0_4px_12px_rgba(45,212,191,0.35)]"
      >
        <defs>
          <linearGradient id="ds-brand-grad" x1="0" y1="0" x2="1" y2="1">
            <stop offset="0%" stopColor="#2dd4bf" />
            <stop offset="100%" stopColor="#0d9488" />
          </linearGradient>
        </defs>
        <rect width="24" height="24" rx="7" fill="url(#ds-brand-grad)" />
        <path
          d="M12 3c3.5 4 6 6.8 6 10a6 6 0 1 1-12 0c0-3.2 2.5-6 6-10z"
          fill="#ffffff"
        />
        <path
          d="M9.5 12.5 12 15l2.5-2.5"
          fill="none"
          stroke="#0d9488"
          strokeWidth="1.8"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </svg>
      <span className="text-sm font-bold tracking-tight text-zinc-100">
        {t('app.name')}
      </span>
    </div>
  )
}
