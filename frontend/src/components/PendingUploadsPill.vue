<script setup lang="ts">
/**
 * PendingUploadsPill — indicateur flottant bottom-right qui signale les
 * uploads en attente de partir.
 *
 * Affiché uniquement quand `inFlightCount > 0`. Couleur change selon
 * l'état réseau :
 *   - online  : Duck Yellow → "ça part"
 *   - offline : Coral Soft → "on attend la connexion"
 *
 * Tap = pas d'action en V1 (pas de drawer expand). On affiche juste
 * un compteur honnête. Une future itération pourra ajouter une liste
 * détaillée si on a un retour utilisateur qui le demande.
 */
import { computed } from 'vue'
import { useUploadQueueStore } from '../stores/uploadQueue'
import { useNetworkStatus } from '../composables/useNetworkStatus'

const uploadQueue = useUploadQueueStore()
const { isOnline } = useNetworkStatus()

const visible = computed(() => uploadQueue.inFlightCount > 0)
const count = computed(() => uploadQueue.inFlightCount)
const failedCount = computed(() => uploadQueue.failed.length)

const label = computed(() => {
  if (!isOnline.value) {
    return count.value === 1
      ? 'En attente du réseau'
      : `${count.value} en attente du réseau`
  }
  if (failedCount.value > 0 && failedCount.value === count.value) {
    return failedCount.value === 1 ? '1 échec' : `${failedCount.value} échecs`
  }
  return count.value === 1 ? 'Envoi en cours' : `${count.value} envois en cours`
})
</script>

<template>
  <Transition name="pill">
    <aside
      v-if="visible"
      class="pill"
      :class="[isOnline ? 'pill--online' : 'pill--offline']"
      role="status"
      aria-live="polite"
    >
      <span class="pill__icon" aria-hidden="true">
        <svg
          v-if="isOnline"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 2v6m0 14v-6m10-4h-6M2 12h6m13.07-7.07l-4.24 4.24M7.17 16.83l-4.24 4.24m0-18.14l4.24 4.24m9.66 9.66l4.24 4.24" />
        </svg>
        <svg
          v-else
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2.4"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M3 3l18 18" />
          <path d="M5 12.55a11 11 0 0 1 5.17-2.39" />
          <path d="M1.42 9a16 16 0 0 1 4.64-3.17" />
          <path d="M22.58 9a16 16 0 0 0-4.95-3.36" />
          <path d="M19 12.55a11 11 0 0 0-2.93-1.91" />
          <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
          <line x1="12" y1="20" x2="12.01" y2="20" />
        </svg>
      </span>
      <span class="pill__label">{{ label }}</span>
    </aside>
  </Transition>
</template>

<style scoped>
.pill {
  position: fixed;
  bottom: calc(16px + env(safe-area-inset-bottom));
  right: 16px;
  z-index: 80;
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 9px 14px 9px 12px;
  border-radius: 999px;
  font-family: var(--sans);
  font-size: 13px;
  font-weight: 600;
  border: 1px solid;
  box-shadow: 0 6px 16px -8px rgba(14, 79, 107, 0.4);
  user-select: none;
}

.pill--online {
  background: var(--duck);
  color: var(--ink);
  border-color: var(--duck-deep);
}

.pill--offline {
  background: var(--coral-soft, #f8d9cf);
  color: var(--ink);
  border-color: var(--coral, #e07a5f);
}

.pill__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.55);
}

.pill--online .pill__icon {
  animation: pill-spin 1.6s linear infinite;
}

@keyframes pill-spin {
  from { transform: rotate(0deg); }
  to   { transform: rotate(360deg); }
}

.pill__label {
  white-space: nowrap;
}

/* ─── Mount/unmount transition ─────────────────────────────── */
.pill-enter-active,
.pill-leave-active {
  transition: transform 0.25s cubic-bezier(0.2, 0.9, 0.3, 1.05),
              opacity 0.2s ease;
}
.pill-enter-from,
.pill-leave-to {
  transform: translateY(20px);
  opacity: 0;
}
</style>
