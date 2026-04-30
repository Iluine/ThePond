<script setup lang="ts">
/**
 * ChallengeBanner — bandeau "Défi en cours" champagne-pale.
 *
 * Repris de design/Upload v2.html .challenge. Pour V1, le bandeau est
 * mounté dans UploadView avec un défi hardcodé. La vraie mécanique de
 * défis (scavenger hunt avec un défi actif à la fois) arrive en P1
 * (cf. PROJECT.md § "Phase 1").
 *
 * Note d'intégration design/INDEX.md : le bandeau s'intègre AU-DESSUS
 * du bouton BARBOTER plutôt que comme 4e CTA séparé.
 */

defineProps<{
  /** Étiquette en mono uppercase, ex. "Défi en cours" */
  title: string
  /** Texte du défi en cours */
  description: string
  /** Si fourni, rend le bandeau cliquable */
  href?: string
}>()

defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <component
    :is="href ? 'a' : 'div'"
    :href="href"
    class="challenge"
    @click="$emit('click', $event)"
  >
    <span class="challenge__ic" aria-hidden="true">
      <svg
        width="20" height="20" viewBox="0 0 24 24"
        fill="none" stroke="currentColor" stroke-width="1.8"
        stroke-linecap="round" stroke-linejoin="round"
      >
        <circle cx="12" cy="12" r="9" />
        <circle cx="12" cy="12" r="5" />
        <circle cx="12" cy="12" r="1.5" fill="currentColor" />
      </svg>
    </span>
    <div class="challenge__body">
      <div class="challenge__title">{{ title }}</div>
      <div class="challenge__desc">{{ description }}</div>
    </div>
    <span v-if="href" class="challenge__arrow" aria-hidden="true">
      <svg
        width="16" height="16" viewBox="0 0 24 24"
        fill="none" stroke="currentColor" stroke-width="2"
        stroke-linecap="round" stroke-linejoin="round"
      >
        <polyline points="9 6 15 12 9 18" />
      </svg>
    </span>
  </component>
</template>

<style scoped>
.challenge {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 14px 16px;
  background: var(--champagne-pale);
  border: 1px solid var(--champagne);
  border-radius: 18px;
  box-shadow: 0 6px 16px -8px rgba(201, 163, 71, 0.4);
  text-decoration: none;
  color: inherit;
}

a.challenge {
  cursor: pointer;
}

.challenge__ic {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: #fff;
  border: 1px solid var(--champagne);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--champagne-deep);
}

.challenge__body {
  flex: 1;
  min-width: 0;
}

.challenge__title {
  font-family: var(--sans);
  font-weight: 600;
  font-size: 13px;
  color: var(--pond-deep);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 2px;
}

.challenge__desc {
  font-family: var(--sans);
  font-weight: 500;
  font-size: 14px;
  color: var(--ink);
  line-height: 1.25;
}

.challenge__arrow {
  flex-shrink: 0;
  color: var(--champagne-deep);
}
</style>
