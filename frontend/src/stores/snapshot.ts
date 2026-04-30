/**
 * useSnapshotStore — état SSE de la mare.
 *
 * PROJECT.md § "Architecture temps réel" : à chaque changement, le
 * backend envoie un snapshot complet. Pas de delta, pas de gestion
 * d'événements manqués. Le client remplace son état local entièrement.
 *
 * Ce store ne fait que stocker l'état. La connexion SSE elle-même
 * vit dans le composable useEventStream() (prompt 5), qui appelle
 * setSnapshot() / setConnected() sur ce store.
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Snapshot } from '../types/snapshot'

export const useSnapshotStore = defineStore('snapshot', () => {
  // ─── State ──────────────────────────────────────────────────
  const snapshot = ref<Snapshot | null>(null)
  const connected = ref(false)
  const lastReceivedAt = ref<Date | null>(null)

  // ─── Getters dérivés ────────────────────────────────────────
  // On expose des accesseurs sûrs (?? null / ?? []) pour que les
  // composants n'aient pas à vérifier snapshot != null partout.
  const phaseCurrent = computed(() => snapshot.value?.phase_current ?? null)
  const phaseVisible = computed(() => snapshot.value?.phase_visible ?? null)
  const phasesAll = computed(() => snapshot.value?.phases_all ?? [])
  const mediaVisible = computed(() => snapshot.value?.media_visible ?? [])
  const mediaRecentForTv = computed(() => snapshot.value?.media_recent_for_tv ?? [])
  const counts = computed(() => snapshot.value?.counts ?? null)
  const serverTime = computed(() => snapshot.value?.server_time ?? null)

  /** Le palier visible existe-t-il ? Sinon on est en état "mare endormie". */
  const hasVisiblePhase = computed(() => phaseVisible.value !== null)

  // ─── Mutators (appelés par useEventStream prompt 5) ─────────
  function setSnapshot(s: Snapshot): void {
    snapshot.value = s
    lastReceivedAt.value = new Date()
  }

  function setConnected(b: boolean): void {
    connected.value = b
  }

  function clear(): void {
    snapshot.value = null
    connected.value = false
    lastReceivedAt.value = null
  }

  return {
    snapshot,
    connected,
    lastReceivedAt,
    phaseCurrent,
    phaseVisible,
    phasesAll,
    mediaVisible,
    mediaRecentForTv,
    counts,
    serverTime,
    hasVisiblePhase,
    setSnapshot,
    setConnected,
    clear,
  }
})
