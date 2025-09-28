import { useParams, Navigate } from 'react-router-dom'
import { CODING_AGENTS } from '../types/providers'
import AgentConfig from '../components/Configuration/AgentConfig'

function ProviderPage() {
  const { providerId } = useParams<{ providerId: string }>()

  const agent = CODING_AGENTS.find(a => a.id === providerId)

  if (!agent) {
    return <Navigate to="/overview" replace />
  }

  return (
    <div className="p-3">
      <div className="mb-3">
        <div className="flex items-center gap-3 mb-2">
          <div className={`avatar placeholder`}>
            <div className={`bg-gradient-to-r ${agent.color} rounded-lg w-8 h-8 text-white`}>
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d={agent.icon} />
              </svg>
            </div>
          </div>
          <div>
            <h1 className="text-lg font-bold text-base-content">{agent.name}</h1>
            <p className="text-sm text-base-content/70">{agent.description}</p>
          </div>
        </div>
      </div>

      <AgentConfig agent={agent} />
    </div>
  )
}

export default ProviderPage