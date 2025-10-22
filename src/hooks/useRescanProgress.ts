import { listen } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

interface RescanProgressData {
  provider: string
  phase: string
  current: number
  total: number
  message: string
}

export function useRescanProgress(providerId: string) {
  const [progress, setProgress] = useState<RescanProgressData | null>(null)

  useEffect(() => {
    // Listen for rescan progress events
    const unlisten = listen<RescanProgressData>('rescan-progress', event => {
      // Only update if this event is for our provider
      if (event.payload.provider === providerId) {
        setProgress(event.payload)

        // Clear progress after completion (after 3 seconds)
        if (event.payload.phase === 'complete') {
          setTimeout(() => {
            setProgress(null)
          }, 3000)
        }
      }
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [providerId])

  const reset = () => setProgress(null)

  return { progress, reset }
}
