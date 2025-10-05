import { useState, useCallback, useEffect } from 'react'
import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'

interface UpdaterState {
  isChecking: boolean
  isDownloading: boolean
  isInstalling: boolean
  hasUpdate: boolean
  currentVersion: string | null
  latestVersion: string | null
  error: string | null
  downloadProgress: number
}

interface UseUpdaterReturn extends UpdaterState {
  checkForUpdates: () => Promise<void>
  downloadAndInstall: () => Promise<void>
  isUpToDate: boolean
}

/**
 * Hook to manage app updates using Tauri's built-in updater
 */
export function useUpdater(): UseUpdaterReturn {
  const [state, setState] = useState<UpdaterState>({
    isChecking: false,
    isDownloading: false,
    isInstalling: false,
    hasUpdate: false,
    currentVersion: null,
    latestVersion: null,
    error: null,
    downloadProgress: 0,
  })

  const [updateInstance, setUpdateInstance] = useState<Update | null>(null)
  const [isUpToDate, setIsUpToDate] = useState(false)

  // Load current app version on mount
  useEffect(() => {
    getVersion().then(version => {
      setState(prev => ({ ...prev, currentVersion: version }))
    }).catch(err => {
      console.error('Failed to get app version:', err)
    })
  }, [])

  const checkForUpdates = useCallback(async () => {
    setState(prev => ({ ...prev, isChecking: true, error: null }))

    try {
      // Log the expected endpoint URL for debugging
      const platform = navigator.platform.toLowerCase()
      const arch = navigator.userAgent.includes('x64') || navigator.userAgent.includes('x86_64')
        ? 'x86_64'
        : navigator.userAgent.includes('aarch64') || navigator.userAgent.includes('arm64')
        ? 'aarch64'
        : 'x86_64'

      let target = 'unknown'
      if (platform.includes('mac') || platform.includes('darwin')) {
        target = 'darwin-universal'
      } else if (platform.includes('linux')) {
        target = `linux-${arch}`
      } else if (platform.includes('win')) {
        target = `windows-${arch}`
      }

      const expectedUrl = `https://install.guideai.dev/desktop/${target}/latest.json`
      console.log('Checking for updates at:', expectedUrl)

      const update = await check()

      if (update) {
        console.log('Update available:', {
          currentVersion: update.currentVersion,
          latestVersion: update.version,
        })
        setState(prev => ({
          ...prev,
          isChecking: false,
          hasUpdate: true,
          currentVersion: update.currentVersion,
          latestVersion: update.version,
        }))
        setUpdateInstance(update)
      } else {
        console.log('No update available - app is up to date')
        setIsUpToDate(true)
        setState(prev => ({
          ...prev,
          isChecking: false,
          hasUpdate: false,
          error: null,
        }))
        // Reset isUpToDate after 3 seconds
        setTimeout(() => setIsUpToDate(false), 3000)
      }
    } catch (error) {
      console.error('Failed to check for updates:', error)
      console.error('Error details:', {
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      })
      setState(prev => ({
        ...prev,
        isChecking: false,
        error: error instanceof Error ? error.message : 'Failed to check for updates',
      }))
    }
  }, [])

  const downloadAndInstall = useCallback(async () => {
    if (!updateInstance) {
      setState(prev => ({ ...prev, error: 'No update available' }))
      return
    }

    setState(prev => ({ ...prev, isDownloading: true, error: null, downloadProgress: 0 }))

    try {
      // Download the update with progress tracking
      let totalDownloaded = 0
      await updateInstance.downloadAndInstall((event) => {
        switch (event.event) {
          case 'Started':
            setState(prev => ({ ...prev, isDownloading: true }))
            break
          case 'Progress':
            // Track chunks downloaded (we don't have total size, so show indeterminate progress)
            totalDownloaded += event.data.chunkLength
            setState(prev => ({
              ...prev,
              downloadProgress: 50, // Indeterminate progress
            }))
            break
          case 'Finished':
            setState(prev => ({
              ...prev,
              isDownloading: false,
              isInstalling: true,
              downloadProgress: 100,
            }))
            break
        }
      })

      // Update installed successfully, relaunch the app
      setState(prev => ({ ...prev, isInstalling: false }))
      await relaunch()
    } catch (error) {
      console.error('Failed to download and install update:', error)
      setState(prev => ({
        ...prev,
        isDownloading: false,
        isInstalling: false,
        error: error instanceof Error ? error.message : 'Failed to download and install update',
      }))
    }
  }, [updateInstance])

  return {
    ...state,
    checkForUpdates,
    downloadAndInstall,
    isUpToDate,
  }
}
