import React from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { appWindow } from '@tauri-apps/api/window'
import App from './App'
import StatusView from './components/StatusView'
import './index.css'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes
      retry: 2,
      refetchOnWindowFocus: false,
      refetchOnMount: true, // Ensure fresh data after login
    },
  },
})

// Detect which window we're in and render appropriate component
const windowLabel = appWindow.label

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      {windowLabel === 'status' ? <StatusView /> : <App />}
    </QueryClientProvider>
  </React.StrictMode>
)