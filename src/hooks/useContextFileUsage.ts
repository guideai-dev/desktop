import { useMemo } from 'react'

interface ContextFile {
  fileName: string
  filePath: string
  relativePath: string
  content: string
  size: number
}

export interface FileUsageStats {
  toolCalls: number
  messages: number
}

/**
 * Hook to track usage of context files within a session transcript.
 *
 * Scans the raw transcript file content for Read tool calls that accessed
 * each context file. Only counts actual Read tool invocations, not string
 * mentions.
 *
 * @param fileContent - Raw session transcript (JSONL format)
 * @param contextFiles - List of context files found in the project
 * @returns Map of relativePath -> FileUsageStats
 */
export function useContextFileUsage(
  fileContent: string | null,
  contextFiles: ContextFile[]
): Map<string, FileUsageStats> {
  return useMemo(() => {
    const usageCounts = new Map<string, FileUsageStats>()

    // Return empty map if no content to scan
    if (!fileContent || contextFiles.length === 0) {
      return usageCounts
    }

    // Build a map of context file paths for fast lookup
    // Include both absolute and relative paths, normalized
    const contextPathMap = new Map<string, string>() // normalized path -> relativePath

    for (const contextFile of contextFiles) {
      const { relativePath, filePath } = contextFile

      // Normalize paths to forward slashes for comparison
      const normalizedRelative = relativePath.replace(/\\/g, '/')
      const normalizedAbsolute = filePath.replace(/\\/g, '/')

      contextPathMap.set(normalizedRelative, relativePath)
      contextPathMap.set(normalizedAbsolute, relativePath)
    }

    // Parse JSONL content into individual messages
    const lines = fileContent.split('\n').filter(line => line.trim())

    // Scan all messages for Read tool calls
    for (const line of lines) {
      try {
        const message = JSON.parse(line)

        // Extract Read tool calls from this message
        const readToolCalls = extractReadToolCalls(message)

        // Check if any Read tool call accessed a context file
        for (const filePath of readToolCalls) {
          const normalizedPath = filePath.replace(/\\/g, '/')
          const matchedRelativePath = contextPathMap.get(normalizedPath)

          if (matchedRelativePath) {
            // Increment tool call count for this context file
            const existing = usageCounts.get(matchedRelativePath) || { toolCalls: 0, messages: 0 }
            usageCounts.set(matchedRelativePath, {
              toolCalls: existing.toolCalls + 1,
              messages: existing.messages,
            })
          }
        }
      } catch (_err) {
        // Skip malformed lines
      }
    }

    return usageCounts
  }, [fileContent, contextFiles])
}

/**
 * Extract file paths from Read tool calls in a message.
 * Returns an array of file paths that were read.
 */
function extractReadToolCalls(message: any): string[] {
  const filePaths: string[] = []

  // Check if this is a tool use message with Read tool
  if (message.message?.content && Array.isArray(message.message.content)) {
    for (const block of message.message.content) {
      if (block.type === 'tool_use' && block.name === 'Read' && block.input?.file_path) {
        filePaths.push(block.input.file_path)
      }
    }
  }

  // Check for user type messages with toolUseResult (Read tool results)
  if (message.type === 'user' && message.toolUseResult) {
    // Read tool results have a 'content' field with file contents
    // We need to check the corresponding tool use message for the file path
    // For now, we'll skip tool results since the tool use is more reliable
  }

  return filePaths
}
