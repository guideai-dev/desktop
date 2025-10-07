import { renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAutoSessionProcessing } from './useAutoSessionProcessing'

const listen = vi.fn()
const invoke = vi.fn()
const processSession = vi.fn()

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: unknown[]) => listen(...args),
}))

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

vi.mock('./useSessionProcessing', () => ({
  useSessionProcessing: () => ({
    processSession: (...args: unknown[]) => processSession(...args),
  }),
}))

const createDeferred = () => {
  let resolve: (value?: unknown) => void
  const promise = new Promise(res => {
    resolve = res
  })
  return {
    promise,
    resolve: resolve!,
  }
}

describe('useAutoSessionProcessing', () => {
  let eventHandler: ((event: { payload: string }) => Promise<void>) | null = null
  let unlistenSpy: ReturnType<typeof vi.fn>

  beforeEach(() => {
    eventHandler = null
    unlistenSpy = vi.fn()
    listen.mockImplementation(async (_event, handler) => {
      eventHandler = handler as typeof eventHandler
      return () => {
        unlistenSpy()
      }
    })

    invoke.mockReset()
    processSession.mockReset()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  it('fetches session info, reads content, and processes metrics after completion event', async () => {
    const sessionRow = {
      provider: 'claude-code',
      file_path: '/tmp/session.jsonl',
      session_id: 'session-1',
    }

    invoke.mockImplementation(async (command, args) => {
      if (command === 'execute_sql') {
        expect(args).toEqual({
          sql: expect.stringContaining('FROM agent_sessions'),
          params: ['session-1'],
        })
        return [sessionRow]
      }

      if (command === 'get_session_content') {
        expect(args).toEqual({
          provider: 'claude-code',
          filePath: '/tmp/session.jsonl',
          sessionId: 'session-1',
        })
        return 'session-content'
      }

      throw new Error(`Unexpected command ${command}`)
    })

    processSession.mockResolvedValue(undefined)

    const { unmount } = renderHook(() => useAutoSessionProcessing())

    await waitFor(() => expect(listen).toHaveBeenCalledWith('session-completed', expect.any(Function)))
    expect(eventHandler).toBeTruthy()

    await eventHandler?.({ payload: 'session-1' })

    expect(processSession).toHaveBeenCalledWith('session-1', 'claude-code', 'session-content', 'local')

    unmount()
    await waitFor(() => expect(unlistenSpy).toHaveBeenCalled())
  })

  it('skips duplicate events while processing is in progress', async () => {
    const sessionRow = {
      provider: 'claude-code',
      file_path: '/tmp/session.jsonl',
      session_id: 'session-2',
    }

    invoke.mockImplementation(async (command) => {
      if (command === 'execute_sql') {
        return [sessionRow]
      }
      if (command === 'get_session_content') {
        return 'content-2'
      }
      throw new Error(`Unexpected command ${command}`)
    })

    const deferred = createDeferred()
    processSession.mockImplementation(() => deferred.promise)

    renderHook(() => useAutoSessionProcessing())
    await waitFor(() => expect(eventHandler).toBeTruthy())

    const handler = eventHandler!
    const processPromise = handler({ payload: 'session-2' })
    await waitFor(() => expect(processSession).toHaveBeenCalledTimes(1))

    await handler({ payload: 'session-2' })
    expect(processSession).toHaveBeenCalledTimes(1)

    deferred.resolve()
    await processPromise
  })

  it('logs when session is missing and does not invoke processor', async () => {
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    invoke.mockImplementation(async (command) => {
      if (command === 'execute_sql') {
        return []
      }
      throw new Error(`Unexpected command ${command}`)
    })

    renderHook(() => useAutoSessionProcessing())
    await waitFor(() => expect(eventHandler).toBeTruthy())

    await eventHandler?.({ payload: 'missing-session' })

    expect(processSession).not.toHaveBeenCalled()
    expect(consoleSpy).toHaveBeenCalledWith('Session missing-session not found in database')

    consoleSpy.mockRestore()
  })
})
