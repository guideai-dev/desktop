import React from 'react'
import Header from './Header'
import SideNav from './SideNav'

interface AppLayoutProps {
  children: React.ReactNode
}

function AppLayout({ children }: AppLayoutProps) {
  return (
    <div className="min-h-screen bg-base-100" data-theme="guideai">
      <Header />
      <div className="flex h-[calc(100vh-60px)]">
        <SideNav />
        <main className="flex-1 overflow-auto bg-base-200">
          {children}
        </main>
      </div>
    </div>
  )
}

export default AppLayout