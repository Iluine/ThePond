/**
 * Types partagés pour le snapshot SSE.
 *
 * Conformes au schéma défini dans PROJECT.md § "Architecture temps réel"
 * et aux migrations SQLite (backend/migrations/0001_init.sql).
 *
 * Ces types sont actuellement maintenus à la main. Quand le backend
 * grossira, on pourra générer le TS depuis Rust avec ts-rs ou similaire.
 */

import type { DuckColor } from './duck'

/** Un palier (Apéro, Dîner, etc.) tel qu'exposé par le backend. */
export type Phase = {
  id: number
  phase_order: number
  name: string
  /** ISO timestamp. Le calcul current/visible se fait via target_time <= NOW() */
  target_time: string
  /** ISO timestamp si déjà déclenché (peut être forcé par un témoin). */
  triggered_at: string | null
  is_final_reveal: boolean
}

/** Un média photo dans le snapshot. */
export type Media = {
  id: string
  user_id: string
  filename: string
  thumb_filename: string
  posted_at: string
  hidden: boolean
  /** Enrichissements pour l'UI — joints côté backend pour éviter un round-trip */
  user_pseudo?: string
  user_color?: DuckColor
}

export type SnapshotCounts = {
  total_users: number
  total_posts: number
  posts_visible: number
  posts_pending: number
}

/**
 * État complet poussé par le backend via SSE.
 * Pas de delta : à chaque changement, le client remplace son état par
 * ce snapshot (cf. PROJECT.md § "SSE avec snapshot complet").
 */
export type Snapshot = {
  /** ISO timestamp serveur — utile pour calculer le drift d'horloge client. */
  server_time: string
  phase_current: Phase
  phase_visible: Phase | null
  phases_all: Phase[]
  media_visible: Media[]
  /** Médias récents pour Mare TV (palier en cours, non encore visibles). */
  media_recent_for_tv: Media[]
  counts: SnapshotCounts
}
