<script setup lang="ts">
/**
 * SecondaryButton — bouton secondaire vertical (icon + label + sub-label).
 *
 * Deux variants conformes aux classes CSS de design/Upload v2.html :
 *   - cream : fond cream-deep, texte pond-deep, ombre cream-line.
 *             Utilisé pour CANCANER (clip vidéo).
 *   - coral : fond coral-soft, texte coral-deep, ombre coral-line.
 *             Réservé au vocal d'après PROJECT.md (FAIRE COIN-COIN).
 *
 * NOTE : Upload v2.html contient un drift inline (les deux boutons sont
 * forcés en cream via style=). On code ici les deux variants conformément
 * à la classe CSS — l'arbitrage cream-vs-coral pour FAIRE COIN-COIN se
 * fera au composant UploadView.vue (prompt 8).
 */

defineProps<{
  /** Palette du bouton. coral est réservé au vocal d'après PROJECT.md. */
  variant?: 'cream' | 'coral'
  /** Texte secondaire (ex. "15s", "60s"). */
  subLabel?: string
  disabled?: boolean
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
    class="btn-secondary"
    :class="[variant === 'coral' ? 'btn-secondary--coral' : 'btn-secondary--cream']"
    @click="$emit('click', $event)"
  >
    <span v-if="$slots.icon" class="btn-secondary__icon">
      <slot name="icon" />
    </span>
    <span class="btn-secondary__label">
      <slot />
    </span>
    <span v-if="subLabel" class="btn-secondary__sub">
      {{ subLabel }}
    </span>
  </button>
</template>

<style scoped>
.btn-secondary {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  height: 100px;
  padding: 14px 12px;
  border: none;
  border-radius: 20px;
  font-family: var(--sans);
  font-weight: 700;
  text-align: center;
  cursor: pointer;
  transition: transform 0.1s ease, box-shadow 0.1s ease;
}

.btn-secondary:active:not(:disabled) {
  transform: translateY(3px);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
}

.btn-secondary__label {
  font-size: 15px;
  font-weight: 700;
  line-height: 1.05;
  letter-spacing: 0.01em;
}

.btn-secondary__sub {
  font-family: var(--mono);
  font-weight: 500;
  font-size: 13px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

/* ─── variant: cream (CANCANER) ────────────────────────────────── */
.btn-secondary--cream {
  background: var(--cream-deep);
  color: var(--pond-deep);
  box-shadow:
    0 4px 0 var(--cream-line),
    0 8px 18px -8px rgba(14, 79, 107, 0.18);
}
.btn-secondary--cream:active:not(:disabled) {
  box-shadow:
    0 1px 0 var(--cream-line),
    0 4px 10px -4px rgba(14, 79, 107, 0.15);
}
.btn-secondary--cream .btn-secondary__sub {
  color: var(--ink-soft);
}

/* ─── variant: coral (FAIRE COIN-COIN) ─────────────────────────── */
.btn-secondary--coral {
  background: var(--coral-soft);
  color: var(--coral-deep);
  box-shadow:
    0 4px 0 var(--coral-line),
    0 10px 20px -8px rgba(184, 90, 63, 0.28);
}
.btn-secondary--coral:active:not(:disabled) {
  box-shadow:
    0 1px 0 var(--coral-line),
    0 4px 10px -4px rgba(184, 90, 63, 0.2);
}
.btn-secondary--coral .btn-secondary__sub {
  color: var(--coral-deep);
  opacity: 0.7;
}
</style>
