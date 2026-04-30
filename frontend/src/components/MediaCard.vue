<script setup lang="ts">
/**
 * MediaCard — carte unique du masonry de la galerie.
 *
 * Discriminée par media.kind :
 *   - photo : thumb depuis /uploads/{thumb_filename}
 *   - clip  : gradient gris foncé + play circle blanc + badge durée
 *   - voice : gradient pond-pale + waveform mini + mic icon
 *
 * card-foot : Duck mini (16px) coloré selon user_color + meta
 * "{pseudo} · {time}". La couleur du couple (champagne border) est
 * laissée à la responsabilité du parent via la prop `couple`.
 */
import { computed } from 'vue'
import Duck from './Duck.vue'
import type { Media } from '../types/snapshot'

const props = defineProps<{
  media: Media
  /** Marque la carte comme étant celle des mariées (border champagne). */
  couple?: boolean
}>()

const thumbUrl = computed(() =>
  props.media.kind === 'voice'
    ? null
    : `/uploads/${props.media.thumb_filename}`,
)

const formattedTime = computed(() => {
  // posted_at est un ISO timestamp. On n'affiche que HHhMM
  // (la date complète est rarement utile en galerie de soirée).
  const d = new Date(props.media.posted_at)
  if (isNaN(d.getTime())) return ''
  const h = d.getHours().toString().padStart(2, '0')
  const m = d.getMinutes().toString().padStart(2, '0')
  return `${h}h${m}`
})

const formattedDuration = computed(() => {
  const s = props.media.duration_seconds
  if (s === null || s === undefined) return ''
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${m}:${sec.toString().padStart(2, '0')}`
})

// Waveform mini : on affiche jusqu'à 16 barres parsées depuis le JSON,
// ou un faux waveform sinon. Hauteur scalée 8..42px.
const waveformBars = computed(() => {
  const n = 16
  if (!props.media.waveform_json) {
    // Fallback : waveform plat
    return new Array(n).fill(0.4)
  }
  try {
    const arr = JSON.parse(props.media.waveform_json) as number[]
    if (!Array.isArray(arr) || arr.length === 0) return new Array(n).fill(0.4)
    // Down-sample pour avoir n bars
    const out: number[] = []
    for (let i = 0; i < n; i++) {
      const idx = Math.floor((i / n) * arr.length)
      out.push(Math.max(0, Math.min(1, arr[idx] ?? 0.4)))
    }
    return out
  } catch {
    return new Array(n).fill(0.4)
  }
})

const userColor = computed(() => props.media.user_color ?? 'yellow')
const pseudo = computed(() => props.media.user_pseudo ?? 'Canard inconnu')
</script>

<template>
  <article class="card" :class="{ 'card--couple': couple }">
    <!-- ─── Photo ──────────────────────────────────────────── -->
    <div v-if="media.kind === 'photo'" class="card__media card__media--photo">
      <img
        v-if="thumbUrl"
        :src="thumbUrl"
        :alt="`Photo de ${pseudo}`"
        class="card__photo"
        loading="lazy"
      />
    </div>

    <!-- ─── Clip ───────────────────────────────────────────── -->
    <RouterLink
      v-else-if="media.kind === 'clip'"
      :to="`/clip/${media.id}`"
      class="card__media card__media--clip"
    >
      <span class="card__play">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="#1F2933">
          <polygon points="6,4 20,12 6,20" />
        </svg>
      </span>
      <span class="card__duration">{{ formattedDuration }}</span>
    </RouterLink>

    <!-- ─── Voice ──────────────────────────────────────────── -->
    <RouterLink
      v-else
      :to="`/voice/${media.id}`"
      class="card__media card__media--voice"
    >
      <span class="card__mic">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <rect x="9" y="2" width="6" height="12" rx="3" />
          <path d="M5 11a7 7 0 0 0 14 0" />
          <line x1="12" y1="18" x2="12" y2="22" />
        </svg>
      </span>
      <div class="card__wf">
        <span
          v-for="(h, i) in waveformBars"
          :key="i"
          :style="{ height: Math.round(8 + h * 34) + 'px' }"
        />
      </div>
      <div class="card__voice-time">{{ formattedDuration }}</div>
    </RouterLink>

    <!-- ─── Footer ─────────────────────────────────────────── -->
    <div class="card__foot">
      <span class="card__avatar">
        <Duck :color="userColor" :size="14" />
      </span>
      <span class="card__meta">
        <b>{{ pseudo }}</b>
        <span class="opacity-60"> · {{ formattedTime }}</span>
      </span>
    </div>
  </article>
</template>

<style scoped>
.card {
  break-inside: avoid;
  margin-bottom: 8px;
  border-radius: 12px;
  overflow: hidden;
  background: var(--cream-deep);
  border: 1px solid var(--cream-line);
  box-shadow: 0 4px 10px -6px rgba(14, 79, 107, 0.18);
  position: relative;
}

.card--couple {
  border: 2px solid var(--champagne);
  box-shadow: 0 4px 14px -6px rgba(201, 163, 71, 0.5);
}

.card__media {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  text-decoration: none;
  color: white;
  background: #c8ccd0;
}

.card__media--photo {
  /* La hauteur intrinsèque vient de l'aspect ratio de l'image. */
  min-height: 100px;
}

.card__photo {
  display: block;
  width: 100%;
  height: auto;
}

.card__media--clip {
  height: 140px;
  background: linear-gradient(135deg, #3a4148, #5a6168);
}

.card__play {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.85);
  display: flex;
  align-items: center;
  justify-content: center;
}

.card__duration {
  position: absolute;
  bottom: 6px;
  right: 6px;
  font-family: var(--mono);
  font-size: 9px;
  color: white;
  background: rgba(0, 0, 0, 0.55);
  padding: 2px 5px;
  border-radius: 4px;
  letter-spacing: 0.04em;
}

.card__media--voice {
  height: 160px;
  background: linear-gradient(180deg, var(--pond-pale), #d6ecf3);
  flex-direction: column;
  gap: 8px;
}

.card__mic {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--pond-deep);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
}

.card__wf {
  display: flex;
  align-items: center;
  gap: 2px;
  height: 48px;
  padding: 0 14px;
}

.card__wf span {
  width: 3px;
  background: var(--pond-mid);
  border-radius: 2px;
  flex-shrink: 0;
}

.card__voice-time {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--pond-deep);
  letter-spacing: 0.04em;
}

.card__foot {
  padding: 6px 8px;
  display: flex;
  align-items: center;
  gap: 6px;
  background: var(--cream);
}

.card__avatar {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 50%;
  background: var(--duck);
  border: 1.2px solid var(--pond-deep);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.card__meta {
  font-family: var(--mono);
  font-size: 9px;
  color: var(--ink-soft);
  letter-spacing: 0.02em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  min-width: 0;
}

.card__meta b {
  color: var(--ink);
  font-weight: 500;
}
</style>
