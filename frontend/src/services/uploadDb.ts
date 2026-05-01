/**
 * uploadDb — IndexedDB wrapper pour persister la queue d'uploads.
 *
 * On stocke à la fois les métadonnées et le Blob lui-même dans la même row
 * (IDB sait sérialiser les Blobs, contrairement à localStorage). Au reload,
 * la queue se reconstitue complètement et le runner peut reprendre les
 * uploads pending sans perte.
 *
 * La structure du store IDB est intentionnellement plate (pas d'index
 * complexes) parce qu'on ne cherche jamais sur un attribut autre que `id` :
 * on lit tout (`getAll`) au boot et on filtre côté JS, ou on cible un id
 * précis. Volume attendu : ~10-50 entries max par invité.
 */
import { openDB, type DBSchema, type IDBPDatabase } from 'idb'

const DB_NAME = 'thepond'
const DB_VERSION = 1
const STORE = 'pending_uploads'

export type UploadKind = 'photo' | 'clip' | 'voice'

export type UploadStatus =
  | 'pending'
  | 'uploading'
  | 'done'
  | 'failed'
  | 'lost'

export interface PendingUploadRow {
  id: string
  kind: UploadKind
  blob: Blob
  fileName: string
  mimeType: string
  size: number
  status: UploadStatus
  retries: number
  createdAt: string
  errorMessage?: string
  durationSeconds?: number
  caption?: string
  serverResponse?: { id: string; filename?: string; thumb_filename?: string }
}

interface ThePondDb extends DBSchema {
  [STORE]: {
    key: string
    value: PendingUploadRow
  }
}

let dbPromise: Promise<IDBPDatabase<ThePondDb>> | null = null

function getDb(): Promise<IDBPDatabase<ThePondDb>> {
  if (!dbPromise) {
    dbPromise = openDB<ThePondDb>(DB_NAME, DB_VERSION, {
      upgrade(db) {
        if (!db.objectStoreNames.contains(STORE)) {
          db.createObjectStore(STORE, { keyPath: 'id' })
        }
      },
    })
  }
  return dbPromise
}

export async function putUpload(row: PendingUploadRow): Promise<void> {
  const db = await getDb()
  await db.put(STORE, row)
}

export async function getUpload(id: string): Promise<PendingUploadRow | undefined> {
  const db = await getDb()
  return db.get(STORE, id)
}

export async function getAllUploads(): Promise<PendingUploadRow[]> {
  const db = await getDb()
  return db.getAll(STORE)
}

export async function deleteUpload(id: string): Promise<void> {
  const db = await getDb()
  await db.delete(STORE, id)
}

export async function clearUploads(): Promise<void> {
  const db = await getDb()
  await db.clear(STORE)
}
