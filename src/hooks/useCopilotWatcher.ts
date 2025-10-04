import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface CopilotWatcherStatus {
  is_running: boolean
  pending_uploads: number
  processing_uploads: number
  failed_uploads: number
}

export function useCopilotWatcherStatus() {
  return useQuery({
    queryKey: ['copilot-watcher-status'],
    queryFn: () => invoke<CopilotWatcherStatus>('get_copilot_watcher_status'),
    refetchInterval: 2000, // Poll every 2 seconds
  })
}

export function useStartCopilotWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (projects: string[]) =>
      invoke<void>('start_copilot_watcher', { projects }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['copilot-watcher-status'] })
    },
  })
}

export function useStopCopilotWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => invoke<void>('stop_copilot_watcher'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['copilot-watcher-status'] })
    },
  })
}
