<script setup lang="ts">
/**
 * StubView — placeholder générique pour toutes les routes non encore
 * implémentées. Le contenu vient des meta de la route :
 *   { title, screen, implementedAt }
 *
 * Quand une vraie vue est livrée à son prompt thématique, il suffit
 * de remplacer `component: StubView` par `component: () => import(...)`
 * dans le router. Ce fichier reste utile tant qu'au moins une route
 * est encore stubbée.
 */
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const title = computed(() => (route.meta.title as string) ?? 'Vue inconnue')
const screen = computed(() => (route.meta.screen as string | undefined) ?? '')
const promptN = computed(() => route.meta.implementedAt as number | undefined)
</script>

<template>
  <main class="min-h-screen flex flex-col items-center justify-center gap-6 p-8 text-center bg-cream">
    <div class="space-y-2">
      <h1 class="font-display text-5xl text-pond-deep leading-none">
        {{ title }}
      </h1>
      <p v-if="screen" class="font-mono text-xs uppercase tracking-widest text-ink-soft">
        {{ screen }}
      </p>
    </div>

    <div class="font-mono text-sm text-ink-soft max-w-md">
      <span class="font-sans text-base">Stub</span>
      <span v-if="promptN"> · sera implémenté au prompt {{ promptN }}</span>
    </div>

    <RouterLink
      to="/dev"
      class="font-sans text-pond-mid hover:text-pond-deep underline underline-offset-4"
    >
      ← retour au showcase
    </RouterLink>
  </main>
</template>
