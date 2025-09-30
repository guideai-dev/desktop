import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { useAuth } from './hooks/useAuth'
import Login from './components/Login'
import AppLayout from './components/Layout/AppLayout'
import OverviewPage from './pages/OverviewPage'
import ProviderPage from './pages/ProviderPage'
import SettingsPage from './pages/SettingsPage'
import UploadQueuePage from './pages/UploadQueuePage'

function App() {
  const { user, isLoading } = useAuth()

  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen bg-base-100" data-theme="guideai">
        <span className="loading loading-spinner loading-lg"></span>
      </div>
    )
  }

  if (!user) {
    return (
      <div className="min-h-screen bg-base-100 flex items-center justify-center" data-theme="guideai">
        <div className="container mx-auto px-2 max-w-md">
          <Login />
        </div>
      </div>
    )
  }

  return (
    <Router>
      <AppLayout>
        <Routes>
          <Route path="/" element={<Navigate to="/overview" replace />} />
          <Route path="/overview" element={<OverviewPage />} />
          <Route path="/provider/:providerId" element={<ProviderPage />} />
          <Route path="/settings" element={<SettingsPage />} />
          <Route path="/upload-queue" element={<UploadQueuePage />} />
        </Routes>
      </AppLayout>
    </Router>
  )
}

export default App