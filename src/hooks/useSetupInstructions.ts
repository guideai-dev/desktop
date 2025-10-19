import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

/**
 * Hook to load provider setup instructions from markdown files
 * @param providerId - The provider ID (e.g., 'claude-code', 'gemini-code')
 * @param setupInstructionsFile - The markdown filename (e.g., 'claude-code.md')
 * @returns Query result with markdown content
 */
export function useSetupInstructions(providerId: string, setupInstructionsFile?: string) {
  return useQuery({
    queryKey: ['setup-instructions', providerId],
    queryFn: async () => {
      if (!setupInstructionsFile) {
        return null
      }

      try {
        const content = await invoke<string>('load_setup_instructions_command', {
          fileName: setupInstructionsFile,
        })
        return content
      } catch (error) {
        console.error(`Failed to load setup instructions for ${providerId}:`, error)
        return null
      }
    },
    enabled: !!setupInstructionsFile,
    staleTime: 1000 * 60 * 60, // 1 hour - instructions don't change often
    gcTime: 1000 * 60 * 60 * 24, // 24 hours
  })
}
