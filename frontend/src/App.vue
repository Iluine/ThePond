<script setup lang="ts">
/**
 * Coque d'application. Le routeur monte la vue active dans <RouterView />.
 * On lance la connexion SSE au montage de l'app — un seul EventSource
 * partagé pour toutes les vues, l'état va dans useSnapshotStore.
 *
 * On rehydrate aussi la queue d'uploads depuis IndexedDB : si l'invité a
 * fermé l'onglet en plein milieu d'un upload, on retrouve les blobs et le
 * runner reprend là où on s'était arrêté (online) ou attend (offline).
 */
import { onMounted } from 'vue'
import { useEventStream } from './composables/useEventStream'
import { useUploadQueueStore } from './stores/uploadQueue'
import PendingUploadsPill from './components/PendingUploadsPill.vue'

useEventStream()

const uploadQueue = useUploadQueueStore()
onMounted(() => {
  void uploadQueue.rehydrate()
})
</script>

<template>
  <RouterView />
  <PendingUploadsPill />
</template>
