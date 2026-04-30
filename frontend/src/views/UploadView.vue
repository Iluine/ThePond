<script setup lang="ts">
/**
 * UploadView — l'écran d'action principal pour un canard authentifié.
 *
 * Reproduit design/Upload v2.html (variante A avec carte défi). Les 3
 * actions :
 *   - BARBOTER : photo native (file picker, capture=environment) → upload
 *     inline POST /api/media → router.push /confirmation
 *   - CANCANER : navigation /upload/clip (capture in-app via MediaRecorder)
 *   - FAIRE COIN-COIN : navigation /upload/voice
 *
 * Salutation jour/nuit : "Bonsoir {pseudo}" avant 23h, "Tes plumes
 * scintillent magnifiquement, {pseudo}" après — microcopies figées
 * de PROJECT.md § "Microcopy figée".
 */
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import Duck from '../components/Duck.vue'
import PondCounter from '../components/PondCounter.vue'
import PrimaryButton from '../components/PrimaryButton.vue'
import SecondaryButton from '../components/SecondaryButton.vue'
import MareTVPill from '../components/MareTVPill.vue'
import ChallengeBanner from '../components/ChallengeBanner.vue'
import { useUserStore } from '../stores/user'
import { useSnapshotStore } from '../stores/snapshot'
import { useUploadQueueStore } from '../stores/uploadQueue'
import type { DuckColor } from '../types/duck'

const router = useRouter()
const userStore = useUserStore()
const snapshotStore = useSnapshotStore()
const uploadQueue = useUploadQueueStore()

// Si pas authentifié, on renvoie sur Welcome (cas d'accès direct par URL).
onMounted(() => {
  if (!userStore.isAuthenticated) {
    router.replace('/')
  }
})

// ─── Salutation jour / nuit ─────────────────────────────────────
const isNight = new Date().getHours() >= 23
const greeting = computed(() =>
  isNight ? `Tes plumes scintillent magnifiquement,` : `Bonsoir`,
)
const greetingName = computed(() => userStore.displayName ?? 'canard')

// ─── Compteur live depuis le snapshot ──────────────────────────
const totalUsers = computed(() => snapshotStore.counts?.total_users ?? 0)
// expected_guests_count est dans theme.ron — pas surfacé via SSE. TODO :
// l'exposer via /health ou /api/event-info, pour V1 hardcodé sur 52.
const expectedGuests = 52

// ─── Couleur du canard utilisateur ─────────────────────────────
const userColor = computed<DuckColor>(
  () => (userStore.user?.duck_color as DuckColor) ?? 'yellow',
)

// ─── BARBOTER : optimistic UI — enqueue + push immédiat ──────
const photoInput = ref<HTMLInputElement | null>(null)

function pickPhoto() {
  photoInput.value?.click()
}

function onPhotoSelected(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  // Le store lance l'upload en background. La confirmation observe le statut.
  const item = uploadQueue.enqueue(file, 'photo')
  router.push(`/confirmation?type=photo&itemId=${item.id}`)
  if (photoInput.value) photoInput.value.value = ''
}

// ─── Défi en cours (V1 placeholder, vraies données en P1) ─────
// TODO P1 : brancher sur un useChallengeStore qui consomme un endpoint
// /api/challenges/current. Pour V1 on hardcode pour valider l'intégration
// visuelle du bandeau.
const showChallenge = ref(true)
const currentChallenge = {
  title: 'Défi en cours',
  description: 'capture une assiette renversée',
}
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="flex items-center justify-between px-5 pt-4 pb-1.5">
      <div class="font-sans font-bold text-lg text-pond-deep tracking-tight">
        The <span class="text-duck-deep">Pond</span>.
      </div>
      <PondCounter :current="totalUsers" :total="expectedGuests" />
    </header>

    <!-- ─── Greeting ────────────────────────────────────────── -->
    <div class="mx-5 mt-3.5 mb-4 px-4 py-4 bg-cream-deep border border-cream-line rounded-2xl flex items-center gap-3.5">
      <div class="w-16 h-16 flex-shrink-0 flex items-center justify-center">
        <Duck :color="userColor" :size="64" />
      </div>
      <div class="flex-1 min-w-0">
        <div
          class="font-sans text-ink-soft leading-tight mb-1"
          :class="isNight ? 'text-[13px]' : 'text-sm'"
        >
          {{ greeting }}{{ isNight ? '' : ',' }}
        </div>
        <div class="font-display text-pond-deep leading-tight break-words text-[22px]">
          {{ greetingName }}
        </div>
      </div>
    </div>

    <!-- ─── Action grid + bandeau défi ─────────────────────── -->
    <section class="px-5 space-y-3.5">
      <ChallengeBanner
        v-if="showChallenge"
        :title="currentChallenge.title"
        :description="currentChallenge.description"
        href="#"
      />

      <!-- BARBOTER : photo native, navigation immédiate vers /confirmation -->
      <div>
        <PrimaryButton size="xl" @click="pickPhoto">
          <template #icon>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
              <circle cx="12" cy="13" r="4" />
            </svg>
          </template>
          BARBOTER
        </PrimaryButton>
        <input
          ref="photoInput"
          type="file"
          accept="image/*"
          capture="environment"
          class="hidden"
          @change="onPhotoSelected"
        />
      </div>

      <div class="grid grid-cols-2 gap-3">
        <SecondaryButton
          variant="cream"
          sub-label="15s"
          @click="router.push('/upload/clip')"
        >
          <template #icon>
            <span class="relative">
              <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <rect x="2" y="6" width="14" height="12" rx="2" />
                <path d="M22 8 L16 12 L22 16 Z" />
              </svg>
              <!-- petit point rouge enregistrement de la maquette -->
              <span
                class="absolute top-0.5 right-0.5 w-2 h-2 rounded-full bg-rec"
                style="box-shadow: 0 0 0 1.5px var(--cream-deep);"
              />
            </span>
          </template>
          CANCANER
        </SecondaryButton>

        <SecondaryButton
          variant="coral"
          sub-label="60s"
          @click="router.push('/upload/voice')"
        >
          <template #icon>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="9" y="2" width="6" height="12" rx="3" />
              <path d="M5 11a7 7 0 0 0 14 0" />
              <line x1="12" y1="18" x2="12" y2="22" />
            </svg>
          </template>
          FAIRE COIN-COIN
        </SecondaryButton>
      </div>
    </section>

    <!-- ─── MareTV link ─────────────────────────────────────── -->
    <div class="text-center mt-auto pt-7 pb-6">
      <MareTVPill to="/gallery" label="Voir ce qui se passe dans la mare" />
    </div>
  </main>
</template>
