import { create } from 'zustand'

/**
 * Session Activity Store
 *
 * Tracks active sessions based on file watcher events from Tauri.
 * A session is considered "active" if it has received activity in the past 2 minutes.
 */

interface ActiveSession {
  sessionId: string
  lastActivityTime: number
}

interface SessionActivityState {
  activeSessions: Map<string, ActiveSession>
  isTrackingEnabled: boolean // Global flag to disable tracking during rescans
  config: {
    activeSessionTimeout: number // milliseconds to consider a session "active"
  }
}

interface SessionActivityActions {
  markSessionActive: (sessionId: string) => void
  isSessionActive: (sessionId: string) => boolean
  cleanupInactiveSessions: () => void
  clearAllActiveSessions: () => void
  setTrackingEnabled: (enabled: boolean) => void
  updateConfig: (config: Partial<SessionActivityState['config']>) => void
}

type SessionActivityStore = SessionActivityState & SessionActivityActions

const initialState: SessionActivityState = {
  activeSessions: new Map(),
  isTrackingEnabled: true,
  config: {
    activeSessionTimeout: 2 * 60 * 1000, // 2 minutes
  },
}

export const useSessionActivityStore = create<SessionActivityStore>()((set, get) => ({
  ...initialState,

  markSessionActive: (sessionId: string) => {
    const { isTrackingEnabled } = get()
    if (!isTrackingEnabled) {
      // Tracking is disabled, ignore this event
      return
    }

    set(state => {
      const newActiveSessions = new Map(state.activeSessions)
      newActiveSessions.set(sessionId, {
        sessionId,
        lastActivityTime: Date.now(),
      })
      return { activeSessions: newActiveSessions }
    })
  },

  isSessionActive: (sessionId: string) => {
    const { activeSessions, config } = get()
    const session = activeSessions.get(sessionId)
    if (!session) return false

    const timeSinceActivity = Date.now() - session.lastActivityTime
    return timeSinceActivity < config.activeSessionTimeout
  },

  cleanupInactiveSessions: () => {
    const { activeSessions, config } = get()
    const newActiveSessions = new Map(activeSessions)
    const now = Date.now()

    for (const [sessionId, session] of newActiveSessions.entries()) {
      if (now - session.lastActivityTime >= config.activeSessionTimeout) {
        newActiveSessions.delete(sessionId)
      }
    }

    if (newActiveSessions.size !== activeSessions.size) {
      set({ activeSessions: newActiveSessions })
    }
  },

  clearAllActiveSessions: () => {
    set({ activeSessions: new Map() })
  },

  setTrackingEnabled: (enabled: boolean) => {
    set({ isTrackingEnabled: enabled })
  },

  updateConfig: config => {
    set(state => ({
      config: { ...state.config, ...config },
    }))
  },
}))
