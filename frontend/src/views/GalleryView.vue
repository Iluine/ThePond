<script setup lang="ts">
/**
 * GalleryView — la mare publique avec ses 4 états.
 *
 * Détection à partir du snapshot SSE (cf. PROJECT.md § "Mécanique des
 * paliers" + "États de la galerie") :
 *
 *   - pre-event : aucun palier n'a encore démarré (avant 19h00 le jour J,
 *     ou en environnement de dev sans seed)
 *   - sleepy    : palier en cours mais aucun palier visible (= on est
 *     dans le premier palier, rien n'a encore été révélé)
 *   - awake     : palier visible existe ET il reste des paliers réguliers
 *     à venir → reveal banner + masonry
 *   - nocturnal : tous les paliers réguliers sont passés, on attend le
 *     final reveal du lendemain
 *   - revealed  : phase_current.is_final_reveal === true → tout est
 *     visible, sort chrono/aléatoire dispo
 *
 * Microcopies hardcodées en V1 ; surfacement depuis themes/strings.ron
 * au prompt 15 polish.
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSnapshotStore } from '../stores/snapshot'
import { useUserStore } from '../stores/user'
import Duck from '../components/Duck.vue'
import PondCounter from '../components/PondCounter.vue'
import MediaCard from '../components/MediaCard.vue'
import FilterChips, { type FilterValue } from '../components/FilterChips.vue'
import type { Media } from '../types/snapshot'

const router = useRouter()
const snapshotStore = useSnapshotStore()
const userStore = useUserStore()

onMounted(() => {
  if (!userStore.isAuthenticated) {
    router.replace('/')
  }
})

// ─── État dérivé du snapshot ───────────────────────────────────
const phaseCurrent = computed(() => snapshotStore.phaseCurrent)
const phaseVisible = computed(() => snapshotStore.phaseVisible)
const phasesAll = computed(() => snapshotStore.phasesAll)
const mediaVisible = computed<Media[]>(() => snapshotStore.mediaVisible)

type GalleryState = 'pre-event' | 'sleepy' | 'awake' | 'nocturnal' | 'revealed'

const galleryState = computed<GalleryState>(() => {
  const cur = phaseCurrent.value
  const vis = phaseVisible.value
  const all = phasesAll.value

  if (!cur) return 'pre-event'
  if (cur.is_final_reveal) return 'revealed'
  if (!vis) return 'sleepy'

  // S'il reste un palier régulier (non-final) APRÈS le current, on est
  // en awake. Sinon on attend le final reveal → nocturnal.
  const hasNextRegular = all.some(
    (p) => p.phase_order > cur.phase_order && !p.is_final_reveal,
  )
  return hasNextRegular ? 'awake' : 'nocturnal'
})

// ─── Phase suivante régulière (non final reveal) ──────────────
const nextRegularPhase = computed(() => {
  if (!phaseCurrent.value) return phasesAll.value.find((p) => !p.is_final_reveal)
  return phasesAll.value
    .filter(
      (p) => p.phase_order > phaseCurrent.value!.phase_order && !p.is_final_reveal,
    )
    .sort((a, b) => a.phase_order - b.phase_order)[0]
})

const finalRevealPhase = computed(() =>
  phasesAll.value.find((p) => p.is_final_reveal),
)

// ─── Format helpers ────────────────────────────────────────────
function formatTime(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const h = d.getHours()
  const m = d.getMinutes()
  return m === 0
    ? `${h}h`
    : `${h.toString().padStart(2, '0')}h${m.toString().padStart(2, '0')}`
}

// ─── Header counter ────────────────────────────────────────────
const totalUsers = computed(() => snapshotStore.counts?.total_users ?? 0)
const totalPosts = computed(() => snapshotStore.counts?.total_posts ?? 0)
const expectedGuests = 52

// ─── Filter + sort (états 2 et 4) ──────────────────────────────
const activeFilter = ref<FilterValue>('all')
type SortMode = 'chrono' | 'random'
const sortMode = ref<SortMode>('chrono')

const counts = computed(() => {
  const list = mediaVisible.value
  return {
    all: list.length,
    photo: list.filter((m) => m.kind === 'photo').length,
    clip: list.filter((m) => m.kind === 'clip').length,
    voice: list.filter((m) => m.kind === 'voice').length,
  }
})

const filteredMedia = computed<Media[]>(() => {
  let list = mediaVisible.value
  if (activeFilter.value !== 'all') {
    list = list.filter((m) => m.kind === activeFilter.value)
  }
  if (sortMode.value === 'random' && galleryState.value === 'revealed') {
    // Shuffle stable-ish : Fisher-Yates sur une copie
    const shuffled = [...list]
    for (let i = shuffled.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1))
      ;[shuffled[i], shuffled[j]] = [shuffled[j], shuffled[i]]
    }
    return shuffled
  }
  return list
})

// ─── État 1+3 messages ─────────────────────────────────────────
const sleepyMessage = computed(() => {
  // "Les premiers barbotages se révèleront avec {next} vers {time}"
  const n = nextRegularPhase.value
  if (!n) return 'En attente du prochain palier'
  return `Les premiers barbotages se révèleront avec ${n.name.toLowerCase()} vers ${formatTime(n.target_time)}`
})

const sleepySubMeta = computed(() => {
  const n = mediaVisible.value.length + (snapshotStore.counts?.posts_pending ?? 0)
  if (n === 0) return 'Pas encore de souvenirs en route'
  if (n === 1) return '1 souvenir barbote dans la mare'
  return `${n} souvenirs barbotent dans la mare`
})

const nocturnalMessage =
  'Plus assez de lumière aujourd’hui · on y verra mieux demain pour les autres barbotages'

const nocturnalSubMeta = computed(() => {
  const f = finalRevealPhase.value
  if (!f) return ''
  return `Les canards seront confits ${
    f.target_time.includes('+1') ? 'demain' : ''
  } vers ${formatTime(f.target_time)}`
})

// ─── État 2 reveal banner ──────────────────────────────────────
const revealBannerText = computed(() => {
  const cur = phaseCurrent.value
  const next = nextRegularPhase.value
  if (!cur || !next) return ''
  return `Les souvenirs du ${cur.name.toLowerCase()} se révèleront avec ${next.name.toLowerCase()} vers ${formatTime(
    next.target_time,
  )}`
})

const arrivedTitle = computed(() => {
  const vis = phaseVisible.value
  if (!vis) return 'La mare'
  return `${vis.name} vient d’arriver`
})

const arrivedSub = computed(() => {
  const n = mediaVisible.value.length
  return `${n} ${n === 1 ? 'souvenir' : 'souvenirs'}`
})

// ─── État 4 (révélation) ──────────────────────────────────────
const revealedSub = computed(() => {
  const n = mediaVisible.value.length
  return `${n} ${n === 1 ? 'souvenir' : 'souvenirs'} · dégustons maintenant…`
})
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="flex items-center justify-between px-5 pt-4 pb-1.5">
      <div class="font-sans font-bold text-lg text-pond-deep">
        The <span class="text-duck-deep">Pond</span>.
      </div>
      <PondCounter
        :current="totalUsers"
        :total="expectedGuests"
        :class="{ 'pond-counter-full': galleryState === 'revealed' }"
      />
    </header>

    <!-- ═══ pre-event ═══════════════════════════════════════ -->
    <template v-if="galleryState === 'pre-event'">
      <div class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-4">
        <Duck color="yellow" :size="100" asleep />
        <p class="font-display text-2xl text-pond-deep">La mare dort encore</p>
        <p class="font-sans text-sm text-ink-soft max-w-xs">
          Les paliers ne sont pas encore configurés.
          {{ phasesAll.length === 0 ? 'Les témoins doivent les ouvrir.' : '' }}
        </p>
      </div>
    </template>

    <!-- ═══ ÉTAT 1 : sleepy ════════════════════════════════ -->
    <template v-else-if="galleryState === 'sleepy'">
      <div class="px-5 pt-2 pb-1.5">
        <h2 class="font-display text-2xl text-pond-deep leading-none">La mare</h2>
        <p class="font-sans text-xs text-ink-soft mt-1">Souvenirs en attente de révélation</p>
      </div>
      <div class="stage stage--day mx-4 mt-2">
        <div class="lily-pad" />
        <div class="duck-sleep">
          <Duck color="yellow" :size="100" asleep />
        </div>
        <div class="zzz z1">z</div>
        <div class="zzz z2">z</div>
        <div class="zzz z3">z</div>
        <div class="water" />
      </div>
      <div class="info-card mx-4 mt-3">
        <p class="info-body">{{ sleepyMessage }}</p>
        <div class="info-meta">{{ sleepySubMeta }}</div>
      </div>
      <div class="text-center mt-auto pb-6 pt-6">
        <RouterLink to="/upload" class="back-link">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="15 6 9 12 15 18" />
          </svg>
          Retour
        </RouterLink>
      </div>
    </template>

    <!-- ═══ ÉTAT 2 : awake ═════════════════════════════════ -->
    <template v-else-if="galleryState === 'awake'">
      <div class="px-5 pt-2 pb-2">
        <h2 class="font-display text-2xl text-pond-deep leading-none">{{ arrivedTitle }}</h2>
        <p class="font-sans text-xs text-ink-soft mt-1">{{ arrivedSub }}</p>
      </div>
      <div class="reveal-banner mx-4 mt-1">
        <div class="moon-icon" />
        <div class="reveal-text">{{ revealBannerText }}</div>
      </div>
      <div class="px-4 mt-3">
        <FilterChips v-model="activeFilter" :counts="counts" />
      </div>
      <div v-if="filteredMedia.length > 0" class="grid-mason mx-4 mt-3 mb-6">
        <MediaCard v-for="m in filteredMedia" :key="m.id" :media="m" />
      </div>
      <p
        v-else
        class="font-mono text-xs text-ink-soft text-center mx-4 mt-3 mb-6 py-8"
      >
        Pas encore de souvenirs dans cette catégorie.
      </p>
    </template>

    <!-- ═══ ÉTAT 3 : nocturnal ═════════════════════════════ -->
    <template v-else-if="galleryState === 'nocturnal'">
      <div class="px-5 pt-2 pb-1.5">
        <h2 class="font-display text-2xl text-pond-deep leading-none">La mare</h2>
        <p class="font-sans text-xs text-ink-soft mt-1">la mare se ferme</p>
      </div>
      <div class="stage stage--night mx-4 mt-2">
        <div class="moon" />
        <div class="stars">
          <span style="top: 20%; left: 18%" />
          <span style="top: 14%; left: 32%; opacity: 0.4" />
          <span style="top: 30%; left: 48%" />
          <span style="top: 18%; left: 62%; opacity: 0.5" />
          <span style="top: 36%; left: 78%" />
          <span style="top: 46%; left: 22%; opacity: 0.4" />
        </div>
        <div class="duck-sleep">
          <Duck color="yellow" :size="100" asleep />
        </div>
        <div class="zzz z1 zzz--night">z</div>
        <div class="zzz z2 zzz--night">z</div>
        <div class="zzz z3 zzz--night">z</div>
        <div class="water water--night" />
      </div>
      <div class="info-card mx-4 mt-3">
        <p class="info-body">{{ nocturnalMessage }}</p>
        <div class="info-meta">{{ nocturnalSubMeta }}</div>
      </div>
      <div class="text-center mt-auto pb-6 pt-6">
        <RouterLink to="/upload" class="back-link">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
            <polyline points="15 6 9 12 15 18" />
          </svg>
          Retour
        </RouterLink>
      </div>
    </template>

    <!-- ═══ ÉTAT 4 : revealed ══════════════════════════════ -->
    <template v-else>
      <div class="px-5 pt-2 pb-2">
        <h2 class="font-display text-2xl text-pond-deep leading-none">
          Les canards<br />sont confits
        </h2>
        <p class="font-sans text-xs text-ink-soft mt-1">{{ revealedSub }}</p>
      </div>
      <div class="px-4 mt-3">
        <FilterChips v-model="activeFilter" :counts="counts" />
      </div>
      <div class="px-4 mt-2 flex items-center justify-between font-mono text-[9px] uppercase tracking-wider text-ink-soft">
        <span>Tri</span>
        <div class="flex gap-1">
          <button
            type="button"
            class="t-pill"
            :class="{ 't-pill--active': sortMode === 'chrono' }"
            @click="sortMode = 'chrono'"
          >
            Chrono
          </button>
          <button
            type="button"
            class="t-pill"
            :class="{ 't-pill--active': sortMode === 'random' }"
            @click="sortMode = 'random'"
          >
            Aléatoire
          </button>
        </div>
      </div>
      <div class="grid-mason mx-4 mt-3 mb-2">
        <MediaCard v-for="m in filteredMedia" :key="m.id" :media="m" />
      </div>
      <p
        v-if="totalPosts > filteredMedia.length"
        class="font-mono text-[9px] text-ink-soft text-center pb-6"
      >
        ↓ continue à scroller pour découvrir les {{ totalPosts }} souvenirs
      </p>
    </template>
  </main>
</template>

<style scoped>
.back-link {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--sans);
  font-size: 13px;
  color: var(--pond-mid);
  text-decoration: none;
}
.back-link:hover {
  color: var(--pond-deep);
}

/* ─── Stage (state 1 + 3) ────────────────────────────────────── */
.stage {
  height: 240px;
  border-radius: 20px;
  position: relative;
  overflow: hidden;
}

.stage--day {
  background: linear-gradient(180deg, var(--pond-pale) 0%, #d6ecf3 100%);
  border: 1px solid rgba(169, 216, 229, 0.5);
}

.stage--night {
  background: linear-gradient(180deg, #c4d8e0 0%, #87a8b9 100%);
  border: 1px solid rgba(95, 122, 138, 0.5);
}

.stage .water {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  height: 60px;
  background: linear-gradient(180deg, var(--pond-light) 0%, var(--pond-mid) 100%);
}
.stage .water--night {
  background: linear-gradient(180deg, #6a90a4 0%, #2c4f63 100%);
}

.lily-pad {
  position: absolute;
  bottom: 38px;
  left: 50%;
  transform: translateX(-50%);
  width: 96px;
  height: 14px;
  background: #4fa86b;
  border-radius: 50%;
  opacity: 0.85;
  box-shadow:
    -50px 14px 0 -2px rgba(79, 168, 107, 0.7),
    56px 18px 0 -3px rgba(79, 168, 107, 0.6);
}
.lily-pad::before {
  content: '';
  position: absolute;
  top: -2px;
  left: 50%;
  transform: translateX(-50%);
  width: 90px;
  height: 5px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.25);
}

.duck-sleep {
  position: absolute;
  left: 50%;
  bottom: 46px;
  transform: translateX(-50%);
  z-index: 2;
  animation: breathe 3.6s ease-in-out infinite;
}
@keyframes breathe {
  0%, 100% { transform: translateX(-50%) scale(1); }
  50%     { transform: translateX(-50%) scale(1.04); }
}

.zzz {
  position: absolute;
  font-family: var(--display);
  color: var(--pond-mid);
  opacity: 0.55;
}
.zzz.z1 { top: 24%; left: 60%; font-size: 18px; transform: rotate(-8deg); }
.zzz.z2 { top: 14%; left: 68%; font-size: 14px; transform: rotate(-5deg); opacity: 0.4; }
.zzz.z3 { top: 6%;  left: 74%; font-size: 11px; transform: rotate(-3deg); opacity: 0.3; }
.zzz--night { color: var(--pond-mid); }

.moon {
  position: absolute;
  top: 18px;
  right: 24px;
  width: 34px;
  height: 34px;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 35%, #f0eaca 0 35%, #d9c98a 36% 100%);
  box-shadow: inset -3px -3px 5px rgba(120, 90, 30, 0.4), 0 0 22px rgba(255, 245, 200, 0.25);
  z-index: 1;
}

.stars {
  position: absolute;
  inset: 0 0 60% 0;
  pointer-events: none;
}
.stars span {
  position: absolute;
  width: 2px;
  height: 2px;
  border-radius: 50%;
  background: #fff;
  opacity: 0.6;
}

/* ─── Info card ──────────────────────────────────────────────── */
.info-card {
  padding: 12px 14px;
  background: var(--cream-deep);
  border: 1px solid var(--cream-line);
  border-radius: 14px;
}
.info-body {
  margin: 0;
  font-family: var(--sans);
  font-size: 13px;
  color: var(--pond-deep);
  line-height: 1.4;
}
.info-meta {
  margin-top: 8px;
  font-family: var(--mono);
  font-size: 10px;
  color: var(--ink-soft);
  letter-spacing: 0.04em;
  display: flex;
  align-items: center;
  gap: 6px;
}
.info-meta::before {
  content: '';
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--pond-mid);
  flex-shrink: 0;
}

/* ─── Reveal banner (state 2) ────────────────────────────────── */
.reveal-banner {
  padding: 9px 11px;
  display: flex;
  align-items: center;
  gap: 9px;
  background: linear-gradient(135deg, var(--champagne-pale) 0%, #fff6dc 100%);
  border: 1px solid var(--champagne);
  border-radius: 13px;
  font-family: var(--sans);
  font-size: 11px;
  color: var(--ink);
  line-height: 1.35;
}
.moon-icon {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 35%, #fff5d0 0 30%, var(--champagne) 32% 100%);
  box-shadow: inset -3px -3px 4px rgba(201, 163, 71, 0.4);
  position: relative;
}
.moon-icon::before {
  content: '';
  position: absolute;
  top: 5px;
  right: 4px;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--champagne-pale);
}
.reveal-text {
  flex: 1;
}

/* ─── Sort row (state 4) ─────────────────────────────────────── */
.t-pill {
  padding: 3px 7px;
  border-radius: 8px;
  border: 1px solid var(--cream-line);
  background: transparent;
  cursor: pointer;
  font-family: inherit;
  font-size: inherit;
  letter-spacing: inherit;
  color: var(--ink-soft);
}
.t-pill--active {
  background: var(--cream-deep);
  color: var(--pond-deep);
  border-color: var(--cream-line);
}

/* ─── Masonry grid ───────────────────────────────────────────── */
.grid-mason {
  column-count: 2;
  column-gap: 7px;
}
</style>
