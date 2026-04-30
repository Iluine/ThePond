<script setup lang="ts">
/**
 * Coque d'application. Le routeur monte la vue active dans <RouterView />.
 * On lance la connexion SSE au montage de l'app — un seul EventSource
 * partagé pour toutes les vues, l'état va dans useSnapshotStore.
 */
import { useEventStream } from './composables/useEventStream'

useEventStream()
</script>

<template>
  <RouterView />
</template>
