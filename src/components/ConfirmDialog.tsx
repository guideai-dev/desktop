interface ConfirmDialogProps {
  isOpen: boolean
  title: string
  message: string
  confirmText?: string
  cancelText?: string
  onConfirm: () => void
  onCancel: () => void
  variant?: 'info' | 'warning' | 'error' | 'success'
}

export default function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmText = 'Confirm',
  cancelText = 'Cancel',
  onConfirm,
  onCancel,
  variant = 'info',
}: ConfirmDialogProps) {
  if (!isOpen) return null

  const variantClasses = {
    info: 'btn-primary',
    warning: 'btn-warning',
    error: 'btn-error',
    success: 'btn-success',
  }

  return (
    <div className="modal modal-open" onClick={(e) => {
      // Prevent closing modal by clicking backdrop
      if (e.target === e.currentTarget) {
        e.stopPropagation()
      }
    }}>
      <div className="modal-box">
        <h3 className="font-bold text-lg mb-4">{title}</h3>
        <p className="text-sm text-base-content/70 whitespace-pre-wrap">{message}</p>
        <div className="modal-action">
          <button
            type="button"
            className={`btn btn-sm ${variantClasses[variant]}`}
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              onConfirm()
            }}
          >
            {confirmText}
          </button>
          <button
            type="button"
            className="btn btn-sm btn-ghost"
            onClick={(e) => {
              e.preventDefault()
              e.stopPropagation()
              onCancel()
            }}
          >
            {cancelText}
          </button>
        </div>
      </div>
    </div>
  )
}
