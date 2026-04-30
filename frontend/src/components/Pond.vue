<script setup lang="ts">
/**
 * Pond — la mare visuelle, scatter de Duck via Bridson Poisson-disk.
 *
 * Algorithme et constantes repris de design/Welcome v2.html (pond-teaser).
 * Le couple (yellow + white crowned) occupe le centre quand couple_visible.
 * En night_mode, fond pond-deeper et glow appliqué à tous les invités
 * (rim light blanc) + glow champagne sur le couple — règles de
 * design/Slideshow TV v2.html .frame.night.
 *
 * V1 simplifications (à faire évoluer plus tard) :
 *   - Positions recalculées si ducks.length change (pas de stabilité par
 *     duck.id). À refactorer quand le SSE poussera des updates incrémentales.
 *   - Dimensions explicites via props width/height. Pour responsive,
 *     enrouler dans un parent qui ResizeObserver et passe les valeurs.
 */
import { ref, watch, computed } from 'vue'
import Duck from './Duck.vue'

type DuckColor = 'yellow' | 'white' | 'blue' | 'rainbow'

export type PondDuck = {
  id?: string | number
  color: DuckColor
}

const props = withDefaults(
  defineProps<{
    ducks: PondDuck[]
    width?: number
    height?: number
    coupleVisible?: boolean
    nightMode?: boolean
    /** Affiche les nénuphars au fond. */
    lilies?: boolean
  }>(),
  {
    width: 348,
    height: 100,
    coupleVisible: true,
    nightMode: false,
    lilies: true,
  },
)

// ─── Constantes algorithmiques ──────────────────────────────────
// Toutes reprises de Welcome v2.html, scalées si besoin.
const COUPLE_R = 50           // rayon de la zone réservée au centre
const MIN_DIST = 30           // distance minimale entre deux ducks
const MAX_ATTEMPTS = 30       // tentatives Bridson par point actif
const INSET_X = 18            // marge horizontale (clip ronds OK)
const INSET_Y = 14            // marge verticale
const DUCK_MIN = 22           // largeur min d'un duck invité
const DUCK_MAX = 28           // largeur max d'un duck invité
const COUPLE_DUCK_W = 74      // largeur d'un duck du couple
const COUPLE_OFFSET = 22      // décalage horizontal de chaque duck du couple

// Lilies en pourcentages (normalisées sur le 348×100 d'origine).
const LILY_POSITIONS = [
  [24 / 348, 18 / 100],
  [60 / 348, 80 / 100],
  [300 / 348, 30 / 100],
  [260 / 348, 82 / 100],
  [180 / 348, 12 / 100],
  [330 / 348, 68 / 100],
] as const

type Pos = { x: number; y: number; size: number; z: number }

// ─── Bridson Poisson-disk ───────────────────────────────────────
function poissonDisk(width: number, height: number, minDist: number): Pos[] {
  const cell = minDist / Math.SQRT2
  const cols = Math.ceil(width / cell)
  const rows = Math.ceil(height / cell)
  const grid: (Pos | null)[] = new Array(cols * rows).fill(null)
  const points: Pos[] = []
  const active: Pos[] = []

  const cx = width / 2
  const cy = height / 2
  const reservedR = props.coupleVisible ? COUPLE_R : 0

  function fits(p: Pos): boolean {
    if (p.x < INSET_X || p.x > width - INSET_X) return false
    if (p.y < INSET_Y || p.y > height - INSET_Y) return false
    if (Math.hypot(p.x - cx, p.y - cy) < reservedR) return false
    const gx = Math.floor(p.x / cell)
    const gy = Math.floor(p.y / cell)
    for (let yy = Math.max(0, gy - 2); yy <= Math.min(rows - 1, gy + 2); yy++) {
      for (let xx = Math.max(0, gx - 2); xx <= Math.min(cols - 1, gx + 2); xx++) {
        const o = grid[yy * cols + xx]
        if (o && Math.hypot(o.x - p.x, o.y - p.y) < minDist) return false
      }
    }
    return true
  }

  function add(p: Pos) {
    points.push(p)
    active.push(p)
    grid[Math.floor(p.y / cell) * cols + Math.floor(p.x / cell)] = p
  }

  // Seed loin du centre pour éviter de chercher à percer la couple zone
  const seed: Pos = { x: INSET_X, y: INSET_Y, size: 0, z: 0 }
  if (fits(seed)) add(seed)

  while (active.length) {
    const i = Math.floor(Math.random() * active.length)
    const p = active[i]
    let placed = false
    for (let k = 0; k < MAX_ATTEMPTS; k++) {
      const a = Math.random() * Math.PI * 2
      const r = minDist * (1 + Math.random())
      const np: Pos = {
        x: p.x + Math.cos(a) * r,
        y: p.y + Math.sin(a) * r,
        size: 0,
        z: 0,
      }
      if (fits(np)) {
        add(np)
        placed = true
        break
      }
    }
    if (!placed) active.splice(i, 1)
  }

  return points
}

// ─── Calcul réactif des positions ───────────────────────────────
const positions = ref<Pos[]>([])

function recompute() {
  const pts = poissonDisk(props.width, props.height, MIN_DIST)

  // "Farthest first" — favorise une composition rim
  const cx = props.width / 2
  const cy = props.height / 2
  pts.sort((a, b) => Math.hypot(b.x - cx, b.y - cy) - Math.hypot(a.x - cx, a.y - cy))

  // Cap au nombre de ducks demandés
  const n = Math.min(props.ducks.length, pts.length)
  const out: Pos[] = []
  for (let i = 0; i < n; i++) {
    const { x, y } = pts[i]
    out.push({
      x,
      y,
      size: DUCK_MIN + Math.round(Math.random() * (DUCK_MAX - DUCK_MIN)),
      // Les ducks plus bas dans la mare passent au-dessus (perspective).
      z: Math.round(2 + (y / props.height) * 6),
    })
  }
  positions.value = out
}

watch(
  () => [props.ducks.length, props.width, props.height, props.coupleVisible],
  recompute,
  { immediate: true },
)

// ─── Couple central ─────────────────────────────────────────────
const couplePositions = computed(() => {
  if (!props.coupleVisible) return null
  const cx = props.width / 2
  const cy = props.height / 2
  return {
    left:  { x: cx - COUPLE_OFFSET, y: cy, z: 19 },
    right: { x: cx + COUPLE_OFFSET, y: cy, z: 20 },
  }
})

// ─── Lilies absolues (pixel) à partir des pourcentages ─────────
const lilyPixels = computed(() =>
  LILY_POSITIONS.map(([px, py]) => ({
    left: px * props.width,
    top: py * props.height,
  })),
)
</script>

<template>
  <div
    class="pond"
    :class="{ 'pond--night': nightMode }"
    :style="{ width: width + 'px', height: height + 'px' }"
  >
    <!-- Lilies -->
    <template v-if="lilies">
      <span
        v-for="(p, i) in lilyPixels"
        :key="`lily-${i}`"
        class="pond__lily"
        :style="{ left: p.left + 'px', top: p.top + 'px' }"
      />
    </template>

    <!-- Invités (Bridson scatter) -->
    <div
      v-for="(duck, i) in ducks.slice(0, positions.length)"
      :key="duck.id ?? `duck-${i}`"
      class="pond__duck"
      :style="{
        left: positions[i].x + 'px',
        top: positions[i].y + 'px',
        zIndex: positions[i].z,
      }"
    >
      <Duck
        :color="duck.color"
        :size="positions[i].size"
        :glow="nightMode"
      />
    </div>

    <!-- Couple central (yellow + white crowned) -->
    <template v-if="couplePositions">
      <div
        class="pond__duck pond__duck--couple"
        :style="{
          left: couplePositions.left.x + 'px',
          top: couplePositions.left.y + 'px',
          zIndex: couplePositions.left.z,
        }"
      >
        <Duck color="yellow" :size="COUPLE_DUCK_W" crowned :glow="nightMode" />
      </div>
      <div
        class="pond__duck pond__duck--couple"
        :style="{
          left: couplePositions.right.x + 'px',
          top: couplePositions.right.y + 'px',
          zIndex: couplePositions.right.z,
        }"
      >
        <Duck color="white" :size="COUPLE_DUCK_W" crowned :glow="nightMode" />
      </div>
    </template>
  </div>
</template>

<style scoped>
.pond {
  position: relative;
  border-radius: 18px;
  overflow: hidden;
  /* Day mode : eau claire avec petits reflets blancs */
  background-image:
    radial-gradient(circle at 25% 30%, rgba(255, 255, 255, 0.55) 0 2px, transparent 3px),
    radial-gradient(circle at 70% 60%, rgba(255, 255, 255, 0.45) 0 3px, transparent 4px),
    radial-gradient(circle at 45% 80%, rgba(255, 255, 255, 0.4) 0 2px, transparent 3px),
    radial-gradient(circle at 88% 22%, rgba(255, 255, 255, 0.35) 0 2px, transparent 3px),
    linear-gradient(180deg, var(--pond-light) 0%, var(--pond-mid) 100%);
  box-shadow: inset 0 0 0 1px rgba(14, 79, 107, 0.08);
}

.pond--night {
  background-image:
    radial-gradient(circle at 25% 30%, rgba(255, 255, 255, 0.18) 0 3px, transparent 5px),
    radial-gradient(circle at 70% 60%, rgba(255, 255, 255, 0.14) 0 5px, transparent 7px),
    radial-gradient(circle at 45% 80%, rgba(255, 255, 255, 0.12) 0 3px, transparent 5px),
    radial-gradient(circle at 88% 22%, rgba(255, 255, 255, 0.1) 0 3px, transparent 5px),
    linear-gradient(180deg, #1B6685 0%, #062E40 100%);
}

/* Lueur de lune sur fond nuit */
.pond--night::after {
  content: '';
  position: absolute;
  inset: 0;
  background: radial-gradient(
    ellipse 60% 30% at 50% 0%,
    rgba(232, 199, 122, 0.15),
    transparent 60%
  );
  pointer-events: none;
}

.pond__lily {
  position: absolute;
  width: 18px;
  height: 8px;
  border-radius: 50%;
  background: #4FA86B;
  opacity: 0.65;
  pointer-events: none;
}

.pond--night .pond__lily {
  background: #2F6E48;
  opacity: 0.85;
}

.pond__duck {
  position: absolute;
  /* Center le SVG sur (x, y) plutôt que de le poser top-left, pour
     coller au comportement de Welcome v2.html `transform: translate(-50%, -50%)`. */
  transform: translate(-50%, -50%);
  pointer-events: none;
}
</style>
