import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface ClaudeWatcherStatus {
  is_running: boolean
  pending_uploads: number
  processing_uploads: number
  failed_uploads: number
}


export interface LogEntry {
  timestamp: string
  level: string
  provider: string
  message: string
  details?: any
}

export function useClaudeWatcherStatus() {
  return useQuery({
    queryKey: ['claude-watcher-status'],
    queryFn: () => invoke<ClaudeWatcherStatus>('get_claude_watcher_status'),
    refetchInterval: 2000, // Poll every 2 seconds
  })
}


export function useStartClaudeWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (projects: string[]) =>
      invoke<void>('start_claude_watcher', { projects }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['claude-watcher-status'] })
    },
  })
}

export function useStopClaudeWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => invoke<void>('stop_claude_watcher'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['claude-watcher-status'] })
    },
  })
}


export function useProviderLogs(provider: string, maxLines?: number) {
  return useQuery({
    queryKey: ['provider-logs', provider, maxLines],
    queryFn: () => invoke<LogEntry[]>('get_provider_logs', {
      provider,
      maxLines
    }),
    refetchInterval: 5000, // Poll every 5 seconds
    enabled: !!provider,
  })
}