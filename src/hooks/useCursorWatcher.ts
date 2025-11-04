import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export interface CursorWatcherStatus {
  is_running: boolean
  pending_uploads: number
  processing_uploads: number
  failed_uploads: number
}

export function useCursorWatcherStatus() {
  return useQuery({
    queryKey: ['cursor-watcher-status'],
    queryFn: () => invoke<CursorWatcherStatus>('get_cursor_watcher_status'),
    refetchInterval: 2000, // Poll every 2 seconds
  })
}

export function useStartCursorWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (projects: string[]) => invoke<void>('start_cursor_watcher', { projects }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['cursor-watcher-status'] })
    },
  })
}

export function useStopCursorWatcher() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: () => invoke<void>('stop_cursor_watcher'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['cursor-watcher-status'] })
    },
  })
}
