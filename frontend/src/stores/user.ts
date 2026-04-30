/**
 * useUserStore — le canard de l'invité courant.
 *
 * Auth modèle de PROJECT.md § "Authentification" : le client génère un
 * UUID v4 au scan QR, l'envoie au backend avec une couleur de canard,
 * le backend renvoie le canard complet (incluant le pseudo généré).
 * Le UUID est persisté en localStorage et sert d'identifiant pour
 * tous les uploads. Pas de session, pas de cookie.
 *
 * L'endpoint POST /api/users sera implémenté au prompt 6.
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DuckColor } from '../types/duck'

export type Canard = {
  id: string
  pseudo: string
  custom_name?: string | null
  duck_color: DuckColor
  created_at?: string
}

const STORAGE_KEY = 'thepond.user.v1'

export const useUserStore = defineStore('user', () => {
  // ─── Init depuis localStorage ───────────────────────────────
  function loadFromStorage(): Canard | null {
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      return raw ? (JSON.parse(raw) as Canard) : null
    } catch {
      return null
    }
  }

  function persist(c: Canard | null) {
    try {
      if (c) localStorage.setItem(STORAGE_KEY, JSON.stringify(c))
      else localStorage.removeItem(STORAGE_KEY)
    } catch {
      // localStorage indisponible (Safari private, etc.) — silent
    }
  }

  // ─── State ──────────────────────────────────────────────────
  const user = ref<Canard | null>(loadFromStorage())

  // ─── Getters ────────────────────────────────────────────────
  const isAuthenticated = computed(() => user.value !== null)
  const userId = computed(() => user.value?.id ?? null)
  const displayName = computed(
    () => user.value?.custom_name?.trim() || user.value?.pseudo || null,
  )

  // ─── Actions ────────────────────────────────────────────────

  /**
   * Inscription d'un nouveau canard. Le client génère le UUID, le serveur
   * persiste avec le pseudo fourni (issu d'un GET /api/pseudo précédent)
   * ou en génère un si aucun n'est passé.
   */
  async function register(opts: {
    duck_color: DuckColor
    custom_name?: string
    pseudo?: string
  }): Promise<Canard> {
    const id = crypto.randomUUID()

    const res = await fetch('/api/users', {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        id,
        duck_color: opts.duck_color,
        custom_name: opts.custom_name ?? null,
        pseudo: opts.pseudo ?? null,
      }),
    })
    if (!res.ok) {
      throw new Error(`Inscription échouée (HTTP ${res.status})`)
    }
    const canard = (await res.json()) as Canard
    user.value = canard
    persist(canard)
    return canard
  }

  /** Met à jour le custom_name (vrai prénom optionnel). */
  async function setCustomName(name: string): Promise<void> {
    if (!user.value) throw new Error('Pas de canard authentifié')

    const trimmed = name.trim()
    const res = await fetch(`/api/users/${user.value.id}`, {
      method: 'PATCH',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ custom_name: trimmed || null }),
    })
    if (!res.ok) {
      throw new Error(`Mise à jour échouée (HTTP ${res.status})`)
    }
    user.value = { ...user.value, custom_name: trimmed || undefined }
    persist(user.value)
  }

  /** Oublie le canard local. Le canard reste côté serveur ; il faudra
   *  recréer un canard pour reposter (cas peu courant : changement de
   *  device pendant la soirée). */
  function clear(): void {
    user.value = null
    persist(null)
  }

  return {
    user,
    isAuthenticated,
    userId,
    displayName,
    register,
    setCustomName,
    clear,
  }
})
