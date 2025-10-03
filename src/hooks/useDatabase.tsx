import { useEffect, useState } from 'react'
import { initializeDatabase } from '../db/client'

/**
 * Database initialization hook
 * Initializes the SQLite database via tauri-plugin-sql
 */
export function useDatabase() {
  const [isReady, setIsReady] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  useEffect(() => {
    let mounted = true

    const init = async () => {
      try {
        await initializeDatabase()
        if (mounted) {
          setIsReady(true)
          console.log('✓ Database ready')
        }
      } catch (err) {
        console.error('Failed to initialize database:', err)
        if (mounted) {
          setError(err as Error)
        }
      }
    }

    init()

    return () => {
      mounted = false
    }
  }, [])

  return { isReady, error }
}
