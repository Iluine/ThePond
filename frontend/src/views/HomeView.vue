<script setup lang="ts">
/**
 * Showcase du design system v0.2 (prompt 1 du bootstrap).
 * Cette vue sera remplacée par le vrai écran d'accueil (Welcome) au prompt 7.
 */
import PrimaryButton from '../components/PrimaryButton.vue'
import SecondaryButton from '../components/SecondaryButton.vue'
import PondCounter from '../components/PondCounter.vue'
import MareTVPill from '../components/MareTVPill.vue'
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
        Bootstrap · Prompt 1 · Design system
      </p>
    </footer>
  </main>
</template>
