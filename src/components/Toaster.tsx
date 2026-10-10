import { useEffect } from 'react'
import { AnimatePresence, motion } from 'framer-motion'
import { AlertTriangle, CheckCircle, Info } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

import type { ToastItem } from '../store/useAppStore'
import { useAppStore } from '../store/useAppStore'

const KIND_META: Record<ToastItem['kind'], { icon: LucideIcon; classes: string }> = {
  success: { icon: CheckCircle, classes: 'text-emerald-300' },
  error: { icon: AlertTriangle, classes: 'text-rose-300' },
  info: { icon: Info, classes: 'text-accent-300' },
}

const AUTO_DISMISS_MS = 3500

function ToastRow({ item }: { item: ToastItem }) {
  const dismissToast = useAppStore((s) => s.dismissToast)
  const meta = KIND_META[item.kind]
  const Icon = meta.icon

  useEffect(() => {
    const timer = window.setTimeout(() => dismissToast(item.id), AUTO_DISMISS_MS)
    return () => window.clearTimeout(timer)
  }, [dismissToast, item.id])

  return (
    <motion.div
      layout
      initial={{ opacity: 0, y: -12, scale: 0.97 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: -8, scale: 0.97 }}
      transition={{ duration: 0.18, ease: 'easeOut' }}
      className="pointer-events-auto flex items-center gap-2 rounded-xl border border-white/10 bg-ink-800/95 px-4 py-2.5 text-sm text-zinc-200 shadow-card backdrop-blur"
    >
      <Icon className={`h-4 w-4 shrink-0 ${meta.classes}`} aria-hidden="true" />
      <span dir="auto">{item.message}</span>
    </motion.div>
  )
}

export function Toaster() {
  const toasts = useAppStore((s) => s.toasts)

  return (
    <div className="pointer-events-none fixed inset-x-0 top-4 z-[60] flex flex-col items-center gap-2 px-4">
      <AnimatePresence initial={false}>
        {toasts.map((item) => (
          <ToastRow key={item.id} item={item} />
        ))}
      </AnimatePresence>
    </div>
  )
}
