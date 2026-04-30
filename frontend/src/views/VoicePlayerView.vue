<script setup lang="ts">
/**
 * VoicePlayerView — lecteur vocal avec waveform interactif.
 *
 * Pattern repris de design/Mosaic 4 screens.html .v-* :
 *   - hero centré : Duck 104px (couleur du canard auteur) +
 *     pseudo Caprasimo + timestamp mono
 *   - waveform 56 barres parsées du JSON, sections played en
 *     pond-deep, pending en pond-light, knob duck-yellow sur la
 *     position courante
 *   - click sur la waveform = seek (proportionnel à la position X)
 *   - bouton play/pause Duck Yellow 72px avec ombre signature
 *   - caption en italique sous la barre
 *   - prev/next links en bas (fonctionnel avec snapshotStore.mediaVisible
 *     filtré par kind=voice)
 *   - pills like/reply décoratifs (backend likes vient plus tard)
 */
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSnapshotStore } from '../stores/snapshot'
import Duck from '../components/Duck.vue'
import type { DuckColor } from '../types/duck'

const route = useRoute()
const router = useRouter()
const snapshotStore = useSnapshotStore()

const WAVEFORM_BARS = 56

// ─── Liste des vocaux visibles ─────────────────────────────────
const voices = computed(() =>
  snapshotStore.mediaVisible.filter((m) => m.kind === 'voice'),
)

const currentId = computed(() => String(route.params.id ?? ''))
const currentIndex = computed(() =>
  voices.value.findIndex((v) => v.id === currentId.value),
)
const current = computed(() =>
  currentIndex.value >= 0 ? voices.value[currentIndex.value] : null,
)
const prevVoice = computed(() =>
  currentIndex.value > 0 ? voices.value[currentIndex.value - 1] : null,
)
const nextVoice = computed(() =>
  currentIndex.value >= 0 && currentIndex.value < voices.value.length - 1
    ? voices.value[currentIndex.value + 1]
    : null,
)

watch(
  [() => snapshotStore.snapshot, currentId],
  ([snap, id]) => {
    if (snap && id && currentIndex.value === -1) {
      router.replace('/gallery')
    }
  },
  { immediate: true },
)

// ─── Audio + state ─────────────────────────────────────────────
const audioEl = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = computed(() => current.value?.duration_seconds ?? 0)

function togglePlayback() {
  const a = audioEl.value
  if (!a) return
  if (a.paused) {
    void a.play()
  } else {
    a.pause()
  }
}

function onPlay() { isPlaying.value = true }
function onPause() { isPlaying.value = false }
function onTimeUpdate() {
  if (audioEl.value) currentTime.value = audioEl.value.currentTime
}
function onEnded() {
  isPlaying.value = false
  currentTime.value = 0
  // Auto-advance vers le suivant si dispo (UX vocal-vers-vocal en série)
  if (nextVoice.value) {
    router.push(`/voice/${nextVoice.value.id}`)
  }
}

// ─── Waveform : parsing + interaction ─────────────────────────
const waveformBars = computed<number[]>(() => {
  const raw = current.value?.waveform_json
  if (!raw) return new Array(WAVEFORM_BARS).fill(0.4)
  try {
    const arr = JSON.parse(raw) as number[]
    if (!Array.isArray(arr) || arr.length === 0) {
      return new Array(WAVEFORM_BARS).fill(0.4)
    }
    // Down/up-sample à WAVEFORM_BARS
    const out: number[] = []
    for (let i = 0; i < WAVEFORM_BARS; i++) {
      const idx = Math.floor((i / WAVEFORM_BARS) * arr.length)
      out.push(Math.max(0, Math.min(1, arr[idx] ?? 0.4)))
    }
    return out
  } catch {
    return new Array(WAVEFORM_BARS).fill(0.4)
  }
})

const playedRatio = computed(() =>
  duration.value > 0
    ? Math.min(1, currentTime.value / duration.value)
    : 0,
)
const playedBarsCount = computed(() =>
  Math.round(waveformBars.value.length * playedRatio.value),
)

const waveformEl = ref<HTMLElement | null>(null)

function seekFromPointer(e: MouseEvent | TouchEvent) {
  const a = audioEl.value
  const wf = waveformEl.value
  if (!a || !wf || duration.value === 0) return
  const rect = wf.getBoundingClientRect()
  const clientX =
    'touches' in e
      ? (e.touches[0]?.clientX ?? e.changedTouches[0]?.clientX ?? 0)
      : e.clientX
  const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
  a.currentTime = duration.value * ratio
  currentTime.value = a.currentTime
}

// ─── Reset au changement de vocal ──────────────────────────────
watch(current, () => {
  isPlaying.value = false
  currentTime.value = 0
})

// ─── Keyboard nav (dev/desktop) ────────────────────────────────
function onKey(e: KeyboardEvent) {
  if (e.key === 'ArrowRight' && nextVoice.value)
    router.push(`/voice/${nextVoice.value.id}`)
  if (e.key === 'ArrowLeft' && prevVoice.value)
    router.push(`/voice/${prevVoice.value.id}`)
  if (e.key === 'Escape') router.push('/gallery')
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

function formatTime(s: number): string {
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${m}:${sec.toString().padStart(2, '0')}`
}

function formatPostedAt(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const h = d.getHours().toString().padStart(2, '0')
  const m = d.getMinutes().toString().padStart(2, '0')
  return `${h}h${m}`
}
</script>

<template>
  <main v-if="current" class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- Header retour -->
    <header class="px-5 pt-4 pb-1.5">
      <RouterLink
        to="/gallery"
        class="font-sans text-pond-mid text-sm inline-flex items-center gap-1 hover:text-pond-deep"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="15 6 9 12 15 18" />
        </svg>
        Retour à la mare
      </RouterLink>
    </header>

    <!-- Hero : avatar + pseudo + timestamp -->
    <div class="text-center px-5 pt-4">
      <div class="v-avatar mx-auto mb-3">
        <Duck :color="userColor" :size="74" />
      </div>
      <h1 class="font-display text-pond-deep text-[22px] leading-tight break-words px-3">
        {{ current.user_pseudo ?? 'Canard inconnu' }}
      </h1>
      <p class="font-mono text-[10px] text-ink-soft tracking-wider mt-1">
        {{ formatPostedAt(current.posted_at) }}
      </p>
    </div>

    <!-- Audio element invisible -->
    <audio
      ref="audioEl"
      :src="`/uploads/${current.filename}`"
      preload="metadata"
      @play="onPlay"
      @pause="onPause"
      @timeupdate="onTimeUpdate"
      @ended="onEnded"
    />

    <!-- Waveform interactif -->
    <div class="px-5 mt-5">
      <div
        ref="waveformEl"
        class="v-wave"
        @click="seekFromPointer"
        @touchend.passive="seekFromPointer"
      >
        <span
          v-for="(h, i) in waveformBars"
          :key="i"
          class="v-bar"
          :class="{ 'v-bar--played': i < playedBarsCount }"
          :style="{ height: Math.round(8 + h * 60) + 'px' }"
        />
        <div
          class="v-knob"
          :style="{ left: (playedRatio * 100) + '%' }"
        />
      </div>
      <div class="flex justify-between font-mono text-[11px] text-ink-soft tracking-wider mt-2">
        <span class="text-pond-deep font-medium">{{ formatTime(currentTime) }}</span>
        <span>{{ formatTime(duration) }}</span>
      </div>
    </div>

    <!-- Bouton play / pause -->
    <div class="flex justify-center mt-5">
      <button
        type="button"
        class="v-play"
        :aria-label="isPlaying ? 'Pause' : 'Lecture'"
        @click="togglePlayback"
      >
        <svg v-if="isPlaying" width="26" height="26" viewBox="0 0 24 24" fill="currentColor">
          <rect x="7" y="5" width="3.5" height="14" rx="1" />
          <rect x="13.5" y="5" width="3.5" height="14" rx="1" />
        </svg>
        <svg v-else width="28" height="28" viewBox="0 0 24 24" fill="currentColor">
          <polygon points="7,4 21,12 7,20" />
        </svg>
      </button>
    </div>

    <!-- Caption -->
    <p
      v-if="current.caption"
      class="text-center font-sans text-[13px] text-ink-soft mt-4 mx-6 leading-snug"
    >
      {{ current.caption }}
    </p>

    <!-- Pills like / reply (décoratifs en V1) -->
    <div class="flex justify-center gap-2.5 mt-5 px-5">
      <button class="v-pill" type="button" aria-label="J’aime ce vocal">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" style="color: var(--rec-red)">
          <path d="M12 21s-7-4.5-9.5-9A5.5 5.5 0 0 1 12 6a5.5 5.5 0 0 1 9.5 6c-2.5 4.5-9.5 9-9.5 9z" />
        </svg>
        Cœur
      </button>
      <button class="v-pill" type="button" aria-label="Répondre par un vocal">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--pond-mid)">
          <rect x="9" y="2" width="6" height="12" rx="3" />
          <path d="M5 11a7 7 0 0 0 14 0" />
        </svg>
        Répondre
      </button>
    </div>

    <!-- Prev / next nav (poussé tout en bas) -->
    <nav class="mt-auto flex justify-between items-center px-6 pt-8 pb-7 font-mono text-[10px] uppercase tracking-wider text-pond-mid">
      <RouterLink
        v-if="prevVoice"
        :to="`/voice/${prevVoice.id}`"
        class="inline-flex items-center gap-1 hover:text-pond-deep"
      >
        ← Précédent
      </RouterLink>
      <span v-else class="opacity-40">← Précédent</span>

      <span class="text-ink-soft">
        {{ currentIndex + 1 }} / {{ voices.length }}
      </span>

      <RouterLink
        v-if="nextVoice"
        :to="`/voice/${nextVoice.id}`"
        class="inline-flex items-center gap-1 hover:text-pond-deep"
      >
        Suivant →
      </RouterLink>
      <span v-else class="opacity-40">Suivant →</span>
    </nav>
  </main>

  <main v-else class="min-h-screen flex items-center justify-center">
    <p class="font-display text-2xl text-ink-soft">…</p>
  </main>
</template>

<style scoped>
.v-avatar {
  width: 104px;
  height: 104px;
  border-radius: 50%;
  background: var(--cream-deep);
  border: 3px solid var(--pond-deep);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.v-wave {
  height: 72px;
  display: flex;
  align-items: center;
  gap: 3px;
  position: relative;
  cursor: pointer;
  user-select: none;
  touch-action: manipulation;
}

.v-bar {
  flex: 1;
  min-width: 4px;
  background: var(--pond-light);
  border-radius: 2px;
  transition: background 0.1s linear;
}

.v-bar--played {
  background: var(--pond-deep);
}

.v-knob {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--duck);
  border: 2px solid var(--pond-deep);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
  pointer-events: none;
}

.v-play {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: var(--duck);
  color: var(--pond-deep);
  border: none;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 4px 0 var(--duck-deep),
    0 14px 24px -8px rgba(242, 180, 0, 0.45);
  transition: transform 0.1s ease, box-shadow 0.1s ease;
}

.v-play:active {
  transform: translateY(3px);
  box-shadow:
    0 1px 0 var(--duck-deep),
    0 4px 10px -4px rgba(242, 180, 0, 0.4);
}

.v-pill {
  padding: 8px 14px;
  border-radius: 999px;
  border: 1px solid var(--cream-line);
  background: var(--cream);
  font-family: var(--sans);
  font-size: 12px;
  font-weight: 500;
  color: var(--ink);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.v-pill:hover {
  background: var(--cream-deep);
}
</style>
