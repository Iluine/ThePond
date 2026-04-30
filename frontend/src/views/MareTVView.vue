<script setup lang="ts">
/**
 * MareTVView — kiosque plein écran à projeter sur la TV de la salle.
 *
 * Reproduit design/Slideshow TV v2.html (variantes day & night).
 *
 * Layout vertical :
 *   - top bar       : brand + sub event + PondCounter lg
 *   - media zone    : grand média en cours + attribution à droite
 *   - teaser band   : "X canards se débloqueront avec Y vers Z"
 *   - pond region   : Pond.vue plein largeur avec ducks scattered
 *
 * Rotation auto toutes les 6s sur snapshotStore.mediaRecentForTv
 * (palier en cours, pas encore visibles dans la galerie publique).
 *
 * Day/night : basé sur l'heure locale (≥ 23h ou < 6h = nuit). En
 * V1 c'est pure horloge — le toggle "Mode nuit après 23h" de
 * l'orchestration témoin est décoratif jusqu'au prompt de polish.
 */
import { ref, computed, watch, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import { useSnapshotStore } from '../stores/snapshot'
import Pond, { type PondDuck } from '../components/Pond.vue'
import Duck from '../components/Duck.vue'
import type { Media } from '../types/snapshot'
import type { DuckColor } from '../types/duck'

const snapshotStore = useSnapshotStore()
const route = useRoute()

const ROTATION_MS = 6000
const POND_DUCKS_CAP = 50

// ─── Rotation ──────────────────────────────────────────────────
const items = computed<Media[]>(() => snapshotStore.mediaRecentForTv)
const currentIdx = ref(0)
let rotationTimer: number | null = null

function advance() {
  if (items.value.length === 0) return
  currentIdx.value = (currentIdx.value + 1) % items.value.length
}

// Reset l'index si la liste rapetisse pour éviter d'être out-of-bounds.
watch(items, (newItems) => {
  if (currentIdx.value >= newItems.length) currentIdx.value = 0
})

const currentMedia = computed<Media | null>(() => items.value[currentIdx.value] ?? null)

// ─── Horloge (clock + day/night) ───────────────────────────────
const now = ref(new Date())
let clockTimer: number | null = null

const isNight = computed(() => {
  const h = now.value.getHours()
  return h >= 23 || h < 6
})

const formattedTime = computed(() => {
  const h = now.value.getHours().toString().padStart(2, '0')
  const m = now.value.getMinutes().toString().padStart(2, '0')
  return `${h}:${m}`
})

// ─── Pond ducks (décoratifs, scaled to total_users) ───────────
function pickColor(): DuckColor {
  const r = Math.random()
  if (r < 0.80) return 'yellow'
  if (r < 0.95) return 'white'
  if (r < 0.99) return 'blue'
  return 'rainbow'
}

const totalUsers = computed(() => snapshotStore.counts?.total_users ?? 0)
const expectedGuests = 52

const pondDucks = ref<PondDuck[]>([])
// Re-génère uniquement quand le compte change pour éviter les jitters.
watch(
  totalUsers,
  (n) => {
    const cap = Math.min(n, POND_DUCKS_CAP)
    if (cap === pondDucks.value.length) return
    if (cap > pondDucks.value.length) {
      // Append seulement les nouveaux pour garder les couleurs existantes stables
      const toAdd = cap - pondDucks.value.length
      const start = pondDucks.value.length
      for (let i = 0; i < toAdd; i++) {
        pondDucks.value.push({ id: start + i, color: pickColor() })
      }
    } else {
      pondDucks.value = pondDucks.value.slice(0, cap)
    }
  },
  { immediate: true },
)

// ─── Phase / event meta ────────────────────────────────────────
const phaseCurrent = computed(() => snapshotStore.phaseCurrent)

function formatTimeShort(iso: string | null | undefined): string {
  if (!iso) return ''
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const h = d.getHours()
  const m = d.getMinutes()
  return m === 0 ? `${h}h` : `${h.toString().padStart(2, '0')}h${m.toString().padStart(2, '0')}`
}

const teaserText = computed(() => {
  const cur = phaseCurrent.value
  if (!cur) return 'En attente du premier palier'
  // Trouve le prochain palier régulier (non final)
  const all = snapshotStore.phasesAll
  const nextRegular = all
    .filter((p) => p.phase_order > cur.phase_order && !p.is_final_reveal)
    .sort((a, b) => a.phase_order - b.phase_order)[0]
  if (nextRegular) {
    const n = items.value.length
    return `🌙 ${n} ${n === 1 ? 'canard se débloquera' : 'canards se débloqueront'} avec ${nextRegular.name.toLowerCase()} vers ${formatTimeShort(nextRegular.target_time)}`
  }
  const final = all.find((p) => p.is_final_reveal)
  if (final) {
    return `🌙 Les canards seront confits demain vers ${formatTimeShort(final.target_time)}`
  }
  return ''
})

// ─── Attribution helpers ──────────────────────────────────────
function postedRelative(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const diffMs = now.value.getTime() - d.getTime()
  const minutes = Math.floor(diffMs / 60000)
  if (minutes < 1) return 'à l’instant'
  if (minutes < 60) return `il y a ${minutes} ${minutes === 1 ? 'minute' : 'minutes'}`
  const hours = Math.floor(minutes / 60)
  return `il y a ${hours} h`
}

function durationLabel(s: number | null | undefined): string {
  if (s === null || s === undefined) return ''
  const m = Math.floor(s / 60)
  const sec = Math.floor(s % 60)
  return `${m}:${sec.toString().padStart(2, '0')}`
}

// ─── Lifecycle ─────────────────────────────────────────────────
onMounted(() => {
  rotationTimer = window.setInterval(advance, ROTATION_MS)
  clockTimer = window.setInterval(() => {
    now.value = new Date()
  }, 30_000)
})

onBeforeUnmount(() => {
  if (rotationTimer) clearInterval(rotationTimer)
  if (clockTimer) clearInterval(clockTimer)
})

// ─── Fullscreen toggle ───────────────────────────────────────
const isFullscreen = ref(false)

async function toggleFullscreen() {
  try {
    if (!document.fullscreenElement) {
      await document.documentElement.requestFullscreen()
      isFullscreen.value = true
    } else {
      await document.exitFullscreen()
      isFullscreen.value = false
    }
  } catch {
    /* certains browsers refusent — on ignore */
  }
}

// ─── Pond width responsive ──────────────────────────────────
// Pond.vue prend des dimensions en px ; on les calcule depuis le
// viewport et on les met à jour au resize.
const pondWidth = ref(1920)
const pondHeight = ref(360)

function updatePondSize() {
  pondWidth.value = window.innerWidth
  // 360px sur 1080p, scale proportionnellement
  pondHeight.value = Math.max(220, Math.round(window.innerHeight * 0.33))
}

onMounted(() => {
  updatePondSize()
  window.addEventListener('resize', updatePondSize)
})
onBeforeUnmount(() => {
  window.removeEventListener('resize', updatePondSize)
})

// ─── Event name (V1 hardcodé, idem WelcomeView) ──────────────
const eventName = 'Le mariage de Marie & Thomas'

// Donne accès à l'origine pour le mode dev (Playwright)
const showHud = computed(() => route.query.hud !== '0')
</script>

<template>
  <div class="tv" :class="{ 'tv--night': isNight, 'tv--day': !isNight }">
    <!-- ─── Top bar ─────────────────────────────────────────── -->
    <header class="topbar">
      <div class="brand">
        The <em>Pond</em>.
        <span class="brand__sub">{{ eventName }} · {{ formattedTime }}</span>
      </div>
      <div class="pill-big">
        <span class="pill-big__num">{{ totalUsers }}</span>
        <span class="pill-big__of">/ {{ expectedGuests }}</span>
        <span class="pill-big__label">canards</span>
      </div>
    </header>

    <!-- ─── Media + attribution ─────────────────────────────── -->
    <section class="media-zone">
      <div class="media">
        <template v-if="!currentMedia">
          <div class="media__empty">
            <Duck color="yellow" :size="160" :asleep="isNight" />
            <p class="media__empty-text">
              {{ phaseCurrent
                ? 'En attente des premiers barbotages…'
                : 'La mare s’ouvre bientôt' }}
            </p>
          </div>
        </template>

        <!-- Photo -->
        <img
          v-else-if="currentMedia.kind === 'photo'"
          :src="`/uploads/${currentMedia.thumb_filename}`"
          :alt="`Photo de ${currentMedia.user_pseudo ?? 'canard'}`"
          class="media__img"
        />

        <!-- Clip -->
        <video
          v-else-if="currentMedia.kind === 'clip'"
          :key="currentMedia.id"
          :src="`/uploads/${currentMedia.filename}`"
          class="media__video"
          autoplay
          muted
          playsinline
          loop
        />

        <!-- Voice : avatar + waveform statique + caption -->
        <div v-else class="media__voice">
          <Duck :color="(currentMedia.user_color as DuckColor) ?? 'yellow'" :size="180" />
          <div class="media__voice-wf">
            <span
              v-for="i in 40"
              :key="i"
              :style="{ height: (12 + Math.abs(Math.sin(i * 0.7)) * 70) + 'px' }"
            />
          </div>
          <p v-if="currentMedia.caption" class="media__voice-cap">
            « {{ currentMedia.caption }} »
          </p>
        </div>

        <!-- Overlays sur le média -->
        <div v-if="currentMedia" class="media__timestamp">
          {{ currentMedia.kind === 'photo' ? '📷' : currentMedia.kind === 'clip' ? '🎬' : '🎤' }}
          Posté {{ postedRelative(currentMedia.posted_at) }}
        </div>
        <div v-if="currentMedia && currentMedia.duration_seconds" class="media__duration">
          {{ durationLabel(currentMedia.duration_seconds) }}
        </div>
      </div>

      <div v-if="currentMedia" class="attribution">
        <div class="attribution__who">
          {{ currentMedia.user_pseudo ?? 'Canard inconnu' }}
        </div>
        <div class="attribution__meta">
          {{ postedRelative(currentMedia.posted_at) }}
        </div>
        <div class="attribution__num">
          — Plongée n°{{ currentIdx + 1 }} / {{ items.length }}
        </div>
      </div>
    </section>

    <!-- ─── Teaser band ─────────────────────────────────────── -->
    <div class="teaser">{{ teaserText }}</div>

    <!-- ─── Pond ────────────────────────────────────────────── -->
    <div class="pond-region">
      <Pond
        :ducks="pondDucks"
        :width="pondWidth"
        :height="pondHeight"
        :night-mode="isNight"
      />
    </div>

    <!-- ─── HUD discret (fullscreen + retour, masquable via ?hud=0) ─ -->
    <div v-if="showHud" class="hud">
      <button type="button" class="hud__btn" :title="isFullscreen ? 'Sortir du plein écran' : 'Plein écran'" @click="toggleFullscreen">
        <svg v-if="!isFullscreen" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="4 8 4 4 8 4" />
          <polyline points="20 8 20 4 16 4" />
          <polyline points="20 16 20 20 16 20" />
          <polyline points="4 16 4 20 8 20" />
        </svg>
        <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="8 4 4 4 4 8" />
          <polyline points="16 4 20 4 20 8" />
          <polyline points="16 20 20 20 20 16" />
          <polyline points="8 20 4 20 4 16" />
        </svg>
      </button>
      <RouterLink to="/upload" class="hud__btn" title="Retour">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <line x1="6" y1="6" x2="18" y2="18" />
          <line x1="6" y1="18" x2="18" y2="6" />
        </svg>
      </RouterLink>
    </div>
  </div>
</template>

<style scoped>
.tv {
  width: 100vw;
  height: 100vh;
  display: grid;
  grid-template-rows: 100px minmax(0, 1fr) 40px auto;
  font-family: var(--sans);
  overflow: hidden;
}

/* ─── Day theme ──────────────────────────────────────────── */
.tv--day {
  background: var(--cream);
  color: var(--ink);
}
.tv--day .brand { color: var(--pond-deep); }
.tv--day .brand em { color: var(--duck-deep); }
.tv--day .brand__sub { color: var(--ink-soft); }
.tv--day .pill-big {
  background: var(--pond-deep);
  color: white;
}
.tv--day .attribution__who { color: var(--pond-deep); }
.tv--day .attribution__meta,
.tv--day .attribution__num { color: var(--ink-soft); }
.tv--day .teaser { color: var(--ink-soft); }

/* ─── Night theme ────────────────────────────────────────── */
.tv--night {
  background: var(--pond-deep);
  background-image:
    radial-gradient(900px 500px at 8% 12%, rgba(255, 201, 60, 0.06), transparent 60%),
    radial-gradient(800px 500px at 92% 88%, rgba(232, 199, 122, 0.07), transparent 60%);
  color: white;
}
.tv--night .brand { color: white; }
.tv--night .brand em { color: var(--duck); }
.tv--night .brand__sub { color: #BFD6E0; }
.tv--night .pill-big {
  background: rgba(255, 255, 255, 0.08);
  color: white;
  border: 1.5px solid rgba(255, 255, 255, 0.18);
}
.tv--night .pill-big__num { color: var(--duck); }
.tv--night .attribution__who { color: white; }
.tv--night .attribution__meta,
.tv--night .attribution__num { color: #BFD6E0; }
.tv--night .teaser { color: #BFD6E0; }
.tv--night .media { background: #2A3A45; }
.tv--night .media__empty-text { color: #BFD6E0; }

/* ─── Top bar ────────────────────────────────────────────── */
.topbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 56px 0;
}

.brand {
  font-family: var(--display);
  font-size: clamp(36px, 4vw, 56px);
  line-height: 1;
}
.brand em { font-style: normal; }
.brand__sub {
  display: block;
  margin-top: 6px;
  font-family: var(--mono);
  font-size: clamp(12px, 1vw, 16px);
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.pill-big {
  font-family: var(--display);
  font-size: clamp(36px, 4vw, 56px);
  padding: 16px 28px;
  border-radius: 999px;
  display: inline-flex;
  align-items: baseline;
  gap: 16px;
  line-height: 1;
}
.pill-big__num { font-weight: 600; }
.pill-big__of { opacity: 0.55; font-size: 0.7em; }
.pill-big__label {
  font-family: var(--mono);
  font-size: clamp(14px, 1.2vw, 20px);
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.8;
}

/* ─── Media zone ─────────────────────────────────────────── */
.media-zone {
  display: flex;
  align-items: stretch;
  justify-content: space-between;
  padding: 16px 56px 0;
  gap: 40px;
  min-height: 0; /* permet à 1fr de s'écraser correctement */
}

.media {
  flex: 0 0 auto;
  width: clamp(480px, 48vw, 920px);
  height: 100%;
  border-radius: 28px;
  overflow: hidden;
  position: relative;
  background: #C8CCD0;
  box-shadow: 0 24px 50px -16px rgba(14, 79, 107, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
}

.media__img,
.media__video {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.media__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 18px;
}
.media__empty-text {
  font-family: var(--mono);
  font-size: 18px;
  color: #6B7480;
  letter-spacing: 0.04em;
}

.media__voice {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 24px;
  padding: 32px;
  background: linear-gradient(180deg, var(--pond-pale), #d6ecf3);
  width: 100%;
  height: 100%;
}
.media__voice-wf {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 90px;
}
.media__voice-wf span {
  width: 6px;
  background: var(--pond-deep);
  border-radius: 3px;
}
.media__voice-cap {
  font-family: var(--display);
  font-size: clamp(20px, 2vw, 30px);
  color: var(--pond-deep);
  text-align: center;
  max-width: 70%;
  line-height: 1.3;
  margin: 0;
}

.media__timestamp,
.media__duration {
  position: absolute;
  top: 22px;
  z-index: 2;
  background: rgba(0, 0, 0, 0.45);
  color: white;
  backdrop-filter: blur(8px);
  font-family: var(--mono);
  font-size: 18px;
  letter-spacing: 0.04em;
  padding: 10px 16px;
  border-radius: 999px;
}
.media__timestamp { left: 22px; }
.media__duration { right: 22px; }

/* ─── Attribution ────────────────────────────────────────── */
.attribution {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  justify-content: center;
}
.attribution__who {
  font-family: var(--display);
  font-size: clamp(36px, 4vw, 80px);
  line-height: 1.02;
  text-wrap: balance;
  word-break: break-word;
  overflow-wrap: break-word;
}
.attribution__meta {
  font-family: var(--mono);
  font-size: clamp(16px, 1.4vw, 24px);
  letter-spacing: 0.08em;
  margin-top: 18px;
  text-transform: uppercase;
}
.attribution__num {
  margin-top: 28px;
  font-family: var(--mono);
  font-size: clamp(14px, 1.2vw, 22px);
  letter-spacing: 0.12em;
  text-transform: uppercase;
  opacity: 0.7;
}

/* ─── Teaser band ────────────────────────────────────────── */
.teaser {
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--mono);
  font-size: clamp(14px, 1.4vw, 24px);
  letter-spacing: 0.04em;
  padding: 0 56px;
}

/* ─── Pond region ────────────────────────────────────────── */
.pond-region {
  position: relative;
  overflow: hidden;
}
.pond-region :deep(.pond) {
  border-radius: 0;
  width: 100% !important;
}

/* ─── HUD ────────────────────────────────────────────────── */
.hud {
  position: fixed;
  top: 16px;
  right: 16px;
  display: flex;
  gap: 8px;
  z-index: 100;
  opacity: 0.4;
  transition: opacity 0.2s;
}
.hud:hover { opacity: 1; }

.hud__btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.4);
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  text-decoration: none;
  backdrop-filter: blur(6px);
}
.hud__btn:hover { background: rgba(0, 0, 0, 0.7); }
</style>
