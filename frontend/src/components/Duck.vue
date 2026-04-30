<script setup lang="ts">
/**
 * Duck — pièce centrale visuelle du projet, SVG paramétrique.
 *
 * Source : symbol #duck-glyph identique dans les 6 maquettes design/.
 * Le bec (#F2B400 + #C9881A shadow) reste DELIBEREMENT jaune pour toutes
 * les variantes — confirmé dans tous les usages du HTML, c'est une
 * signature visuelle.
 *
 * ─── Arbitrage WHITE ────────────────────────────────────────────
 * On adopte la version warm cream (#EADFB8) du Welcome / Confirmation /
 * Mosaic, plutôt que la true white (#FAF3E3) de Slideshow TV.
 *   - Le commentaire de Welcome v2.html justifie : "warm Cream tone...
 *     so it stays visible on Cream Deep without needing an outline".
 *   - Utilisée dans 4 des 5 contextes maquettés.
 *   - La version TV était un override contextuel pour la lisibilité
 *     écran ; MareTVView pourra réinjecter ses propres --body/--belly/
 *     --shade via CSS si nécessaire (les variables sont scoped à l'inline
 *     style du SVG, donc surchargeable depuis l'extérieur).
 *
 * ─── Glow ───────────────────────────────────────────────────────
 * Quand glow=true, on applique le drop-shadow filter du mode nuit Mare TV.
 *   - Standard (invités)  : halo blanc subtil
 *   - Couple (crowned)    : halo champagne plus large
 * Le couple a ce halo plus prononcé pour qu'on le distingue toujours
 * dans la mare.
 */
import { computed, useId } from 'vue'

type DuckColor = 'yellow' | 'white' | 'blue' | 'rainbow'

const props = withDefaults(
  defineProps<{
    color?: DuckColor
    size?: number
    crowned?: boolean
    glow?: boolean
    /** Variante "endormi" : tête abaissée, œil fermé en courbe.
     *  Utilisée par GalleryView pour les états mare-endormie/nocturne. */
    asleep?: boolean
  }>(),
  {
    color: 'yellow',
    size: 56,
    crowned: false,
    glow: false,
    asleep: false,
  },
)

// Variantes solides — body / belly / shade.
// shade = surfaces auto-occlusées (queue, aile).
const VARIANTS = {
  yellow: { body: '#FFC93C', belly: '#FFE08A', shade: '#F2B400' },
  white:  { body: '#EADFB8', belly: '#F5EBC9', shade: '#C9B988' },
  blue:   { body: '#7AB8D9', belly: '#A9D8E5', shade: '#4E91B5' },
} as const satisfies Record<Exclude<DuckColor, 'rainbow'>, { body: string; belly: string; shade: string }>

// Une instance = un id unique pour son linearGradient rainbow,
// pour qu'on puisse mettre 50 ducks rainbow sur la même page sans
// collision de #rainbow-grad.
const uid = useId()
const rainbowId = computed(() => `pond-rainbow-${uid}`)

const cssVars = computed<Record<string, string>>(() => {
  if (props.color === 'rainbow') {
    return {
      '--body':  `url(#${rainbowId.value})`,
      '--belly': 'rgba(255, 255, 255, 0.55)',
      // shade = violet doux, valeur reprise du HTML Welcome
      '--shade': '#9F70B5',
    }
  }
  const v = VARIANTS[props.color]
  return {
    '--body':  v.body,
    '--belly': v.belly,
    '--shade': v.shade,
  }
})

const viewBox = computed(() => (props.crowned ? '0 0 64 60' : '0 0 64 50'))

// Hauteur calculée depuis size pour que le ratio reste correct.
// crowned=true → ratio 60/64, sinon 50/64.
const heightPx = computed(() =>
  Math.round((props.size * (props.crowned ? 60 : 50)) / 64),
)

const glowFilter = computed(() => {
  if (!props.glow) return undefined
  if (props.crowned) {
    // Couple : halo champagne large, ombre prononcée
    return 'drop-shadow(0 0 16px rgba(232,199,122,.45)) drop-shadow(0 4px 6px rgba(0,0,0,.5))'
  }
  // Invités : halo blanc subtil
  return 'drop-shadow(0 0 8px rgba(255,255,255,.18)) drop-shadow(0 2px 4px rgba(0,0,0,.4))'
})

const computedStyle = computed(() => ({
  ...cssVars.value,
  ...(glowFilter.value ? { filter: glowFilter.value } : {}),
  display: 'block',
}))
</script>

<template>
  <svg
    :width="size"
    :height="heightPx"
    :viewBox="viewBox"
    :style="computedStyle"
    role="img"
    :aria-label="`canard ${color}${crowned ? ' couronné' : ''}${asleep ? ' endormi' : ''}`"
  >
    <defs v-if="color === 'rainbow'">
      <linearGradient :id="rainbowId" x1="0" x2="1" y1="0" y2="0">
        <stop offset="0%"   stop-color="#E07A5F" />
        <stop offset="25%"  stop-color="#FFC93C" />
        <stop offset="50%"  stop-color="#7DC97D" />
        <stop offset="75%"  stop-color="#7AB8D9" />
        <stop offset="100%" stop-color="#B58AC9" />
      </linearGradient>
    </defs>

    <!-- Corps du canard — translate(0,10) quand crowned pour laisser
         de la place à la couronne sans toucher au glyph -->
    <g :transform="crowned ? 'translate(0,10)' : undefined">
      <!-- ombre au sol -->
      <ellipse cx="32" cy="44" rx="26" ry="3.5" fill="#1F2933" opacity=".12" />
      <!-- corps -->
      <ellipse cx="30" cy="32" rx="24" ry="11" fill="var(--body)" />
      <!-- ventre -->
      <ellipse cx="26" cy="35" rx="16" ry="5" fill="var(--belly)" opacity=".7" />
      <!-- queue -->
      <path d="M52 28 Q60 24 58 32 Q54 32 52 30 Z" fill="var(--shade)" />

      <!-- Tête : éveillé (cy=20 r=11) ou endormi (cy=22 r=10) -->
      <template v-if="asleep">
        <circle cx="46" cy="22" r="10" fill="var(--body)" />
        <!-- Bec endormi : décalé vers le bas et plus large -->
        <path d="M55 21 L62 23.5 L55 26 Z" fill="#F2B400" />
        <!-- Œil fermé : courbe -->
        <path d="M44 19 Q47 17 50 19" stroke="#1F2933" stroke-width="1.4" fill="none" stroke-linecap="round" />
      </template>
      <template v-else>
        <circle cx="46" cy="20" r="11" fill="var(--body)" />
        <!-- Bec (toujours jaune duck-deep, signature visuelle) -->
        <path d="M55 19 L62 22 L55 25 Z" fill="#F2B400" />
        <path d="M55 21 L60 22.5 L55 24 Z" fill="#C9881A" opacity=".5" />
        <!-- Œil ouvert -->
        <circle cx="49" cy="17" r="2.2" fill="#1F2933" />
        <circle cx="49.7" cy="16.4" r=".7" fill="#FAF3E3" />
      </template>

      <!-- aile -->
      <path d="M28 26 Q34 22 40 28 Q36 32 28 30 Z" fill="var(--shade)" />
    </g>

    <!-- Couronne (champagne, légèrement penchée pour ne pas avoir
         l'air d'un chapeau centré) -->
    <g v-if="crowned" transform="translate(36,11) rotate(-6 9 5)">
      <path
        d="M0 6 L3 0 L6 6 L9 0 L12 6 L15 0 L18 6 L18 10 L0 10 Z"
        fill="#E8C77A"
        stroke="#C9A347"
        stroke-width="0.5"
        stroke-linejoin="round"
      />
      <circle cx="3"  cy="0.5" r="1.2" fill="#C9A347" />
      <circle cx="9"  cy="0.5" r="1.2" fill="#C9A347" />
      <circle cx="15" cy="0.5" r="1.2" fill="#C9A347" />
    </g>
  </svg>
</template>
