<script setup lang="ts">
/**
 * ConfirmationView — écran post-upload "Splash !".
 *
 * Reproduit design/Confirmation.html (variante photo validée pixel-correct)
 * avec dérivation des variantes clip et voice. Le composant lit l'itemId
 * dans la query, suit le statut depuis useUploadQueueStore et bascule
 * entre uploading → done → failed.
 *
 * Optimistic UI : on est mounté dès que l'upload commence côté Upload/
 * Capture views. Pendant le upload on affiche déjà toute la confirmation
 * (avec un thumbnail object-URL local), au done on switch sur l'URL
 * server, au failed on offre un retry.
 *
 * Microcopies figées de PROJECT.md § "Microcopy figée" — hardcodées en
 * V1, à surfacer depuis strings.ron au prompt 15 polish.
 */
import { ref, computed, watch, onBeforeUnmount, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useUploadQueueStore, type UploadType } from '../stores/uploadQueue'
import { useUserStore } from '../stores/user'
import { useSnapshotStore } from '../stores/snapshot'
import Duck from '../components/Duck.vue'
import PondCounter from '../components/PondCounter.vue'
import PrimaryButton from '../components/PrimaryButton.vue'
import MareTVPill from '../components/MareTVPill.vue'
import type { DuckColor } from '../types/duck'

const route = useRoute()
const router = useRouter()
const uploadQueue = useUploadQueueStore()
const userStore = useUserStore()
const snapshotStore = useSnapshotStore()

// ─── Param parsing ──────────────────────────────────────────────
const itemId = computed(() => (route.query.itemId as string) || '')
const typeFromQuery = computed<UploadType | null>(() => {
  const t = route.query.type as string
  if (t === 'photo' || t === 'clip' || t === 'voice') return t
  return null
})

const item = computed(() =>
  itemId.value ? uploadQueue.getById(itemId.value) : undefined,
)
const type = computed<UploadType>(
  () => item.value?.type ?? typeFromQuery.value ?? 'photo',
)
const status = computed(() => item.value?.status ?? null)

// ─── Object URL pour le preview local pendant upload ───────────
// On le crée dès qu'on a accès au File et on le révoque à l'unmount
// ou au passage à 'done' (l'URL server prend le relais).
const localObjectUrl = ref<string | null>(null)

function refreshLocalUrl() {
  if (!itemId.value) return
  const file = uploadQueue.getFile(itemId.value)
  if (!file) return
  // Photos seulement : object-url visuel utile. Clips/voice n'ont pas
  // besoin (le clip placeholder server-side suffit).
  if (type.value !== 'photo') return
  if (localObjectUrl.value) return
  localObjectUrl.value = URL.createObjectURL(file)
}

watch(itemId, refreshLocalUrl, { immediate: true })

onBeforeUnmount(() => {
  if (localObjectUrl.value) {
    URL.revokeObjectURL(localObjectUrl.value)
    localObjectUrl.value = null
  }
})

// ─── Thumbnail à afficher ──────────────────────────────────────
const thumbUrl = computed(() => {
  if (status.value === 'done' && item.value?.serverResponse?.thumb_filename) {
    return `/uploads/${item.value.serverResponse.thumb_filename}`
  }
  return localObjectUrl.value
})

// ─── Microcopies par type ──────────────────────────────────────
const variants = {
  photo: {
    confirmText: 'Ta photo a rejoint la mare',
    previewLabel: 'Ta photo',
    ctaLabel: 'CONTINUER À BARBOTER',
    iconKind: 'photo' as const,
  },
  clip: {
    confirmText: 'Ton clip a rejoint la mare',
    previewLabel: 'Ton clip',
    ctaLabel: 'CONTINUER À CANCANER',
    iconKind: 'clip' as const,
  },
  voice: {
    confirmText: 'Ton coin-coin a rejoint la mare',
    previewLabel: 'Ton coin-coin',
    ctaLabel: 'CONTINUER À COIN-COIN',
    iconKind: 'voice' as const,
  },
} as const
const variant = computed(() => variants[type.value])

// ─── Counter (avec animation bumpIn déclenchée au mount) ──────
const totalUsers = computed(() => snapshotStore.counts?.total_users ?? 0)
const expectedGuests = 52

// ─── Couleur du canard utilisateur (pour Frame 3 + greeting) ──
const userColor = computed<DuckColor>(
  () => (userStore.user?.duck_color as DuckColor) ?? 'yellow',
)

// ─── Garde : sans itemId on offre un retour propre ────────────
const isOrphan = computed(() => !itemId.value || !item.value)

onMounted(() => {
  // Si pas d'itemId, on attend juste — l'utilisateur reste sur cette
  // page jusqu'à click manuel.
})

// ─── Actions ───────────────────────────────────────────────────
function continueUpload() {
  router.push('/upload')
}

function retry() {
  if (!itemId.value) return
  uploadQueue.retry(itemId.value)
}

function dismiss() {
  if (itemId.value) uploadQueue.dismiss(itemId.value)
  router.push('/upload')
}

// Pour la Frame 3 — quelques canards "spectateurs" décoratifs
const spectatorColors: DuckColor[] = ['yellow', 'white', 'yellow', 'blue', 'yellow']
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Empty state si pas d'itemId ─────────────────────── -->
    <template v-if="isOrphan">
      <header class="flex items-center justify-between px-5 pt-4 pb-1.5">
        <div class="font-sans font-bold text-lg text-pond-deep">
          The <span class="text-duck-deep">Pond</span>.
        </div>
        <PondCounter :current="totalUsers" :total="expectedGuests" />
      </header>
      <div class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-4">
        <p class="font-display text-3xl text-pond-deep">Rien à confirmer</p>
        <p class="font-sans text-sm text-ink-soft max-w-xs">
          Pas d'upload en cours sur ce lien. Retour à la mare.
        </p>
        <PrimaryButton class="!w-auto" @click="continueUpload">
          BARBOTER
        </PrimaryButton>
      </div>
    </template>

    <template v-else>
      <!-- ─── Header ────────────────────────────────────────── -->
      <header class="flex items-center justify-between px-5 pt-4 pb-1.5">
        <div class="font-sans font-bold text-lg text-pond-deep">
          The <span class="text-duck-deep">Pond</span>.
        </div>
        <div class="bumpin">
          <PondCounter :current="totalUsers" :total="expectedGuests" />
        </div>
      </header>

      <!-- ─── 3-frame comic strip ───────────────────────────── -->
      <section class="strip mx-4 mt-4">
        <!-- Frame 1 : duck mid-air + "Splash !" word -->
        <div class="frame f1">
          <div class="num-tag">01</div>
          <div class="speed-line" />
          <div class="speed-line s2" />
          <div class="duck-air">
            <Duck :color="userColor" :size="60" />
          </div>
          <div class="splash-word">Splash !</div>
          <div class="water" />
        </div>

        <div class="strip-arrow">
          <svg width="14" height="6" viewBox="0 0 14 6">
            <line x1="0" y1="3" x2="10" y2="3" stroke="currentColor" stroke-width="1.5" stroke-dasharray="1.5 1.8" />
            <polyline points="9,1 13,3 9,5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>

        <!-- Frame 2 : ripples + droplets + duck head -->
        <div class="frame f2">
          <div class="num-tag">02</div>
          <div class="droplet d1" />
          <div class="droplet d2" />
          <div class="droplet d3" />
          <div class="droplet d4" />
          <div class="droplet d5" />
          <div class="water" />
          <div class="ripple r3" />
          <div class="ripple r2" />
          <div class="ripple r1" />
          <div class="head"><div class="eye" /></div>
        </div>

        <div class="strip-arrow">
          <svg width="14" height="6" viewBox="0 0 14 6">
            <line x1="0" y1="3" x2="10" y2="3" stroke="currentColor" stroke-width="1.5" stroke-dasharray="1.5 1.8" />
            <polyline points="9,1 13,3 9,5" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>

        <!-- Frame 3 : ducks dans la mare avec le user en avant -->
        <div class="frame f3">
          <div class="num-tag">03</div>
          <div class="water" />
          <div
            v-for="(c, i) in spectatorColors"
            :key="i"
            class="pond-spectator"
            :style="{
              left: ([20, 78, 18, 82, 50][i] ?? 50) + '%',
              top: ([48, 50, 78, 80, 88][i] ?? 70) + '%',
            }"
          >
            <Duck :color="c" :size="[18, 20, 16, 18, 14][i] ?? 16" />
          </div>
          <div class="me-duck" aria-label="ton canard">
            <div class="me-duck__halo" />
            <Duck :color="userColor" :size="38" />
            <div class="me-tag">TOI</div>
          </div>
        </div>
      </section>

      <!-- ─── Confirmation message ──────────────────────────── -->
      <div class="text-center px-6 pt-3.5 pb-2">
        <h1 class="font-display text-pond-deep leading-none text-4xl">
          Splash<span class="text-duck-deep ml-0.5">&nbsp;!</span>
        </h1>
        <p class="font-sans text-base text-ink-soft mt-2 leading-tight">
          {{ variant.confirmText }}
        </p>
      </div>

      <!-- ─── Preview card ──────────────────────────────────── -->
      <div class="mx-4 mt-3.5 px-3.5 pt-3.5 pb-3 bg-cream border border-cream-line rounded-2xl flex items-center gap-3.5"
           style="box-shadow: 0 6px 18px -10px rgba(14,79,107,.18);">
        <div class="thumb">
          <img
            v-if="thumbUrl && type !== 'voice'"
            :src="thumbUrl"
            class="thumb__img"
            alt="aperçu"
          />
          <span v-else class="thumb__placeholder">
            <svg
              v-if="variant.iconKind === 'photo'"
              width="28" height="28" viewBox="0 0 24 24"
              fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"
            >
              <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
              <circle cx="12" cy="13" r="4" />
            </svg>
            <svg
              v-else-if="variant.iconKind === 'clip'"
              width="28" height="28" viewBox="0 0 24 24"
              fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"
            >
              <rect x="2" y="6" width="14" height="12" rx="2" />
              <path d="M22 8 L16 12 L22 16 Z" />
            </svg>
            <svg
              v-else
              width="28" height="28" viewBox="0 0 24 24"
              fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"
            >
              <rect x="9" y="2" width="6" height="12" rx="3" />
              <path d="M5 11a7 7 0 0 0 14 0" />
              <line x1="12" y1="18" x2="12" y2="22" />
            </svg>
          </span>
        </div>
        <div class="flex-1 min-w-0">
          <div class="font-sans font-semibold text-sm text-ink mb-1">
            {{ variant.previewLabel }}
          </div>
          <div class="font-mono text-[11px] text-ink-soft tracking-wider flex items-center gap-1.5">
            <template v-if="status === 'uploading' || status === 'pending'">
              <span class="w-1.5 h-1.5 rounded-full bg-pond-mid animate-pulse" />
              envoi…
            </template>
            <template v-else-if="status === 'done'">
              <span class="w-1.5 h-1.5 rounded-full bg-green" />
              barbote dans la mare
            </template>
            <template v-else-if="status === 'failed'">
              <span class="w-1.5 h-1.5 rounded-full bg-coral-deep" />
              échec · {{ item?.errorMessage ?? 'erreur' }}
            </template>
            <template v-else-if="status === 'lost'">
              <span class="w-1.5 h-1.5 rounded-full bg-coral-deep" />
              fichier perdu (page rechargée)
            </template>
          </div>
        </div>
      </div>

      <!-- ─── Primary CTA + retry ─────────────────────────── -->
      <div class="mx-4 mt-4 mb-2">
        <PrimaryButton
          v-if="status !== 'failed' && status !== 'lost'"
          size="lg"
          @click="continueUpload"
        >
          <template #icon>
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
              <circle cx="12" cy="13" r="4" />
            </svg>
          </template>
          {{ variant.ctaLabel }}
        </PrimaryButton>

        <div v-else class="space-y-2">
          <PrimaryButton
            v-if="status === 'failed'"
            size="lg"
            @click="retry"
          >
            Réessayer l'envoi
          </PrimaryButton>
          <button
            type="button"
            class="w-full text-center font-sans text-sm text-ink-soft underline underline-offset-4 py-2"
            @click="dismiss"
          >
            {{ status === 'lost' ? 'Repartir à la mare' : 'Annuler cet envoi' }}
          </button>
        </div>
      </div>

      <!-- ─── Secondary ─────────────────────────────────────── -->
      <div class="text-center pb-6 pt-1 font-sans text-sm text-pond-mid">
        <button
          type="button"
          class="hover:text-pond-deep underline underline-offset-4"
          @click="continueUpload"
        >
          Faire autre chose
        </button>
        <span class="text-pond-light mx-2.5 font-bold">·</span>
        <MareTVPill class="!text-pond-mid hover:!text-pond-deep" />
      </div>
    </template>
  </main>
</template>

<style scoped>
/* ─── Counter bump-in animation ───────────────────────────────── */
@keyframes bump-in {
  0%   { transform: scale(0.8); opacity: 0; }
  60%  { transform: scale(1.12); opacity: 1; }
  100% { transform: scale(1); }
}
.bumpin {
  animation: bump-in 600ms cubic-bezier(0.2, 0.9, 0.3, 1.4) both 100ms;
}

/* ─── Comic strip — pieces from design/Confirmation.html ────── */
.strip {
  background: linear-gradient(180deg, var(--pond-pale) 0%, #EAF7FB 100%);
  border-radius: 22px;
  border: 1px solid rgba(169, 216, 229, 0.6);
  padding: 18px 12px 16px;
  display: flex;
  align-items: stretch;
  justify-content: space-between;
  position: relative;
}

.frame {
  flex: 1;
  height: 230px;
  border-radius: 14px;
  background: rgba(255, 255, 255, 0.5);
  backdrop-filter: blur(4px);
  border: 1px solid rgba(169, 216, 229, 0.4);
  position: relative;
  overflow: hidden;
}

.num-tag {
  position: absolute;
  top: 8px; left: 8px;
  font-family: var(--mono);
  font-size: 9px;
  letter-spacing: 0.16em;
  color: var(--pond-mid);
  text-transform: uppercase;
  background: rgba(255, 255, 255, 0.7);
  padding: 2px 6px;
  border-radius: 6px;
}

.strip-arrow {
  width: 22px;
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  color: var(--pond-mid);
}
.strip-arrow svg { display: block; }

/* ─── Frame 1 : duck mid-air ─────────────────────────────────── */
.f1 .duck-air {
  position: absolute;
  top: 38px; left: 46%;
  transform: translateX(-50%) rotate(14deg);
  filter: drop-shadow(0 8px 8px rgba(14, 79, 107, 0.25));
}
.f1 .splash-word {
  position: absolute;
  top: 80px; right: 8px;
  font-family: var(--display);
  font-size: 18px;
  color: var(--pond-deep);
  transform: rotate(-8deg);
}
.f1 .water {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 46px;
  background: linear-gradient(180deg, var(--pond-light) 0%, var(--pond-mid) 100%);
}
.f1 .water::before {
  content: '';
  position: absolute;
  top: -3px; left: 0; right: 0;
  height: 6px;
  background:
    radial-gradient(circle at 12% 0, rgba(255, 255, 255, 0.7) 0 1.2px, transparent 1.5px),
    radial-gradient(circle at 38% 50%, rgba(255, 255, 255, 0.5) 0 1.5px, transparent 2px),
    radial-gradient(circle at 70% 0, rgba(255, 255, 255, 0.6) 0 1.2px, transparent 1.5px),
    radial-gradient(circle at 90% 50%, rgba(255, 255, 255, 0.5) 0 1.5px, transparent 2px);
}
.f1 .speed-line {
  position: absolute;
  top: 108px; left: 30%;
  width: 2px; height: 18px;
  background: var(--pond-mid);
  opacity: 0.4;
  transform: rotate(-25deg);
  border-radius: 1px;
}
.f1 .speed-line.s2 { left: 55%; top: 120px; height: 14px; opacity: 0.3; }

/* ─── Frame 2 : impact + ripples ─────────────────────────────── */
.f2 .water {
  position: absolute;
  bottom: 0; left: 0; right: 0;
  height: 80px;
  background: linear-gradient(180deg, var(--pond-light) 0%, var(--pond-mid) 100%);
}
.f2 .ripple {
  position: absolute;
  left: 50%; bottom: 50px;
  transform: translate(-50%, 50%);
  border-radius: 50%;
  border: 2px solid var(--pond-pale);
  opacity: 0.6;
}
.f2 .r1 { width: 50px; height: 14px; opacity: 0.85; }
.f2 .r2 { width: 78px; height: 22px; opacity: 0.55; }
.f2 .r3 { width: 108px; height: 32px; opacity: 0.3; }

.f2 .head {
  position: absolute;
  left: 50%; bottom: 54px;
  transform: translate(-50%, 0);
  width: 24px; height: 14px;
  background: var(--duck);
  border: 1.5px solid var(--pond-deep);
  border-radius: 50% 50% 0 0 / 100% 100% 0 0;
  border-bottom: none;
}
.f2 .head .eye {
  position: absolute;
  top: 5px; left: 14px;
  width: 3px; height: 3px;
  border-radius: 50%;
  background: var(--ink);
}
.f2 .droplet {
  position: absolute;
  width: 5px; height: 7px;
  background: var(--pond-mid);
  border-radius: 50% 50% 50% 50% / 60% 60% 40% 40%;
}
.f2 .d1 { bottom: 96px;  left: 24%; transform: rotate(-35deg); }
.f2 .d2 { bottom: 118px; left: 38%; transform: rotate(-15deg) scale(1.1); }
.f2 .d3 { bottom: 128px; left: 52%; transform: translateX(-50%) scale(1.2); }
.f2 .d4 { bottom: 118px; right: 32%; transform: rotate(15deg) scale(1.1); }
.f2 .d5 { bottom: 96px;  right: 22%; transform: rotate(35deg); }

/* ─── Frame 3 : surfaced + spectator ducks ──────────────────── */
.f3 .water {
  position: absolute;
  bottom: 0; left: 0; right: 0; top: 34%;
  background: linear-gradient(180deg, var(--pond-light) 0%, var(--pond-mid) 100%);
}
.f3 .water::before {
  /* lily pads */
  content: '';
  position: absolute;
  top: 18%; left: 8%;
  width: 14px; height: 6px;
  background: #4FA86B;
  border-radius: 50%;
  opacity: 0.7;
  box-shadow: 52px 24px 0 0 #4FA86B, -2px 30px 0 -1px #4FA86B;
}

.pond-spectator {
  position: absolute;
  transform: translate(-50%, -50%);
}

.me-duck {
  position: absolute;
  left: 50%; top: 64%;
  transform: translate(-50%, -50%);
  z-index: 20;
  filter: drop-shadow(0 4px 6px rgba(14, 79, 107, 0.3));
}
.me-duck__halo {
  position: absolute;
  left: 50%; top: 62%;
  transform: translate(-50%, -50%);
  width: 54px; height: 18px;
  border-radius: 50%;
  border: 2px solid var(--champagne-deep);
  background: radial-gradient(closest-side, rgba(255, 255, 255, 0.5), transparent 80%);
  z-index: -1;
}
.me-tag {
  position: absolute;
  left: 100%; top: 0;
  transform: translate(2px, -4px);
  font-family: var(--mono);
  font-size: 8px;
  font-weight: 500;
  letter-spacing: 0.14em;
  color: var(--champagne-deep);
  background: rgba(250, 243, 227, 0.95);
  padding: 1px 5px;
  border-radius: 5px;
  border: 1px solid var(--champagne-deep);
  white-space: nowrap;
}

/* ─── Preview thumb ──────────────────────────────────────────── */
.thumb {
  width: 64px;
  height: 64px;
  flex-shrink: 0;
  border-radius: 12px;
  background: #C8CCD0;
  border: 1px solid #B6BBC2;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  overflow: hidden;
}
.thumb__img {
  width: 100%; height: 100%;
  object-fit: cover;
}
.thumb__placeholder {
  display: flex; align-items: center; justify-content: center;
}
</style>
