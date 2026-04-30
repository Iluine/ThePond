<script setup lang="ts">
/**
 * PondCounter — pill compteur de canards "{N} / {M} {label}".
 *
 * Variants de taille :
 *   - sm : header mobile (Upload, Confirmation, galerie). 13px mono.
 *   - lg : Mare TV plein écran. 56px display Caprasimo.
 *
 * Le nombre courant est mis en avant en duck-deep ; le total et le label
 * en pond-deep / ink-soft selon la variante. Quand current === total, on
 * passe la pill en plein duck (signal "complet") — voir Mosaic 6 état 4.
 *
 * Sources : design/Upload v2.html .pill, design/Confirmation.html .pill,
 * design/Slideshow TV v2.html .pill-big, design/Mosaic 6 screens.html .pill.full.
 */

const props = withDefaults(
  defineProps<{
    current: number
    total: number
    label?: string
    size?: 'sm' | 'lg'
  }>(),
  {
    label: 'canards',
    size: 'sm',
  },
)

const isFull = () => props.current >= props.total
</script>

<template>
  <span
    class="pond-counter"
    :class="[
      size === 'lg' ? 'pond-counter--lg' : 'pond-counter--sm',
      { 'pond-counter--full': isFull() },
    ]"
  >
    <span class="pond-counter__num">{{ current }}</span>
    <span class="pond-counter__sep">/ {{ total }}</span>
    <span class="pond-counter__label">{{ label }}</span>
  </span>
</template>

<style scoped>
.pond-counter {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
  background: var(--cream-deep);
  color: var(--pond-deep);
  border: 1px solid var(--cream-line);
  border-radius: 999px;
  letter-spacing: 0.02em;
}

.pond-counter__num {
  color: var(--duck-deep);
  font-weight: 600;
}

.pond-counter__sep {
  opacity: 0.55;
}

.pond-counter__label {
  font-weight: 500;
}

/* ─── size: sm — mobile header pill ─────────────────────────────── */
.pond-counter--sm {
  font-family: var(--mono);
  font-size: 13px;
  font-weight: 500;
  padding: 8px 14px;
}
.pond-counter--sm .pond-counter__num {
  font-size: 15px;
  font-weight: 700;
}

/* ─── size: lg — Mare TV display pill ──────────────────────────── */
.pond-counter--lg {
  font-family: var(--display);
  font-size: 56px;
  line-height: 1;
  padding: 18px 32px;
  gap: 18px;
  background: var(--pond-deep);
  color: white;
  border: none;
}
.pond-counter--lg .pond-counter__num {
  color: white;
}
.pond-counter--lg .pond-counter__sep {
  font-size: 40px;
  opacity: 0.55;
}
.pond-counter--lg .pond-counter__label {
  font-family: var(--mono);
  font-size: 18px;
  font-weight: 500;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.85;
}

/* ─── modifier: full (current === total) ───────────────────────── */
.pond-counter--full.pond-counter--sm {
  background: var(--duck);
  color: var(--ink);
  border-color: var(--duck-deep);
}
.pond-counter--full.pond-counter--sm .pond-counter__num {
  color: var(--ink);
}
</style>
