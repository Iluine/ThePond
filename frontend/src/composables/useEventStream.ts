/**
 * useEventStream — connexion SSE persistante vers /api/events.
 *
 * Le backend envoie un snapshot complet à chaque changement (cf.
 * PROJECT.md § "Architecture temps réel"). On parse, on appelle
 * useSnapshotStore.setSnapshot(), et on laisse Pinia notifier l'UI.
 *
 * Reconnexion : EventSource retente automatiquement avec backoff
 * exponentiel quand la connexion saute. On suit l'état via
 * onopen/onerror et on l'expose via useSnapshotStore.connected.
 *
 * Usage typique : monter une seule fois en haut de l'arbre (App.vue).
 *   useEventStream() // autoConnect par défaut
 */
import { onBeforeUnmount, onMounted } from 'vue'
import { useSnapshotStore } from '../stores/snapshot'
import type { Snapshot } from '../types/snapshot'

export type UseEventStreamOptions = {
  /** Endpoint SSE. Le proxy Vite redirige /api → backend en dev. */
  url?: string
  /** Si true (défaut), connect onMounted + disconnect onBeforeUnmount. */
  autoConnect?: boolean
}

export function useEventStream(opts: UseEventStreamOptions = {}) {
  const url = opts.url ?? '/api/events'
  const autoConnect = opts.autoConnect ?? true

  const store = useSnapshotStore()
  let es: EventSource | null = null

  function connect(): void {
    if (es) return
    es = new EventSource(url)

    es.onopen = () => {
      store.setConnected(true)
      // tracing dev — utile pour debug, pas catastrophique en prod
      // eslint-disable-next-line no-console
      console.debug('[useEventStream] open', url)
    }

    es.onmessage = (ev) => {
      try {
        const snap = JSON.parse(ev.data) as Snapshot
        store.setSnapshot(snap)
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error('[useEventStream] failed to parse snapshot payload', err, ev.data)
      }
    }

    es.onerror = () => {
      // EventSource passe automatiquement en CONNECTING puis retente.
      // On reflète l'état dans le store. onopen rebasculera connected=true
      // au retour de la connexion.
      store.setConnected(false)
    }
  }

  function disconnect(): void {
    if (!es) return
    es.close()
    es = null
    store.setConnected(false)
  }

  if (autoConnect) {
    onMounted(connect)
    onBeforeUnmount(disconnect)
  }

  return { connect, disconnect }
}
