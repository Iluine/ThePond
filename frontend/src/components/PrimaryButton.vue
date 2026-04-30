<script setup lang="ts">
/**
 * PrimaryButton — bouton physique Duck Yellow signature.
 *
 * Ombre 0 4px 0 var(--duck-deep) qui s'écrase au :active (translateY +
 * ombre réduite). Présent dans Welcome (CTA "PLONGER DANS LA MARE"),
 * Upload (BARBOTER), Confirmation (CONTINUER À BARBOTER), Orchestration
 * (Passer au prochain palier).
 *
 * Sources : design/Welcome v2.html .cta, design/Upload v2.html .btn-primary,
 * design/Confirmation.html .cta, design/Mosaic 6 screens.html .o-cta.
 */

defineProps<{
  /** Taille verticale. sm=48 (orchestration), md=64 (welcome), lg=80
   *  (confirmation), xl=112 (upload BARBOTER pleine largeur). */
  size?: 'sm' | 'md' | 'lg' | 'xl'
  /** Désactive le bouton (pas de :active, opacité réduite). */
  disabled?: boolean
  /** Submit form si dans un <form>, sinon bouton standard. */
  type?: 'button' | 'submit'
}>()

defineEmits<{
  click: [event: MouseEvent]
}>()
</script>

<template>
  <button
    :type="type ?? 'button'"
    :disabled="disabled"
    class="btn-primary"
    :class="{
      'h-12 text-sm rounded-2xl px-5':                size === 'sm',
      'h-20 text-lg rounded-2xl px-6':                size === 'lg',
      'h-28 text-xl rounded-3xl px-6':                size === 'xl',
      'h-16 text-base rounded-2xl px-6':              !size || size === 'md',
    }"
    @click="$emit('click', $event)"
  >
    <span v-if="$slots.icon" class="btn-primary__icon">
      <slot name="icon" />
    </span>
    <span class="btn-primary__label">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.btn-primary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 14px;
  width: 100%;
  font-family: var(--sans);
  font-weight: 700;
  letter-spacing: 0.02em;
  background: var(--duck);
  color: var(--ink);
  border: none;
  cursor: pointer;
  /* Ombre signature physique : nappe duck-deep + halo doux */
  box-shadow:
    0 4px 0 var(--duck-deep),
    0 10px 24px -8px rgba(242, 180, 0, 0.55);
  transition: transform 0.1s ease, box-shadow 0.1s ease;
}

.btn-primary:active:not(:disabled) {
  transform: translateY(3px);
  box-shadow:
    0 1px 0 var(--duck-deep),
    0 4px 10px -4px rgba(242, 180, 0, 0.4);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 42px;
  height: 42px;
  flex-shrink: 0;
  border-radius: 12px;
  background: rgba(255, 255, 255, 0.5);
  color: var(--pond-deep);
}

.btn-primary__label {
  flex: 1 1 auto;
  text-align: center;
  min-width: 0;
}
</style>
