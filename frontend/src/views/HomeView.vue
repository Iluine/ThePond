<script setup lang="ts">
// Hello World du bootstrap : header "The Pond." en Caprasimo + un canard SVG
// volontairement basique. Le vrai design system v0.2 (4 variantes de couleur,
// crowned, glow) sera branché dans le composant Duck.vue paramétrique au
// prompt suivant.
</script>

<template>
  <main
    class="min-h-screen flex flex-col items-center justify-center gap-8 p-8 text-center"
  >
    <h1 class="font-display text-7xl text-pond-deep leading-none">
      The <span class="text-duck-deep">Pond</span>.
    </h1>

    <p class="font-mono text-xs uppercase tracking-widest text-ink-soft">
      Bootstrap · Hello World
    </p>

    <!-- Canard provisoire : un rond Duck Yellow + bec triangulaire
         Pond Deep + un œil. À remplacer par <Duck /> au prompt suivant. -->
    <svg
      width="160"
      height="120"
      viewBox="0 0 64 50"
      class="drop-shadow-md"
      aria-label="Canard"
    >
      <ellipse cx="32" cy="44" rx="26" ry="3.5" fill="#1F2933" opacity=".12" />
      <ellipse cx="30" cy="32" rx="24" ry="11" fill="var(--duck)" />
      <circle cx="46" cy="20" r="11" fill="var(--duck)" />
      <path d="M55 19 L62 22 L55 25 Z" fill="var(--duck-deep)" />
      <circle cx="49" cy="17" r="2.2" fill="var(--ink)" />
    </svg>

    <p class="font-sans text-sm text-ink-soft max-w-md">
      Si tu vois ce texte en
      <span class="font-display text-pond-deep">Caprasimo</span>
      au-dessus, le canard en jaune et ces lignes en
      <span class="font-mono">DM Mono / DM Sans</span> —
      Tailwind, les tokens et les polices sont OK.
    </p>
  </main>
</template>
