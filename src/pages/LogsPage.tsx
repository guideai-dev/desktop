import { useState } from 'react'
import { useProviderLogs } from '../hooks/useClaudeWatcher'

function LogsPage() {
  const [selectedProvider, setSelectedProvider] = useState('claude-code')
  const [maxLines, setMaxLines] = useState(100)

  const { data: logs = [], isLoading, error } = useProviderLogs(selectedProvider, maxLines)

  const getLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'text-red-400'
      case 'warn':
      case 'warning':
        return 'text-yellow-400'
      case 'info':
        return 'text-blue-400'
      case 'debug':
        return 'text-gray-500'
      default:
        return 'text-gray-300'
    }
  }

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp)
    return date.toISOString().replace('T', ' ').slice(0, 19)
  }

  const formatLevel = (level: string) => {
    return level.toUpperCase().padEnd(5)
  }

  return (
    <div className="p-4 space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-base-content">Logs</h1>
          <p className="text-sm text-base-content/70">View provider logs and system events</p>
        </div>
      </div>

      {/* Controls */}
      <div className="card bg-base-100 shadow-sm border border-base-300">
        <div className="card-body">
          <div className="flex flex-wrap gap-4 items-end">
            <div className="form-control">
              <label className="label">
                <span className="label-text">Provider</span>
              </label>
              <select
                className="select select-bordered"
                value={selectedProvider}
                onChange={(e) => setSelectedProvider(e.target.value)}
              >
                <option value="claude-code">Claude Code</option>
                <option value="opencode">OpenCode</option>
                <option value="codex">Codex</option>
              </select>
            </div>

            <div className="form-control">
              <label className="label">
                <span className="label-text">Max Lines</span>
              </label>
              <select
                className="select select-bordered"
                value={maxLines}
                onChange={(e) => setMaxLines(Number(e.target.value))}
              >
                <option value={50}>50</option>
                <option value={100}>100</option>
                <option value={200}>200</option>
                <option value={500}>500</option>
              </select>
            </div>

            <div className="form-control">
              <button
                className="btn btn-primary"
                onClick={() => window.location.reload()}
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                </svg>
                Refresh
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Logs Display */}
      <div className="card bg-base-100 shadow-sm border border-base-300">
        <div className="card-body">
          <div className="flex items-center gap-2 mb-4">
            <h2 className="text-lg font-semibold">
              {selectedProvider.charAt(0).toUpperCase() + selectedProvider.slice(1)} Logs
            </h2>
            {isLoading && <span className="loading loading-spinner loading-sm"></span>}
          </div>

          {error && (
            <div className="alert alert-error">
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span>Failed to load logs: {String(error)}</span>
            </div>
          )}

          {!error && (
            <div className="bg-black rounded-lg p-4 max-h-96 overflow-y-auto">
              {logs.length === 0 ? (
                <div className="text-center text-gray-400 py-8 font-mono">
                  {isLoading ? 'Loading logs...' : 'No logs available'}
                </div>
              ) : (
                <div className="font-mono text-xs leading-tight">
                  {logs.map((log, index) => (
                    <div key={index} className="hover:bg-gray-900 px-1 py-0.5 text-gray-200">
                      <span className="text-gray-400 inline-block w-48 shrink-0">
                        {formatTimestamp(log.timestamp)}
                      </span>
                      <span className={`inline-block w-12 shrink-0 font-medium ${getLevelColor(log.level)}`}>
                        {formatLevel(log.level)}
                      </span>
                      <span className="text-gray-300 inline-block w-24 shrink-0 truncate">
                        {log.provider}
                      </span>
                      <span className="text-gray-100">
                        {log.message}
                        {log.details && (
                          <span className="text-gray-500 ml-2">
                            {JSON.stringify(log.details)}
                          </span>
                        )}
                      </span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          )}

          {logs.length > 0 && (
            <div className="text-xs text-base-content/70 mt-2">
              Showing {logs.length} most recent log entries
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default LogsPage