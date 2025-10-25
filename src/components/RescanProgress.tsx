interface RescanProgressProps {
  phase: string
  current: number
  total: number
  message: string
  onCancel?: () => void
}

function RescanProgress({ phase, current, total, message, onCancel }: RescanProgressProps) {
  const percentage = total > 0 ? Math.round((current / total) * 100) : 0
  const isComplete = phase === 'complete'
  const isScanning = phase === 'scanning'

  return (
    <div className="bg-base-200 rounded-lg p-4 space-y-3">
      {/* Status indicator */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {!isComplete ? (
            <span className="loading loading-spinner loading-sm text-primary" />
          ) : (
            <svg
              className="w-5 h-5 text-success"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          )}
          <span className="text-sm font-medium">
            {isComplete
              ? 'Scan Complete'
              : isScanning
                ? 'Scanning Directory...'
                : 'Processing Sessions...'}
          </span>
        </div>
        {onCancel && !isComplete && (
          <button onClick={onCancel} className="btn btn-ghost btn-xs">
            Cancel
          </button>
        )}
      </div>

      {/* Progress bar */}
      {!isScanning && total > 0 && (
        <>
          <div className="flex justify-between text-xs text-base-content/70">
            <span>
              {current} / {total} sessions
            </span>
            <span>{percentage}%</span>
          </div>
          <progress className="progress progress-primary w-full" value={current} max={total} />
        </>
      )}

      {/* Status message */}
      <div className="text-xs text-base-content/60">{message}</div>
    </div>
  )
}

export default RescanProgress
