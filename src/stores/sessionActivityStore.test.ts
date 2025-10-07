import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { useSessionActivityStore } from './sessionActivityStore'

const DEFAULT_TIMEOUT = 2 * 60 * 1000

const resetStore = () => {
  useSessionActivityStore.setState({
    activeSessions: new Map(),
    isTrackingEnabled: true,
    config: {
      activeSessionTimeout: DEFAULT_TIMEOUT,
    },
  })
}

describe('useSessionActivityStore', () => {
  beforeEach(() => {
    resetStore()
  })

  afterEach(() => {
    vi.useRealTimers()
    resetStore()
  })

  it('marks a session active when tracking is enabled', () => {
    const { markSessionActive, isSessionActive } = useSessionActivityStore.getState()

    markSessionActive('session-1')

    expect(isSessionActive('session-1')).toBe(true)
  })

  it('ignores session activity when tracking is disabled', () => {
    const { setTrackingEnabled, markSessionActive, isSessionActive } =
      useSessionActivityStore.getState()

    setTrackingEnabled(false)
    markSessionActive('session-1')

    expect(isSessionActive('session-1')).toBe(false)
    expect(useSessionActivityStore.getState().activeSessions.size).toBe(0)
  })

  it('removes inactive sessions after the timeout', () => {
    vi.useFakeTimers()

    const store = useSessionActivityStore.getState()

    store.updateConfig({ activeSessionTimeout: 1_000 })

    store.markSessionActive('session-1')
    expect(store.isSessionActive('session-1')).toBe(true)

    vi.advanceTimersByTime(1_000)

    store.cleanupInactiveSessions()
    expect(useSessionActivityStore.getState().isSessionActive('session-1')).toBe(false)
  })
})

