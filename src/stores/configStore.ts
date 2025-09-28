import { create } from 'zustand'
import { ProviderConfig } from '../types/providers'
import { invoke } from '@tauri-apps/api'

interface ConfigState {
  providerConfigs: Record<string, ProviderConfig>
  isLoading: boolean
  error: string | null

  // Actions
  loadProviderConfig: (providerId: string) => Promise<void>
  saveProviderConfig: (providerId: string, config: ProviderConfig) => Promise<void>
  deleteProviderConfig: (providerId: string) => Promise<void>
  clearError: () => void
}

export const useConfigStore = create<ConfigState>((set) => ({
  providerConfigs: {},
  isLoading: false,
  error: null,

  loadProviderConfig: async (providerId: string) => {
    try {
      set({ isLoading: true, error: null })
      const config = await invoke<ProviderConfig>('load_provider_config_command', { providerId })
      set(state => ({
        providerConfigs: { ...state.providerConfigs, [providerId]: config },
        isLoading: false
      }))
    } catch (error) {
      set({ error: error as string, isLoading: false })
    }
  },

  saveProviderConfig: async (providerId: string, config: ProviderConfig) => {
    try {
      set({ isLoading: true, error: null })
      await invoke('save_provider_config_command', { providerId, config })
      set(state => ({
        providerConfigs: { ...state.providerConfigs, [providerId]: config },
        isLoading: false
      }))
    } catch (error) {
      set({ error: error as string, isLoading: false })
    }
  },

  deleteProviderConfig: async (providerId: string) => {
    try {
      set({ isLoading: true, error: null })
      await invoke('delete_provider_config_command', { providerId })
      set(state => {
        const newConfigs = { ...state.providerConfigs }
        delete newConfigs[providerId]
        return { providerConfigs: newConfigs, isLoading: false }
      })
    } catch (error) {
      set({ error: error as string, isLoading: false })
    }
  },

  clearError: () => set({ error: null })
}))