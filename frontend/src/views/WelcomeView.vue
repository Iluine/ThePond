<script setup lang="ts">
/**
 * WelcomeView — point d'entrée de l'app pour un invité (post-scan QR).
 *
 * Reproduction de design/Welcome v2.html avec :
 *   - Header "The Pond." + nom de l'événement
 *   - Mini-mare avec couple + canards aléatoires
 *   - Carte de génération : Duck dynamique + pseudo serveur + reroll
 *   - Champ custom_name optionnel
 *   - CTA "PLONGER DANS LA MARE"
 *   - Zone install PWA
 *
 * Le pseudo est servi par GET /api/pseudo (chaque reroll = un fetch).
 * Sur PLONGER, register() envoie le pseudo + la couleur choisis ;
 * le backend persiste tel quel (cf. routes/users.rs).
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Duck from '../components/Duck.vue'
import Pond, { type PondDuck } from '../components/Pond.vue'
import PrimaryButton from '../components/PrimaryButton.vue'
import { useUserStore } from '../stores/user'
import { useSnapshotStore } from '../stores/snapshot'
import { useInstallPrompt } from '../composables/useInstallPrompt'
import type { DuckColor } from '../types/duck'

const router = useRouter()
const userStore = useUserStore()
const snapshotStore = useSnapshotStore()
const { canInstall, install, isIosLike } = useInstallPrompt()

// ─── État du canard en cours de génération ──────────────────────
const pseudo = ref('')
const color = ref<DuckColor>('yellow')
const customName = ref('')

// Distribution v0.2 du SVG canard (PROJECT.md : 80/15/4/1).
function pickColor(): DuckColor {
  const r = Math.random()
  if (r < 0.80) return 'yellow'
  if (r < 0.95) return 'white'
  if (r < 0.99) return 'blue'
  return 'rainbow'
}

const pseudoLoading = ref(false)
const pseudoError = ref<string | null>(null)

async function reroll() {
  pseudoError.value = null
  pseudoLoading.value = true
  // La couleur est tirée localement, pas besoin de réseau pour ça
  color.value = pickColor()
  try {
    const res = await fetch('/api/pseudo')
    if (!res.ok) throw new Error(`HTTP ${res.status}`)
    const data = (await res.json()) as { pseudo: string }
    pseudo.value = data.pseudo
  } catch (err) {
    pseudoError.value = err instanceof Error ? err.message : String(err)
  } finally {
    pseudoLoading.value = false
  }
}

// Si déjà authentifié, on saute Welcome et on file directement à l'upload.
// Sinon on charge un premier pseudo.
onMounted(async () => {
  if (userStore.isAuthenticated) {
    router.replace('/upload')
    return
  }
  await reroll()
})

// Pseudo Caprasimo adaptatif (maquette : 24 / 20 / 17px selon longueur).
const pseudoSizeClass = computed(() => {
  const len = pseudo.value.length
  if (len > 26) return 'text-[17px] tracking-tight'
  if (len > 22) return 'text-[20px]'
  return 'text-2xl'
})

// ─── PLONGER ───────────────────────────────────────────────────
const registering = ref(false)
const registerError = ref<string | null>(null)

async function plonger() {
  if (!pseudo.value || registering.value) return
  registerError.value = null
  registering.value = true
  try {
    await userStore.register({
      duck_color: color.value,
      pseudo: pseudo.value,
      custom_name: customName.value.trim() || undefined,
    })
    router.push('/upload')
  } catch (err) {
    registerError.value = err instanceof Error ? err.message : String(err)
  } finally {
    registering.value = false
  }
}

// ─── Mini-mare décorative ──────────────────────────────────────
// 15 ducks générés une fois pour la durée de la vue, count réel
// affiché depuis le snapshot SSE.
const teaserDucks: PondDuck[] = Array.from({ length: 15 }, (_, i) => ({
  id: i,
  color: pickColor(),
}))
const guestCount = computed(() => snapshotStore.counts?.total_users ?? 0)

// TODO : surfacer event_name depuis le snapshot ou un endpoint dédié
//        (actuellement seul /health l'expose). Hardcodé en attendant.
const eventName = 'Le mariage de Marie & Thomas'

// ─── Install PWA ───────────────────────────────────────────────
const installResult = ref<string | null>(null)
async function onInstall() {
  const outcome = await install()
  if (outcome === 'ios') {
    // TODO prompt 14 : ouvrir une sheet d'instructions iOS dédiée
    installResult.value = 'Sur iOS : utilise le menu Partager → "Sur l\'écran d\'accueil"'
  } else if (outcome === 'accepted') {
    installResult.value = 'Installée !'
  } else if (outcome === 'dismissed') {
    installResult.value = null // l'utilisateur a refusé, ne rien dire
  }
}
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="text-center pt-6 pb-2 px-6">
      <h1 class="font-display text-[54px] text-pond-deep leading-none">
        The <span class="text-duck-deep">Pond</span>.
      </h1>
      <p class="font-sans text-[13px] text-ink-soft tracking-wider mt-2">
        {{ eventName }}
      </p>
    </header>

    <!-- ─── Pond teaser ─────────────────────────────────────── -->
    <div class="mt-3.5 mx-4">
      <Pond :ducks="teaserDucks" :width="348" :height="100" />
    </div>
    <p class="text-center font-mono text-[13px] text-ink-soft mt-2 mb-3.5">
      <b class="text-duck-deep font-medium">{{ guestCount }}</b>
      {{ guestCount === 1 ? 'canard barbote' : 'canards barbotent' }} déjà
    </p>

    <!-- ─── Gen card ────────────────────────────────────────── -->
    <section
      class="mx-4 px-5 pt-6 pb-5 bg-cream-deep rounded-3xl text-center"
      style="box-shadow: inset 0 0 0 1px var(--cream-line), 0 8px 24px -10px rgba(14,79,107,.18);"
    >
      <div class="h-40 flex items-end justify-center relative">
        <div class="duck-bob">
          <Duck :color="color" :size="140" />
        </div>
      </div>

      <!-- Pseudo Caprasimo adaptatif -->
      <div class="mt-2 px-1">
        <div
          class="font-display text-pond-deep leading-tight whitespace-nowrap overflow-visible"
          :class="pseudoSizeClass"
        >
          {{ pseudo || '…' }}
        </div>
      </div>

      <!-- Reroll -->
      <div class="flex flex-col items-center gap-1.5 mt-3.5">
        <button
          type="button"
          class="reroll-btn"
          :disabled="pseudoLoading || registering"
          aria-label="Générer un nouveau canard"
          @click="reroll"
        >
          <svg
            width="24" height="24" viewBox="0 0 24 24"
            fill="none" stroke="currentColor" stroke-width="1.8"
            stroke-linecap="round" stroke-linejoin="round"
          >
            <rect x="3" y="3" width="18" height="18" rx="4" />
            <circle cx="8" cy="8" r="1.4" fill="currentColor" />
            <circle cx="16" cy="8" r="1.4" fill="currentColor" />
            <circle cx="12" cy="12" r="1.4" fill="currentColor" />
            <circle cx="8" cy="16" r="1.4" fill="currentColor" />
            <circle cx="16" cy="16" r="1.4" fill="currentColor" />
          </svg>
        </button>
        <span class="font-mono text-[11px] uppercase tracking-wider text-ink-soft">
          Reroule
        </span>
      </div>

      <p v-if="pseudoError" class="font-mono text-[11px] text-coral-deep mt-2">
        Pseudo indisponible · {{ pseudoError }}
      </p>
    </section>

    <!-- ─── Custom name ─────────────────────────────────────── -->
    <div class="mx-4 mt-4">
      <label
        for="custom-name"
        class="block font-sans text-sm font-medium text-ink mb-2"
      >
        Tu préfères ton vrai prénom&nbsp;?
      </label>
      <input
        id="custom-name"
        v-model="customName"
        type="text"
        placeholder="ex. Léa, la cousine de Marie"
        maxlength="80"
        class="custom-name-input"
      />
      <p class="font-mono text-[11px] text-ink-soft mt-1.5 tracking-wider">
        (optionnel · modifiable plus tard)
      </p>
    </div>

    <!-- ─── CTA PLONGER ─────────────────────────────────────── -->
    <div class="mx-6 mt-4 mb-3">
      <PrimaryButton :disabled="!pseudo || registering" @click="plonger">
        {{ registering ? 'En cours…' : 'PLONGER DANS LA MARE' }}
      </PrimaryButton>
      <p
        v-if="registerError"
        class="font-mono text-[11px] text-coral-deep text-center mt-2"
      >
        {{ registerError }}
      </p>
    </div>

    <!-- ─── Install PWA ─────────────────────────────────────── -->
    <div
      v-if="canInstall"
      class="mt-auto px-6 pt-5 pb-7 bg-cream-deep border-t border-cream-line text-center"
    >
      <button
        type="button"
        class="font-sans font-semibold text-pond-deep inline-flex items-center gap-2"
        @click="onInstall"
      >
        <svg
          width="20" height="20" viewBox="0 0 24 24"
          fill="none" stroke="currentColor" stroke-width="1.8"
          stroke-linecap="round" stroke-linejoin="round"
        >
          <rect x="6" y="2" width="12" height="20" rx="2.5" />
          <line x1="11" y1="18" x2="13" y2="18" />
        </svg>
        {{ isIosLike ? 'Ajouter à l’écran d’accueil' : 'Installer sur mon téléphone' }}
      </button>
      <p class="font-sans text-[13px] text-ink-soft mt-1.5">
        Pour ne pas perdre la mare ce soir
      </p>
      <p
        v-if="installResult"
        class="font-mono text-[11px] text-pond-mid mt-2"
      >
        {{ installResult }}
      </p>
    </div>
  </main>
</template>

<style scoped>
/* Bob du canard de la gen card — repris de design/Welcome v2.html .duck-bob */
@keyframes welcome-bob {
  0%, 100% {
    transform: translateY(0) rotate(-1deg);
  }
  50% {
    transform: translateY(-6px) rotate(1.5deg);
  }
}

.duck-bob {
  animation: welcome-bob 2.6s ease-in-out infinite;
  filter: drop-shadow(0 12px 14px rgba(14, 79, 107, 0.22));
}

/* Reroll : bouton circulaire pond-mid avec halo pulse — design/Welcome v2 .reroll */
.reroll-btn {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  border: none;
  background: var(--pond-mid);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
  box-shadow:
    0 3px 0 var(--pond-deep),
    0 8px 18px -6px rgba(14, 79, 107, 0.45);
  transition: transform 0.12s ease, box-shadow 0.12s ease;
}

.reroll-btn::after {
  content: '';
  position: absolute;
  inset: -5px;
  border-radius: 50%;
  border: 2px solid var(--duck);
  opacity: 0;
  animation: reroll-pulse 2.4s ease-out infinite;
}

@keyframes reroll-pulse {
  0% {
    transform: scale(0.92);
    opacity: 0.55;
  }
  70% {
    transform: scale(1.22);
    opacity: 0;
  }
  100% {
    transform: scale(1.28);
    opacity: 0;
  }
}

.reroll-btn:active:not(:disabled) {
  transform: translateY(2px);
  box-shadow:
    0 1px 0 var(--pond-deep),
    0 4px 10px -4px rgba(14, 79, 107, 0.35);
}

.reroll-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Custom name input — repris de design/Welcome v2 .field input */
.custom-name-input {
  width: 100%;
  height: 52px;
  padding: 0 18px;
  border-radius: 14px;
  border: 1.5px solid var(--cream-line);
  background: var(--cream);
  font-family: var(--sans);
  font-size: 15px;
  color: var(--ink);
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.custom-name-input::placeholder {
  color: #9aa5b0;
  font-style: italic;
}

.custom-name-input:focus {
  border-color: var(--pond-mid);
  box-shadow: 0 0 0 3px rgba(62, 138, 168, 0.18);
}
</style>
