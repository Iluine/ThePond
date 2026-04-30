<script setup lang="ts">
/**
 * MareTVPill — lien discret vers Mare TV.
 *
 * Présent dans le footer de Confirmation et un peu partout dans la nav
 * pour proposer aux invités de voir l'écran de diffusion. Style "secondary
 * link" : pond-mid au repos, pond-deep au hover, flèche → en monospace.
 *
 * Si tu veux le badge LIVE animé qui apparaît dans Mosaic 6 sur l'aperçu
 * de l'écran TV, c'est un autre composant — il vit avec MareTVView.
 *
 * Source : design/Confirmation.html .secondary a + design/Upload v2.html .quiet a
 *          (le lien "Voir ce qui se passe dans la mare →" a la même mécanique).
 */

const props = withDefaults(
  defineProps<{
    /** Cible du lien. Défaut : la route /mare-tv (à câbler dans le router
     *  au prompt 4 si on garde ce nom). */
    to?: string
    /** Override le libellé. Défaut : la microcopie figée de strings.ron. */
    label?: string
  }>(),
  {
    to: '/mare-tv',
    label: 'Voir Mare TV',
  },
)
</script>

<template>
  <RouterLink :to="props.to" class="mare-tv-pill">
    {{ props.label }}
    <span class="mare-tv-pill__arrow" aria-hidden="true">→</span>
  </RouterLink>
</template>

<style scoped>
.mare-tv-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 4px;
  font-family: var(--sans);
  font-size: 14px;
  color: var(--pond-mid);
  text-decoration: none;
  transition: color 0.15s ease;
}

.mare-tv-pill:hover,
.mare-tv-pill:focus-visible {
  color: var(--pond-deep);
}

.mare-tv-pill:focus-visible {
  outline: 2px solid var(--pond-mid);
  outline-offset: 3px;
  border-radius: 4px;
}

.mare-tv-pill__arrow {
  font-family: var(--mono);
  font-size: 16px;
  line-height: 1;
  transition: transform 0.15s ease;
}

.mare-tv-pill:hover .mare-tv-pill__arrow {
  transform: translateX(2px);
}
</style>
