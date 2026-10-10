import { DropZone } from './components/DropZone'
import { Footer } from './components/Footer'
import { Header } from './components/Header'
import { PresetSelector } from './components/PresetSelector'
import { QueueList } from './components/QueueList'
import { SettingsDialog } from './components/SettingsDialog'
import { Toaster } from './components/Toaster'
import { useHotkeys } from './hooks/useHotkeys'
import { useTauriEvents } from './hooks/useTauriEvents'

export default function App() {
  useTauriEvents()
  useHotkeys()

  return (
    <div className="flex h-full flex-col overflow-hidden bg-ink-950">
      <Header />
      <main className="flex-1 overflow-y-auto px-4 py-6 sm:px-6">
        <div className="mx-auto flex w-full max-w-2xl flex-col gap-6">
          <PresetSelector />
          <DropZone />
          <QueueList />
        </div>
      </main>
      <Footer />
      <SettingsDialog />
      <Toaster />
    </div>
  )
}
