<script setup lang="ts">
/**
 * Showcase du design system v0.2 (prompt 1 du bootstrap).
 * Cette vue sera remplacée par le vrai écran d'accueil (Welcome) au prompt 7.
 */
import PrimaryButton from '../components/PrimaryButton.vue'
import SecondaryButton from '../components/SecondaryButton.vue'
import PondCounter from '../components/PondCounter.vue'
import MareTVPill from '../components/MareTVPill.vue'
import Duck from '../components/Duck.vue'
import Pond, { type PondDuck } from '../components/Pond.vue'

// ─── Helpers pour la démo Pond ──────────────────────────────────
function makeFakeDucks(n: number): PondDuck[] {
  // Distribution v0.2 : 80% yellow, 15% white, 4% blue, 1% rainbow
  const out: PondDuck[] = []
  for (let i = 0; i < n; i++) {
    const r = Math.random()
    const color = r < 0.80 ? 'yellow' : r < 0.95 ? 'white' : r < 0.99 ? 'blue' : 'rainbow'
    out.push({ id: i, color })
  }
  return out
}

const teaserDucks = makeFakeDucks(15)
const largeDucks = makeFakeDucks(30)
</script>

<template>
  <main class="min-h-screen p-6 md:p-10 max-w-3xl mx-auto space-y-12">
    <header class="text-center space-y-3">
      <h1 class="font-display text-6xl text-pond-deep leading-none">
        The <span class="text-duck-deep">Pond</span>.
      </h1>
      <p class="font-mono text-xs uppercase tracking-widest text-ink-soft">
        Design system v0.2 · Showcase
      </p>
    </header>

    <!-- ─── Pond ────────────────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">Pond</h2>

      <!-- Mini teaser 348×100 (Welcome v2) -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-3">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Teaser · 348×100 · jour · couple visible
        </div>
        <div class="flex justify-center">
          <Pond :ducks="teaserDucks" />
        </div>
        <p class="font-mono text-[10px] text-ink-soft text-center">
          {{ teaserDucks.length }} canards barbotent · couple au centre
        </p>
      </div>

      <!-- Pond plus grand 600×220 -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-3">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Mare étendue · 600×220 · jour
        </div>
        <div class="flex justify-center overflow-x-auto">
          <Pond :ducks="largeDucks" :width="600" :height="220" />
        </div>
      </div>

      <!-- Mode nuit (Mare TV) -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-3">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Mode nuit · 600×220 · glow auto · couple champagne
        </div>
        <div class="flex justify-center overflow-x-auto">
          <Pond :ducks="largeDucks" :width="600" :height="220" night-mode />
        </div>
      </div>

      <!-- Sans couple -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-3">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Sans couple · zone centrale ouverte
        </div>
        <div class="flex justify-center">
          <Pond :ducks="teaserDucks" :couple-visible="false" />
        </div>
      </div>
    </section>

    <!-- ─── Duck ────────────────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">Duck</h2>

      <!-- Les 4 variantes de couleur -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-4">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          4 variantes · ratio v0.2 (80 / 15 / 4 / 1)
        </div>
        <div class="flex items-end justify-around gap-4 flex-wrap">
          <div class="flex flex-col items-center gap-2">
            <Duck color="yellow" />
            <span class="font-mono text-[10px] uppercase text-ink-soft">yellow · 80%</span>
          </div>
          <div class="flex flex-col items-center gap-2">
            <Duck color="white" />
            <span class="font-mono text-[10px] uppercase text-ink-soft">white · 15%</span>
          </div>
          <div class="flex flex-col items-center gap-2">
            <Duck color="blue" />
            <span class="font-mono text-[10px] uppercase text-ink-soft">blue · 4%</span>
          </div>
          <div class="flex flex-col items-center gap-2">
            <Duck color="rainbow" />
            <span class="font-mono text-[10px] uppercase text-ink-soft">rainbow · 1%</span>
          </div>
        </div>
      </div>

      <!-- Le couple : 2 ducks crowned 140px -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-4">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Couple · 2 crowned 140px
        </div>
        <div class="flex items-end justify-center gap-2">
          <Duck color="yellow" :size="140" crowned />
          <Duck color="white" :size="140" crowned />
        </div>
      </div>

      <!-- Glow contre fond pond-deep (mode nuit Mare TV) -->
      <div class="p-5 bg-pond-deep rounded-2xl space-y-4">
        <div class="font-mono text-[10px] uppercase tracking-wider text-pond-light">
          Glow · mode nuit Mare TV
        </div>
        <div class="flex items-end justify-around gap-4 flex-wrap">
          <div class="flex flex-col items-center gap-2">
            <Duck color="yellow" :size="64" glow />
            <span class="font-mono text-[10px] uppercase text-pond-light">invité jaune</span>
          </div>
          <div class="flex flex-col items-center gap-2">
            <Duck color="white" :size="64" glow />
            <span class="font-mono text-[10px] uppercase text-pond-light">invité blanc</span>
          </div>
          <div class="flex flex-col items-center gap-2">
            <Duck color="blue" :size="64" glow />
            <span class="font-mono text-[10px] uppercase text-pond-light">invité bleu</span>
          </div>
          <div class="flex flex-col items-end gap-2">
            <div class="flex items-end gap-1">
              <Duck color="yellow" :size="100" crowned glow />
              <Duck color="white" :size="100" crowned glow />
            </div>
            <span class="font-mono text-[10px] uppercase text-pond-light">couple champagne</span>
          </div>
        </div>
      </div>

      <!-- Variations de tailles -->
      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line space-y-4">
        <div class="font-mono text-[10px] uppercase tracking-wider text-ink-soft">
          Tailles · 24 / 56 (default) / 140
        </div>
        <div class="flex items-end justify-center gap-6">
          <Duck color="yellow" :size="24" />
          <Duck color="yellow" />
          <Duck color="yellow" :size="140" />
        </div>
      </div>
    </section>

    <!-- ─── PondCounter ─────────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">PondCounter</h2>

      <div class="space-y-4 p-5 bg-cream-deep rounded-2xl border border-cream-line">
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">sm</span>
          <PondCounter :current="42" :total="52" />
        </div>
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">sm full</span>
          <PondCounter :current="52" :total="52" />
        </div>
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16 self-start mt-3">lg</span>
          <PondCounter :current="42" :total="52" size="lg" />
        </div>
      </div>
    </section>

    <!-- ─── PrimaryButton ───────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">PrimaryButton</h2>

      <div class="space-y-4 p-5 bg-cream-deep rounded-2xl border border-cream-line">
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">sm · 48</span>
          <div class="flex-1 min-w-[180px]">
            <PrimaryButton size="sm">PASSER AU PROCHAIN PALIER</PrimaryButton>
          </div>
        </div>
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">md · 64</span>
          <div class="flex-1 min-w-[180px]">
            <PrimaryButton>PLONGER DANS LA MARE</PrimaryButton>
          </div>
        </div>
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">lg · 80</span>
          <div class="flex-1 min-w-[180px]">
            <PrimaryButton size="lg">
              <template #icon>
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
                  <circle cx="12" cy="13" r="4" />
                </svg>
              </template>
              CONTINUER À BARBOTER
            </PrimaryButton>
          </div>
        </div>
        <div class="flex items-center gap-4 flex-wrap">
          <span class="font-mono text-xs uppercase tracking-wider text-ink-soft w-16">xl · 112</span>
          <div class="flex-1 min-w-[180px]">
            <PrimaryButton size="xl">
              <template #icon>
                <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
                  <circle cx="12" cy="13" r="4" />
                </svg>
              </template>
              BARBOTER
            </PrimaryButton>
          </div>
        </div>
      </div>
    </section>

    <!-- ─── SecondaryButton ─────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">SecondaryButton</h2>

      <div class="grid grid-cols-2 gap-3 p-5 bg-cream-deep rounded-2xl border border-cream-line">
        <SecondaryButton variant="cream" sub-label="15s">
          <template #icon>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="2" y="6" width="14" height="12" rx="2" />
              <path d="M22 8 L16 12 L22 16 Z" />
            </svg>
          </template>
          CANCANER
        </SecondaryButton>

        <SecondaryButton variant="coral" sub-label="60s">
          <template #icon>
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="9" y="2" width="6" height="12" rx="3" />
              <path d="M5 11a7 7 0 0 0 14 0" />
              <line x1="12" y1="18" x2="12" y2="22" />
            </svg>
          </template>
          FAIRE COIN-COIN
        </SecondaryButton>
      </div>
    </section>

    <!-- ─── MareTVPill ──────────────────────────────────────────── -->
    <section class="space-y-4">
      <h2 class="font-display text-2xl text-pond-deep">MareTVPill</h2>

      <div class="p-5 bg-cream-deep rounded-2xl border border-cream-line text-center">
        <!-- override `to` pour le showcase : la vraie route arrivera au prompt 13 -->
        <MareTVPill to="/" />
      </div>
    </section>

    <footer class="text-center pt-8 pb-4">
      <p class="font-mono text-[10px] uppercase tracking-widest text-ink-soft">
        Bootstrap · Prompt 3 · Pond.vue (la mare)
      </p>
    </footer>
  </main>
</template>
