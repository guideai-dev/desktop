import React, { useState } from 'react'
import { useAuth } from '../hooks/useAuth'

export default function Login() {
  const [serverUrl, setServerUrl] = useState('http://localhost:3000')
  const { login, isLoggingIn } = useAuth()

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    login(serverUrl)
  }

  return (
    <div className="card bg-base-200 shadow-lg">
      <div className="card-body">
        <h2 className="card-title justify-center mb-2 text-base">Sign In</h2>

        <form onSubmit={handleSubmit} className="space-y-3">
          <div className="form-control">
            <label className="label">
              <span className="label-text">Server URL</span>
            </label>
            <input
              type="url"
              className="input input-bordered"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              placeholder="https://api.guideai.com"
              disabled={isLoggingIn}
            />
          </div>

          <div className="card-actions justify-center">
            <button
              type="submit"
              className="btn btn-primary"
              disabled={isLoggingIn || !serverUrl}
            >
              {isLoggingIn ? (
                <>
                  <span className="loading loading-spinner loading-sm"></span>
                  Signing In...
                </>
              ) : (
                'Sign In with GitHub'
              )}
            </button>
          </div>
        </form>

        <div className="text-center mt-2">
          <p className="text-sm text-base-content/70">
            This will open your browser to complete the OAuth flow
          </p>
        </div>
      </div>
    </div>
  )
}