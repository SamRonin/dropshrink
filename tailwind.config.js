/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{ts,tsx}'],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        // Theme-driven colours: the RGB values live in src/index.css, so the
        // light theme only has to redefine variables.
        ink: {
          950: 'rgb(var(--ink-950) / <alpha-value>)',
          900: 'rgb(var(--ink-900) / <alpha-value>)',
          850: 'rgb(var(--ink-850) / <alpha-value>)',
          800: 'rgb(var(--ink-800) / <alpha-value>)',
          700: 'rgb(var(--ink-700) / <alpha-value>)',
          600: 'rgb(var(--ink-600) / <alpha-value>)',
        },
        // Text greys: the 950 shade is not themed on purpose (it is used as the
        // text colour on bright accent buttons, which stays dark in both themes).
        zinc: {
          100: 'rgb(var(--zinc-100) / <alpha-value>)',
          200: 'rgb(var(--zinc-200) / <alpha-value>)',
          300: 'rgb(var(--zinc-300) / <alpha-value>)',
          400: 'rgb(var(--zinc-400) / <alpha-value>)',
          500: 'rgb(var(--zinc-500) / <alpha-value>)',
          600: 'rgb(var(--zinc-600) / <alpha-value>)',
        },
        // Status text colours: light tints on dark, deeper shades on light.
        // Only the 300 shade is themed; the other shades keep Tailwind's values.
        emerald: { 300: 'rgb(var(--emerald-300) / <alpha-value>)' },
        amber: { 300: 'rgb(var(--amber-300) / <alpha-value>)' },
        rose: { 300: 'rgb(var(--rose-300) / <alpha-value>)' },
        // Translucent white in dark mode, translucent black in light mode.
        overlay: 'rgb(var(--overlay) / <alpha-value>)',
        accent: {
          200: 'rgb(var(--accent-200) / <alpha-value>)',
          300: 'rgb(var(--accent-300) / <alpha-value>)',
          400: '#2dd4bf',
          500: '#14b8a6',
          600: '#0d9488',
          700: '#0f766e',
        },
      },
      fontFamily: {
        sans: ['var(--font-main)', 'ui-sans-serif', 'system-ui', 'sans-serif'],
      },
      boxShadow: {
        glow: '0 0 44px -10px rgba(45, 212, 191, 0.45)',
        card: '0 1px 0 0 rgba(255,255,255,0.04) inset, 0 8px 24px -16px rgba(0,0,0,0.7)',
      },
      animation: {
        'shimmer': 'shimmer 1.6s linear infinite',
      },
      keyframes: {
        shimmer: {
          '0%': { transform: 'translateX(-100%)' },
          '100%': { transform: 'translateX(100%)' },
        },
      },
    },
  },
  plugins: [],
}
