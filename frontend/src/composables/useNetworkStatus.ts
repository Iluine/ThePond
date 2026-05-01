/**
 * useNetworkStatus — reactive isOnline tracker.
 *
 * Wrap navigator.onLine + online/offline events. Listener attaché 1 fois
 * au premier appel et laissé pour la durée de vie de l'app (event globaux
 * qui survivent au navigation entre routes).
 *
 * Note : navigator.onLine est connu pour ses faux positifs (le browser
 * peut dire "online" alors que la requête réelle échouera). Pour V1 on
 * s'en contente — le runner d'upload retry de toute façon en cas de
 * 50x ou erreur réseau.
 */
import { ref, computed, onMounted } from 'vue'

const onlineRef = ref<boolean>(
  typeof navigator !== 'undefined' ? navigator.onLine : true,
)
let attached = false

function attach() {
  if (attached) return
  attached = true
  window.addEventListener('online', () => {
    onlineRef.value = true
  })
  window.addEventListener('offline', () => {
    onlineRef.value = false
  })
}

export function useNetworkStatus() {
  onMounted(attach)
  const isOnline = computed(() => onlineRef.value)
  const isOffline = computed(() => !onlineRef.value)
  return { isOnline, isOffline }
}

/** Subscribe to online events outside a Vue component (e.g. from the
 *  upload queue store). Calls `cb()` whenever the browser reports it
 *  has reconnected. Returns a cleanup function. */
export function onReconnect(cb: () => void): () => void {
  attach()
  const handler = () => cb()
  window.addEventListener('online', handler)
  return () => window.removeEventListener('online', handler)
}
