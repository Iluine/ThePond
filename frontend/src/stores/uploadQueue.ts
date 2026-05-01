/**
 * useUploadQueueStore — queue d'uploads offline-first.
 *
 * Persistance complète dans IndexedDB (metadata + Blob), via
 * `services/uploadDb.ts`. Survit aux reloads et aux fermetures d'onglet.
 *
 * Le runner respecte `navigator.onLine` :
 *   - online + pending  → POST immédiat
 *   - offline           → reste en pending, retry automatique au retour
 *                         de l'événement `online` (cf. installAutoRetry)
 *
 * `enqueue()` fire-and-forget pour préserver l'optimistic UI (la
 * navigation /confirmation est immédiate).
 *
 * Statut 'lost' n'est plus utilisé activement — il subsiste comme valeur
 * possible si IDB est invalidé par le browser, mais en condition normale
 * on bascule directement entre pending → uploading → done|failed.
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useUserStore } from './user'
import {
  putUpload,
  getAllUploads,
  deleteUpload as dbDelete,
  clearUploads as dbClear,
  type PendingUploadRow,
  type UploadKind,
  type UploadStatus,
} from '../services/uploadDb'
import { onReconnect } from '../composables/useNetworkStatus'

export type UploadType = UploadKind
export type { UploadStatus }

export type UploadServerResponse = {
  id: string
  filename?: string
  thumb_filename?: string
}

export type UploadItem = {
  id: string
  type: UploadType
  fileMeta: { name: string; size: number; mimeType: string }
  status: UploadStatus
  createdAt: string
  errorMessage?: string
  durationSeconds?: number
  caption?: string
  serverResponse?: UploadServerResponse
}

export type EnqueueOptions = {
  durationSeconds?: number
  caption?: string
}

const ENDPOINT_FOR: Record<UploadType, string> = {
  photo: '/api/media',
  clip: '/api/clips',
  voice: '/api/voice',
}

const MAX_RETRIES = 5

function rowToItem(row: PendingUploadRow): UploadItem {
  return {
    id: row.id,
    type: row.kind,
    fileMeta: {
      name: row.fileName,
      size: row.size,
      mimeType: row.mimeType,
    },
    status: row.status,
    createdAt: row.createdAt,
    errorMessage: row.errorMessage,
    durationSeconds: row.durationSeconds,
    caption: row.caption,
    serverResponse: row.serverResponse,
  }
}

export const useUploadQueueStore = defineStore('uploadQueue', () => {
  const items = ref<UploadItem[]>([])
  const blobs = new Map<string, Blob>()
  let rehydrated = false
  let autoRetryInstalled = false

  // ─── Boot rehydrate ─────────────────────────────────────────
  async function rehydrate(): Promise<void> {
    if (rehydrated) return
    rehydrated = true
    try {
      const rows = await getAllUploads()
      for (const row of rows) {
        // Tout ce qui était 'uploading' au moment d'un crash redevient
        // 'pending' pour être re-tenté au prochain runner tick.
        if (row.status === 'uploading') {
          row.status = 'pending'
          await putUpload(row)
        }
        blobs.set(row.id, row.blob)
        items.value.push(rowToItem(row))
      }
    } catch (err) {
      console.warn('[uploadQueue] rehydrate failed', err)
    }
    installAutoRetry()
    // Tente immédiatement de drainer la queue si on est online.
    if (typeof navigator === 'undefined' || navigator.onLine) {
      void drainPending()
    }
  }

  function installAutoRetry(): void {
    if (autoRetryInstalled) return
    autoRetryInstalled = true
    onReconnect(() => {
      void drainPending()
    })
  }

  /** Relance tous les items pending. Appelé au boot et sur 'online'. */
  async function drainPending(): Promise<void> {
    const pendings = items.value.filter(
      (i) => i.status === 'pending' || i.status === 'failed',
    )
    for (const item of pendings) {
      // Skip les failed qui ont dépassé MAX_RETRIES
      const row = await tryGetRow(item.id)
      if (row && row.retries >= MAX_RETRIES && item.status === 'failed') continue
      // Sequentiel pour ne pas saturer le backend en cas de 50 items.
      await runUpload(item.id)
    }
  }

  async function tryGetRow(id: string): Promise<PendingUploadRow | undefined> {
    try {
      const rows = await getAllUploads()
      return rows.find((r) => r.id === id)
    } catch {
      return undefined
    }
  }

  // ─── Getters ────────────────────────────────────────────────
  const pending   = computed(() => items.value.filter((i) => i.status === 'pending'))
  const uploading = computed(() => items.value.filter((i) => i.status === 'uploading'))
  const failed    = computed(() => items.value.filter((i) => i.status === 'failed'))
  const lost      = computed(() => items.value.filter((i) => i.status === 'lost'))
  const done      = computed(() => items.value.filter((i) => i.status === 'done'))
  const active    = computed(() =>
    items.value.filter((i) => i.status !== 'done'),
  )
  /** Compteur agrégé "X en attente" affiché dans la pill UI. */
  const inFlightCount = computed(
    () => pending.value.length + uploading.value.length + failed.value.length,
  )

  // ─── Mutations ──────────────────────────────────────────────

  function enqueue(
    file: File,
    type: UploadType,
    opts?: EnqueueOptions,
  ): UploadItem {
    const id = crypto.randomUUID()
    const item: UploadItem = {
      id,
      type,
      fileMeta: {
        name: file.name,
        size: file.size,
        mimeType: file.type,
      },
      status: 'pending',
      createdAt: new Date().toISOString(),
      durationSeconds: opts?.durationSeconds,
      caption: opts?.caption,
    }
    blobs.set(id, file)
    items.value.push(item)

    // Persist async (don't block UI).
    void putUpload({
      id,
      kind: type,
      blob: file,
      fileName: file.name,
      mimeType: file.type,
      size: file.size,
      status: 'pending',
      retries: 0,
      createdAt: item.createdAt,
      durationSeconds: opts?.durationSeconds,
      caption: opts?.caption,
    })

    void runUpload(id)
    return item
  }

  function getById(id: string): UploadItem | undefined {
    return items.value.find((i) => i.id === id)
  }

  function getFile(id: string): Blob | undefined {
    return blobs.get(id)
  }

  async function setStatus(
    id: string,
    status: UploadStatus,
    extra?: { errorMessage?: string; serverResponse?: UploadServerResponse },
  ): Promise<void> {
    const item = items.value.find((i) => i.id === id)
    if (!item) return
    item.status = status
    item.errorMessage = extra?.errorMessage
    if (extra?.serverResponse) item.serverResponse = extra.serverResponse

    if (status === 'done') {
      // L'item ne sert plus de queue work : on libère la place.
      blobs.delete(id)
      items.value = items.value.filter((i) => i.id !== id)
      try { await dbDelete(id) } catch { /* idempotent */ }
      return
    }

    // Sinon on persiste l'évolution.
    const row = await tryGetRow(id)
    if (row) {
      row.status = status
      row.errorMessage = extra?.errorMessage
      if (status === 'failed') row.retries += 1
      try { await putUpload(row) } catch { /* idempotent */ }
    }
  }

  async function runUpload(id: string): Promise<void> {
    const item = items.value.find((i) => i.id === id)
    if (!item) return
    if (item.status === 'uploading' || item.status === 'done') return

    const blob = blobs.get(id)
    if (!blob) {
      await setStatus(id, 'lost')
      return
    }

    if (typeof navigator !== 'undefined' && !navigator.onLine) {
      // Offline : on garde 'pending' et le retry auto sur 'online' fera
      // le job. Pas de tentative bruyante.
      return
    }

    const userStore = useUserStore()
    const userId = userStore.userId
    if (!userId) {
      await setStatus(id, 'failed', { errorMessage: 'pas de canard authentifié' })
      return
    }

    await setStatus(id, 'uploading')

    try {
      const formData = new FormData()
      formData.append('user_id', userId)
      formData.append('file', blob, item.fileMeta.name)
      if ((item.type === 'clip' || item.type === 'voice') && item.durationSeconds !== undefined) {
        formData.append('duration_seconds', item.durationSeconds.toFixed(2))
      }
      if (item.type === 'voice' && item.caption) {
        formData.append('caption', item.caption)
      }

      const res = await fetch(ENDPOINT_FOR[item.type], {
        method: 'POST',
        body: formData,
      })
      if (!res.ok) {
        const body = await res.json().catch(() => ({}))
        throw new Error(body.error || `HTTP ${res.status}`)
      }
      const serverResponse = (await res.json()) as UploadServerResponse
      await setStatus(id, 'done', { serverResponse })
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err)
      await setStatus(id, 'failed', { errorMessage: msg })
    }
  }

  function retry(id: string): void {
    const item = items.value.find((i) => i.id === id)
    if (!item || (item.status !== 'failed' && item.status !== 'lost')) return
    if (!blobs.has(id)) {
      void setStatus(id, 'lost')
      return
    }
    item.errorMessage = undefined
    void runUpload(id)
  }

  async function dismiss(id: string): Promise<void> {
    blobs.delete(id)
    items.value = items.value.filter((i) => i.id !== id)
    try { await dbDelete(id) } catch { /* idempotent */ }
  }

  async function clear(): Promise<void> {
    blobs.clear()
    items.value = []
    try { await dbClear() } catch { /* idempotent */ }
  }

  return {
    items,
    pending,
    uploading,
    failed,
    lost,
    done,
    active,
    inFlightCount,
    rehydrate,
    enqueue,
    getById,
    getFile,
    retry,
    dismiss,
    clear,
  }
})
