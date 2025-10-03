import { invoke } from '@tauri-apps/api/core'
import { useEffect, useState } from 'react'

export interface LocalProject {
  id: string
  name: string
  githubRepo: string | null
  cwd: string
  type: string
  createdAt: number
  updatedAt: number
  sessionCount: number
}

/**
 * Fetch all projects from the local database
 */
export function useLocalProjects() {
  const [projects, setProjects] = useState<LocalProject[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = async () => {
    try {
      setLoading(true)
      setError(null)
      const result = await invoke<LocalProject[]>('get_all_projects')
      setProjects(result)
    } catch (err) {
      console.error('Failed to fetch projects:', err)
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    refresh()
  }, [])

  return { projects, loading, error, refresh }
}

/**
 * Fetch a single project by ID
 */
export function useLocalProject(projectId: string | undefined) {
  const [project, setProject] = useState<LocalProject | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    if (!projectId) {
      setLoading(false)
      return
    }

    const fetchProject = async () => {
      try {
        setLoading(true)
        setError(null)
        const result = await invoke<LocalProject | null>('get_project_by_id', { projectId })
        setProject(result)
      } catch (err) {
        console.error('Failed to fetch project:', err)
        setError(err instanceof Error ? err.message : String(err))
      } finally {
        setLoading(false)
      }
    }

    fetchProject()
  }, [projectId])

  return { project, loading, error }
}
