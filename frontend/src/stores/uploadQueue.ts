/**
 * useUploadQueueStore — queue locale d'uploads avec runner intégré.
 *
 * Persistance hybride :
 *   - Les MÉTADONNÉES (id, type, statut, taille, ...) sont en localStorage
 *     pour qu'on puisse afficher "tu as N uploads en cours" même après
 *     un reload de la page.
 *   - Les FILES eux-mêmes restent en mémoire (Map non-réactive). Les
 *     File objects ne sont pas JSON-serializables, et stocker en base64
 *     coûterait beaucoup pour un cas d'usage rare (page rechargée pendant
 *     un upload). Au reload, les items qui étaient pending/uploading
 *     basculent sur statut 'lost' — l'UI peut alors proposer de
 *     re-sélectionner le fichier.
 *
 * enqueue() lance immédiatement runUpload() en fire-and-forget pour que
 * la navigation /confirmation soit instantanée (optimistic UI). Le statut
 * passe pending → uploading → done|failed sans bloquer l'appelant.
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useUserStore } from './user'

export type UploadType = 'photo' | 'clip' | 'voice'

export type UploadStatus =
  | 'pending'   // dans la queue, pas encore envoyé
  | 'uploading' // POST en cours
  | 'done'      // backend a confirmé
  | 'failed'    // erreur réseau ou serveur, retry possible
  | 'lost'      // page reloadée pendant le upload, File perdu

/** Réponse minimale renvoyée par le backend après un upload réussi.
 *  Les filenames servent à construire l'URL des thumbnails côté client. */
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
  /** ISO timestamp */
  createdAt: string
  errorMessage?: string
  /** Pour les clips et vocaux. */
  durationSeconds?: number
  /** Pour les vocaux uniquement (≤ 280 chars validé côté backend). */
  caption?: string
  /** Présent une fois status === 'done'. Persiste à travers les reloads. */
  serverResponse?: UploadServerResponse
}

export type EnqueueOptions = {
  durationSeconds?: number
  caption?: string
}

const STORAGE_KEY = 'thepond.uploadQueue.v1'

const ENDPOINT_FOR: Record<UploadType, string> = {
  photo: '/api/media',
  clip: '/api/clips',
  voice: '/api/voice',
}

export const useUploadQueueStore = defineStore('uploadQueue', () => {
  // ─── Files (mémoire seule) ──────────────────────────────────
  const fileMap = new Map<string, File>()

  // ─── Persistance des métadonnées ────────────────────────────
  function loadMetadata(): UploadItem[] {
    try {
      const raw = localStorage.getItem(STORAGE_KEY)
      return raw ? (JSON.parse(raw) as UploadItem[]) : []
    } catch {
      return []
    }
  }

  function persist(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(items.value))
    } catch {
      /* localStorage indisponible ou plein */
    }
  }

  // ─── State ──────────────────────────────────────────────────
  const items = ref<UploadItem[]>(loadMetadata())

  // Au boot, tout ce qui était en transit a perdu son File.
  for (const item of items.value) {
    if (item.status === 'pending' || item.status === 'uploading') {
      item.status = 'lost'
    }
  }
  persist()

  // ─── Getters dérivés ────────────────────────────────────────
  const pending   = computed(() => items.value.filter((i) => i.status === 'pending'))
  const uploading = computed(() => items.value.filter((i) => i.status === 'uploading'))
  const failed    = computed(() => items.value.filter((i) => i.status === 'failed'))
  const lost      = computed(() => items.value.filter((i) => i.status === 'lost'))
  const done      = computed(() => items.value.filter((i) => i.status === 'done'))
  const active    = computed(() =>
    items.value.filter((i) => i.status !== 'done'),
  )

  // ─── Actions ────────────────────────────────────────────────

  /**
   * Ajoute un fichier à la queue et lance immédiatement son upload en
   * background. L'appelant peut naviguer vers /confirmation tout de suite
   * et observer l'évolution du statut via getById().
   */
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
    fileMap.set(id, file)
    items.value.push(item)
    persist()
    // Fire-and-forget : on ne bloque pas l'appelant.
    void runUpload(id)
    return item
  }

  function getById(id: string): UploadItem | undefined {
    return items.value.find((i) => i.id === id)
  }

  function getFile(id: string): File | undefined {
    return fileMap.get(id)
  }

  function setStatus(
    id: string,
    status: UploadStatus,
    errorMessage?: string,
  ): void {
    const item = items.value.find((i) => i.id === id)
    if (!item) return
    item.status = status
    item.errorMessage = errorMessage
    if (status === 'done') {
      // Libère le File mais GARDE le serverResponse pour l'UI
      fileMap.delete(id)
    }
    persist()
  }

  /** Lance l'upload pour un item. POST sur l'endpoint approprié,
   *  met à jour le statut et stocke la réponse serveur en cas de succès. */
  async function runUpload(id: string): Promise<void> {
    const item = items.value.find((i) => i.id === id)
    if (!item) return
    const file = fileMap.get(id)
    if (!file) {
      setStatus(id, 'lost')
      return
    }

    const userStore = useUserStore()
    const userId = userStore.userId
    if (!userId) {
      setStatus(id, 'failed', 'pas de canard authentifié')
      return
    }

    setStatus(id, 'uploading')

    try {
      const formData = new FormData()
      formData.append('user_id', userId)
      formData.append('file', file)
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

      const itemRef = items.value.find((i) => i.id === id)
      if (itemRef) {
        itemRef.serverResponse = serverResponse
      }
      setStatus(id, 'done')
    } catch (err) {
      setStatus(id, 'failed', err instanceof Error ? err.message : String(err))
    }
  }

  /** Relance un upload qui a échoué. Le File doit toujours être en
   *  mémoire (sinon le statut serait déjà 'lost'). */
  function retry(id: string): void {
    const item = items.value.find((i) => i.id === id)
    if (!item || (item.status !== 'failed' && item.status !== 'lost')) return
    if (!fileMap.has(id)) {
      setStatus(id, 'lost')
      return
    }
    item.errorMessage = undefined
    void runUpload(id)
  }

  function dismiss(id: string): void {
    fileMap.delete(id)
    items.value = items.value.filter((i) => i.id !== id)
    persist()
  }

  function clear(): void {
    fileMap.clear()
    items.value = []
    persist()
  }

  return {
    items,
    pending,
    uploading,
    failed,
    lost,
    done,
    active,
    enqueue,
    getById,
    getFile,
    setStatus,
    retry,
    dismiss,
    clear,
  }
})
