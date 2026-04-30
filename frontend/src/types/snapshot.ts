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

/** Type discriminator pour Media. */
export type MediaKind = 'photo' | 'clip' | 'voice'

/**
 * Un média polymorphe dans le snapshot. Le champ `kind` discrimine le
 * type. Les champs optionnels sont peuplés selon le kind :
 *   - photo : pas de duration / waveform / caption
 *   - clip  : duration_seconds
 *   - voice : duration_seconds + waveform_json + caption?
 *
 * `user_pseudo` et `user_color` sont joints côté backend pour éviter
 * un round-trip par carte de la galerie.
 */
export type Media = {
  id: string
  kind: MediaKind
  user_id: string
  filename: string
  /** Vide pour les vocaux (pas de thumbnail concept). */
  thumb_filename: string
  posted_at: string
  hidden: boolean
  duration_seconds?: number | null
  waveform_json?: string | null
  caption?: string | null
  user_pseudo?: string | null
  user_color?: DuckColor | null
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
  /** Palier en cours. null tant qu'aucun palier n'a démarré
   *  (ex. avant 19h00 le jour J, ou en environnement de dev sans seed). */
  phase_current: Phase | null
  phase_visible: Phase | null
  phases_all: Phase[]
  media_visible: Media[]
  /** Médias récents pour Mare TV (palier en cours, non encore visibles). */
  media_recent_for_tv: Media[]
  counts: SnapshotCounts
}
