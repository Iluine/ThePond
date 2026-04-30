/**
 * useWitnessStore — état d'auth des témoins.
 *
 * Le token vient de l'URL (?token=…) que l'admin a partagée via QR.
 * On le stocke en sessionStorage : il survit aux navigations dans
 * l'onglet mais pas à la fermeture (les témoins se reconnectent
 * via le QR si besoin, et un restart du backend invalide tous les
 * tokens existants de toute façon).
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

const STORAGE_KEY = 'thepond.witness.v1'

export const useWitnessStore = defineStore('witness', () => {
  function load(): string | null {
    try {
      return sessionStorage.getItem(STORAGE_KEY)
    } catch {
      return null
    }
  }

  const token = ref<string | null>(load())

  const isAuthenticated = computed(() => token.value !== null && token.value.length > 0)

  function setToken(t: string): void {
    token.value = t
    try {
      sessionStorage.setItem(STORAGE_KEY, t)
    } catch {
      /* sessionStorage indisponible */
    }
  }

  function clear(): void {
    token.value = null
    try {
      sessionStorage.removeItem(STORAGE_KEY)
    } catch {
      /* noop */
    }
  }

  /** Construit une URL d'API en y collant ?token=… */
  function withToken(path: string): string {
    if (!token.value) return path
    const sep = path.includes('?') ? '&' : '?'
    return `${path}${sep}token=${encodeURIComponent(token.value)}`
  }

  return { token, isAuthenticated, setToken, clear, withToken }
})
