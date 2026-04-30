<script setup lang="ts">
/**
 * ClipPlayerView — lecteur clip TikTok-style fullscreen.
 *
 * Pattern repris de design/Mosaic 4 screens.html .clip-stage :
 *   - vidéo plein écran centrée (object-fit cover)
 *   - autoplay muted (politique navigateur, on unmute au 1er tap)
 *   - tap sur la vidéo : play/pause toggle (+ unmute si premier tap)
 *   - swipe vertical via touchstart/touchend deltaY :
 *       up   → clip suivant
 *       down → clip précédent (ou close si premier)
 *   - HUD : top close + position pill, right side actions
 *     (like / share / save — décoratifs en V1, backend likes prompt
 *     ultérieur), bottom avatar + pseudo + caption + timestamp
 *
 * La liste des clips vient de snapshotStore.mediaVisible filtré par
 * kind=clip. Si l'id n'est pas trouvé (clip caché, kind différent,
 * ou direct URL bizarre), on renvoie sur /gallery.
 */
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSnapshotStore } from '../stores/snapshot'
import Duck from '../components/Duck.vue'
import type { DuckColor } from '../types/duck'

const route = useRoute()
const router = useRouter()
const snapshotStore = useSnapshotStore()

// ─── Liste des clips visibles ──────────────────────────────────
const clips = computed(() =>
  snapshotStore.mediaVisible.filter((m) => m.kind === 'clip'),
)

const currentId = computed(() => String(route.params.id ?? ''))
const currentIndex = computed(() =>
  clips.value.findIndex((c) => c.id === currentId.value),
)
const current = computed(() =>
  currentIndex.value >= 0 ? clips.value[currentIndex.value] : null,
)
const prevClip = computed(() =>
  currentIndex.value > 0 ? clips.value[currentIndex.value - 1] : null,
)
const nextClip = computed(() =>
  currentIndex.value >= 0 && currentIndex.value < clips.value.length - 1
    ? clips.value[currentIndex.value + 1]
    : null,
)

// Si on a un snapshot chargé et que le clip n'existe pas, on rentre.
watch(
  [() => snapshotStore.snapshot, currentId],
  ([snap, id]) => {
    if (snap && id && currentIndex.value === -1) {
      router.replace('/gallery')
    }
  },
  { immediate: true },
)

// ─── Player state ──────────────────────────────────────────────
const videoEl = ref<HTMLVideoElement | null>(null)
const isPlaying = ref(false)
const isMuted = ref(true)

function togglePlayback() {
  const v = videoEl.value
  if (!v) return
  if (v.paused) {
    // Sur le premier tap, on unmute aussi (gesture utilisateur)
    if (v.muted) {
      v.muted = false
      isMuted.value = false
    }
    void v.play()
  } else {
    v.pause()
  }
}

function onVideoPlay() { isPlaying.value = true }
function onVideoPause() { isPlaying.value = false }
function onVideoEnded() {
  // Auto-advance vers le suivant. Si pas de suivant, loop sur place.
  if (nextClip.value) {
    router.push(`/clip/${nextClip.value.id}`)
  } else if (videoEl.value) {
    videoEl.value.currentTime = 0
    void videoEl.value.play()
  }
}

// ─── Swipe vertical ────────────────────────────────────────────
let touchStartY = 0
let touchStartTime = 0
const SWIPE_PX = 50
const TAP_MS = 300

function onTouchStart(e: TouchEvent) {
  touchStartY = e.touches[0].clientY
  touchStartTime = Date.now()
}

function onTouchEnd(e: TouchEvent) {
  const endY = e.changedTouches[0].clientY
  const dy = endY - touchStartY
  const dt = Date.now() - touchStartTime
  if (Math.abs(dy) < SWIPE_PX) {
    // Tap court → toggle playback
    if (dt < TAP_MS) togglePlayback()
    return
  }
  if (dy < 0) {
    // Swipe up → next
    if (nextClip.value) router.push(`/clip/${nextClip.value.id}`)
  } else {
    // Swipe down → prev (ou close si on est sur le premier)
    if (prevClip.value) router.push(`/clip/${prevClip.value.id}`)
    else close()
  }
}

function close() {
  router.push('/gallery')
}

// ─── Reset playback state on clip change ───────────────────────
watch(current, (c) => {
  if (!c) return
  isPlaying.value = false
  // On laisse muted=true au changement de clip pour respecter l'autoplay
  isMuted.value = true
  // Le video element va re-load via :src reactive ; onPlay/onPause
  // remettront isPlaying à jour.
})

// ─── Keyboard nav (dev/desktop) ────────────────────────────────
function onKey(e: KeyboardEvent) {
  if (e.key === 'ArrowDown' && nextClip.value) router.push(`/clip/${nextClip.value.id}`)
  if (e.key === 'ArrowUp' && prevClip.value) router.push(`/clip/${prevClip.value.id}`)
  if (e.key === 'Escape') close()
  if (e.key === ' ') {
    e.preventDefault()
    togglePlayback()
  }
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))

// ─── Formatting ────────────────────────────────────────────────
const userColor = computed<DuckColor>(
  () => (current.value?.user_color as DuckColor) ?? 'yellow',
)

function formatPostedAt(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const minutes = Math.floor(diffMs / 60000)
  if (minutes < 1) return 'à l’instant'
  if (minutes < 60) return `il y a ${minutes} min`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `il y a ${hours} h`
  return d.toLocaleDateString('fr-FR')
}

function formatDuration(s: number | null | undefined): string {
  if (s === null || s === undefined) return ''
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${m}:${sec.toString().padStart(2, '0')}`
}
</script>

<template>
  <div
    v-if="current"
    class="clip"
    @touchstart.passive="onTouchStart"
    @touchend.passive="onTouchEnd"
  >
    <video
      ref="videoEl"
      class="clip__video"
      :src="`/uploads/${current.filename}`"
      :muted="isMuted"
      autoplay
      playsinline
      @play="onVideoPlay"
      @pause="onVideoPause"
      @ended="onVideoEnded"
    />

    <!-- Pause overlay (visible quand pause manuelle) -->
    <div
      v-if="!isPlaying"
      class="clip__pause-overlay"
      @click.stop="togglePlayback"
    >
      <svg width="32" height="32" viewBox="0 0 24 24" fill="white">
        <polygon points="6,4 20,12 6,20" />
      </svg>
    </div>

    <!-- HUD top -->
    <div class="clip__hud-top">
      <button class="clip__close" aria-label="Fermer" @click.stop="close">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <line x1="6" y1="6" x2="18" y2="18" />
          <line x1="6" y1="18" x2="18" y2="6" />
        </svg>
      </button>
      <div class="clip__pill">
        <span>{{ currentIndex + 1 }}</span>
        <span class="opacity-50">/ {{ clips.length }}</span>
      </div>
    </div>

    <!-- HUD right (actions décoratives V1) -->
    <div class="clip__hud-right">
      <button class="clip__act" aria-label="J’aime">
        <span class="clip__act-ic">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="white">
            <path d="M12 21s-7-4.5-9.5-9A5.5 5.5 0 0 1 12 6a5.5 5.5 0 0 1 9.5 6c-2.5 4.5-9.5 9-9.5 9z" />
          </svg>
        </span>
      </button>
      <button class="clip__act" aria-label="Partager">
        <span class="clip__act-ic">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 11.5a8.4 8.4 0 0 1-1.4 4.6L21 21l-4.9-1.4a8.5 8.5 0 1 1 4.9-8.1z" />
          </svg>
        </span>
      </button>
      <button class="clip__act" aria-label="Sauvegarder">
        <span class="clip__act-ic">
          <svg width="18" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z" />
          </svg>
        </span>
      </button>
      <div class="clip__author-mini">
        <Duck :color="userColor" :size="28" />
      </div>
    </div>

    <!-- HUD bottom : auteur + caption + ts -->
    <div class="clip__hud-bottom">
      <div class="clip__bottom-row">
        <span class="clip__avatar">
          <Duck :color="userColor" :size="20" />
        </span>
        <span class="clip__pseudo">{{ current.user_pseudo ?? 'Canard inconnu' }}</span>
      </div>
      <p v-if="current.caption" class="clip__cap">{{ current.caption }}</p>
      <div class="clip__meta">
        <span>{{ formatPostedAt(current.posted_at) }}</span>
        <span class="opacity-50">·</span>
        <span>{{ formatDuration(current.duration_seconds) }}</span>
      </div>
    </div>

    <!-- Hint swipe (visible 3s puis disparaît si on a un suivant) -->
    <div v-if="nextClip" class="clip__swipe-hint">
      ↑ swipe pour le suivant
    </div>

    <!-- Tap-to-unmute (visible si encore muted) -->
    <div v-if="isMuted" class="clip__unmute-hint" @click.stop="togglePlayback">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
        <line x1="22" y1="9" x2="16" y2="15" />
        <line x1="16" y1="9" x2="22" y2="15" />
      </svg>
      Touche pour le son
    </div>
  </div>

  <!-- Loading / not-found state pendant que le snapshot arrive -->
  <div v-else class="clip clip--empty">
    <p class="font-display text-2xl">…</p>
  </div>
</template>

<style scoped>
.clip {
  position: fixed;
  inset: 0;
  background: #000;
  color: white;
  overflow: hidden;
  user-select: none;
  /* TikTok-like : on bloque le scroll du body */
  touch-action: none;
}
.clip--empty {
  display: flex;
  align-items: center;
  justify-content: center;
}

.clip__video {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  background: #111;
}

.clip__pause-overlay {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 64px;
  height: 64px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.18);
  display: flex;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(6px);
  pointer-events: auto;
  z-index: 3;
}

.clip__hud-top {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 18px 16px;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.55) 0%, transparent 100%);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 4;
}

.clip__close {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  backdrop-filter: blur(6px);
}

.clip__pill {
  font-family: var(--mono);
  font-size: 11px;
  background: rgba(255, 255, 255, 0.15);
  border: 1px solid rgba(255, 255, 255, 0.2);
  padding: 6px 12px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.clip__hud-right {
  position: absolute;
  right: 10px;
  bottom: 140px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
  z-index: 4;
}

.clip__act {
  background: none;
  border: none;
  color: white;
  cursor: pointer;
  padding: 0;
}

.clip__act-ic {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.12);
  backdrop-filter: blur(6px);
}

.clip__author-mini {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: var(--duck);
  border: 2px solid var(--pond-deep);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 6px;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.4);
}

.clip__hud-bottom {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 18px 70px 32px 16px;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.75) 0%, rgba(0, 0, 0, 0) 100%);
  z-index: 4;
}

.clip__bottom-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.clip__avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  background: var(--cream-deep);
  border: 2px solid var(--pond-deep);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.clip__pseudo {
  font-family: var(--sans);
  font-weight: 700;
  font-size: 13px;
}

.clip__cap {
  font-family: var(--sans);
  font-size: 13px;
  line-height: 1.35;
  margin: 0 0 4px;
}

.clip__meta {
  font-family: var(--mono);
  font-size: 10px;
  color: rgba(255, 255, 255, 0.7);
  letter-spacing: 0.03em;
  display: flex;
  align-items: center;
  gap: 8px;
}

.clip__swipe-hint {
  position: absolute;
  bottom: 140px;
  left: 50%;
  transform: translateX(-50%);
  font-family: var(--mono);
  font-size: 9px;
  color: rgba(255, 255, 255, 0.55);
  letter-spacing: 0.16em;
  text-transform: uppercase;
  white-space: nowrap;
  z-index: 3;
  animation: hint-fade 6s ease-out forwards;
}

@keyframes hint-fade {
  0%, 70% { opacity: 1; }
  100%    { opacity: 0; pointer-events: none; }
}

.clip__unmute-hint {
  position: absolute;
  top: 70px;
  left: 50%;
  transform: translateX(-50%);
  font-family: var(--mono);
  font-size: 11px;
  letter-spacing: 0.04em;
  background: rgba(0, 0, 0, 0.6);
  padding: 8px 14px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  backdrop-filter: blur(6px);
  z-index: 3;
}
</style>
