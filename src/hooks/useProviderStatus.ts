import { useMemo } from 'react'
import type { ProviderConfig, ProviderStatus } from '../types/providers'
import { useProviderConfig } from './useProviderConfig'
import { useClaudeWatcherStatus } from './useClaudeWatcher'
import { useCopilotWatcherStatus } from './useCopilotWatcher'
import { useOpenCodeWatcherStatus } from './useOpenCodeWatcher'
import { useCodexWatcherStatus } from './useCodexWatcher'
import { useGeminiWatcherStatus } from './useGeminiWatcher'

interface UseProviderStatusResult {
  status: ProviderStatus
  isLoading: boolean
  error: Error | null
  refetch: () => Promise<void>
}

/**
 * Calculate provider operational status from config data
 * Note: Status reflects the provider's CONFIGURED mode, not whether the watcher is currently running.
 * The watcher can be paused/stopped temporarily without changing the provider's operational mode.
 */
function calculateProviderStatus(
  isRunning: boolean | undefined,
  config: ProviderConfig | undefined
): ProviderStatus {
  // No config or explicitly disabled
  if (!config || !config.enabled) {
    return 'disabled'
  }

  // Provider is enabled - determine mode from syncMode
  switch (config.syncMode) {
    case 'Nothing':
      return 'local-only'
    case 'Metrics Only':
      return 'metrics-only'
    case 'Transcript and Metrics':
      return 'full-sync'
    default:
      console.warn(`Invalid syncMode: ${config.syncMode}. Defaulting to disabled.`)
      return 'disabled'
  }
}

/**
 * Hook to compute provider status from watcher and config data
 */
export function useProviderStatus(providerId: string): UseProviderStatusResult {
  // Get watcher status based on provider ID
  const claudeWatcher = useClaudeWatcherStatus()
  const copilotWatcher = useCopilotWatcherStatus()
  const opencodeWatcher = useOpenCodeWatcherStatus()
  const codexWatcher = useCodexWatcherStatus()
  const geminiWatcher = useGeminiWatcherStatus()

  // Select the appropriate watcher based on provider ID
  const watcherQuery = useMemo(() => {
    switch (providerId) {
      case 'claude-code':
        return claudeWatcher
      case 'github-copilot':
        return copilotWatcher
      case 'opencode':
        return opencodeWatcher
      case 'codex':
        return codexWatcher
      case 'gemini-code':
        return geminiWatcher
      default:
        return { data: undefined, isLoading: false, error: new Error(`Unknown provider: ${providerId}`), refetch: async () => {} }
    }
  }, [providerId, claudeWatcher, copilotWatcher, opencodeWatcher, codexWatcher, geminiWatcher])

  // Get provider config from React Query (single source of truth)
  const { data: config, isLoading: configLoading, error: configError, refetch: refetchConfig } = useProviderConfig(providerId)

  // Compute status
  const status = useMemo(() => {
    return calculateProviderStatus(watcherQuery.data?.is_running, config)
  }, [watcherQuery.data?.is_running, config])

  // isLoading true only during initial fetch of either watcher or config
  const isLoading = (watcherQuery.isLoading && !watcherQuery.data) || (configLoading && !config)

  // Refetch function
  const refetch = async () => {
    await refetchConfig()
    await watcherQuery.refetch()
  }

  return {
    status,
    isLoading,
    error: (watcherQuery.error || configError) as Error | null,
    refetch,
  }
}
