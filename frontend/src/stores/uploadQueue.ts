/**
 * useUploadQueueStore — queue locale d'uploads.
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
 * La logique d'envoi HTTP elle-même (POST /api/media etc.) vivra dans
 * un composable useUploadQueue() au prompt 8, qui consommera ce store.
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type UploadType = 'photo' | 'clip' | 'voice'

export type UploadStatus =
  | 'pending'   // dans la queue, pas encore envoyé
  | 'uploading' // POST en cours
  | 'done'      // backend a confirmé
  | 'failed'    // erreur réseau ou serveur, retry possible
  | 'lost'      // page reloadée pendant le upload, File perdu

export type UploadItem = {
  id: string
  type: UploadType
  fileMeta: { name: string; size: number; mimeType: string }
  status: UploadStatus
  /** ISO timestamp */
  createdAt: string
  errorMessage?: string
}

const STORAGE_KEY = 'thepond.uploadQueue.v1'

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
      // localStorage indisponible ou plein — on garde en mémoire
    }
  }

  // ─── State ──────────────────────────────────────────────────
  const items = ref<UploadItem[]>(loadMetadata())

  // Au boot, tout ce qui était en transit a perdu son File.
  // On le marque 'lost' pour que l'UI propose un retry manuel.
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

  /** Items actifs : pending + uploading + failed + lost. Done est exclu
   *  (l'UI peut filtrer dessus pour afficher "X uploads à finaliser"). */
  const active    = computed(() =>
    items.value.filter((i) => i.status !== 'done'),
  )

  // ─── Actions ────────────────────────────────────────────────

  function enqueue(file: File, type: UploadType): UploadItem {
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
    }
    fileMap.set(id, file)
    items.value.push(item)
    persist()
    return item
  }

  /** Récupère le File associé à un item. undefined si statut 'lost'
   *  (le File n'a pas survécu au reload). */
  function getFile(id: string): File | undefined {
    return fileMap.get(id)
  }

  function setStatus(id: string, status: UploadStatus, errorMessage?: string): void {
    const item = items.value.find((i) => i.id === id)
    if (!item) return
    item.status = status
    item.errorMessage = errorMessage
    if (status === 'done') {
      // Libère la référence au File une fois confirmé côté serveur
      fileMap.delete(id)
    }
    persist()
  }

  /** Retire un item de la queue. */
  function dismiss(id: string): void {
    fileMap.delete(id)
    items.value = items.value.filter((i) => i.id !== id)
    persist()
  }

  /** Vide complètement la queue. Utile pour clear() quand le canard
   *  se déconnecte. */
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
    getFile,
    setStatus,
    dismiss,
    clear,
  }
})
