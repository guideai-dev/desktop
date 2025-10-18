import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface GeminiWatcherStatus {
  is_running: boolean
  pending_uploads: number
  processing_uploads: number
  failed_uploads: number
}

export function useGeminiWatcherStatus() {
  return useQuery({
    queryKey: ['gemini-watcher-status'],
    queryFn: () => invoke<GeminiWatcherStatus>('get_gemini_watcher_status'),
    refetchInterval: 2000, // Poll every 2 seconds
  })
}

export function useStartGeminiWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (projects: string[]) => invoke<void>('start_gemini_watcher', { projects }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['gemini-watcher-status'] })
    },
  })
}

export function useStopGeminiWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => invoke<void>('stop_gemini_watcher'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['gemini-watcher-status'] })
    },
  })
}
