import React from 'react'
import Header from './Header'
import SideNav from './SideNav'

interface AppLayoutProps {
  children: React.ReactNode
}

function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="min-h-screen bg-base-100">
      <Header />
      <div className="flex h-[calc(100vh-60px)]">
        <SideNav />
        <main className="flex-1 overflow-auto main-gradient p-6">
          <div className="max-w-7xl mx-auto">
            {children}
          </div>
        </main>
      </div>
    </div>
  )
}

export default AppLayout