<script setup lang="ts">
/**
 * MareTVInstructionsView — sheet mobile expliquant Mare TV.
 *
 * Reproduit design/Mosaic 6 screens.html column 6 :
 *   - Header "Mare TV" + sous-titre
 *   - Illustration TV avec un canard à l'écran et badge "live"
 *   - 3 cards "explain" numérotées (qu'est-ce que c'est / comment
 *     l'installer / facultative)
 *   - QR code généré côté client pointant sur window.location.origin
 *     + '/mare-tv' — l'invité scanne avec un autre device pour
 *     lancer le diffuseur
 *   - Retour bas de page
 */
import { ref, computed, onMounted } from 'vue'
import QRCode from 'qrcode'
import Duck from '../components/Duck.vue'

const qrCanvas = ref<HTMLCanvasElement | null>(null)
const tvUrl = computed(() =>
  typeof window !== 'undefined' ? `${window.location.origin}/mare-tv` : '/mare-tv',
)

onMounted(async () => {
  if (!qrCanvas.value) return
  try {
    await QRCode.toCanvas(qrCanvas.value, tvUrl.value, {
      width: 200,
      margin: 1,
      color: { dark: '#1F2933', light: '#FFFFFF' },
      errorCorrectionLevel: 'M',
    })
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error('[MareTVInstructions] QR generation failed', err)
  }
})

function copyUrl() {
  if (typeof navigator === 'undefined' || !navigator.clipboard) return
  void navigator.clipboard.writeText(tvUrl.value).catch(() => {
    /* silent */
  })
}
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="px-5 pt-4 pb-1.5">
      <RouterLink
        to="/upload"
        class="font-sans text-pond-mid text-sm inline-flex items-center gap-1 hover:text-pond-deep"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="15 6 9 12 15 18" />
        </svg>
        Retour
      </RouterLink>
    </header>

    <div class="px-5 pt-2 pb-2 text-center">
      <h1 class="font-display text-3xl text-pond-deep leading-none">Mare TV</h1>
      <p class="font-sans text-sm text-ink-soft mt-1.5">
        L'écran de la mare en direct
      </p>
    </div>

    <!-- ─── TV illustration ─────────────────────────────────── -->
    <div class="tv-stage mx-4 mt-3">
      <span class="tv-live">
        <span class="tv-live__dot" /> live
      </span>
      <div class="tv-set">
        <div class="tv-duck">
          <Duck color="yellow" :size="44" />
        </div>
      </div>
      <div class="tv-base" />
    </div>

    <!-- ─── 3 explain cards ─────────────────────────────────── -->
    <div class="explain mx-4 mt-3.5">
      <div class="explain__head">
        <span class="explain__num">1</span>
        <h3>Qu'est-ce que c'est&nbsp;?</h3>
      </div>
      <p class="explain__body">
        Mare TV diffuse les barbotages en direct sur un écran de la salle.
        C'est un avant-goût de la soirée — sans pouvoir zoomer ni revenir en
        arrière.
      </p>
    </div>

    <div class="explain mx-4 mt-2.5">
      <div class="explain__head">
        <span class="explain__num">2</span>
        <h3>Comment l'installer&nbsp;?</h3>
      </div>
      <p class="explain__body">
        Scanne ce QR code avec un autre appareil connecté à la TV pour lancer
        le diffuseur.
      </p>
      <div class="qr-wrap">
        <canvas ref="qrCanvas" class="qr-canvas" />
      </div>
      <button
        type="button"
        class="qr-copy"
        @click="copyUrl"
      >
        ou copie l'URL : <code>{{ tvUrl }}</code>
      </button>
    </div>

    <div class="explain mx-4 mt-2.5 mb-6">
      <div class="explain__head">
        <span class="explain__num">3</span>
        <h3>Cette fonctionnalité<br />est facultative</h3>
      </div>
      <p class="explain__body">
        Si la salle n'a pas d'écran disponible, ce n'est pas grave — tous les
        souvenirs se révèleront sur ton téléphone.
      </p>
    </div>

    <div class="text-center mt-auto pb-7">
      <RouterLink to="/upload" class="font-sans text-pond-mid text-sm hover:text-pond-deep">
        ← Retour à la mare
      </RouterLink>
    </div>
  </main>
</template>

<style scoped>
/* ─── TV illustration ──────────────────────────────────────── */
.tv-stage {
  height: 170px;
  border-radius: 18px;
  background: linear-gradient(180deg, var(--pond-pale) 0%, #d6ecf3 100%);
  border: 1px solid rgba(169, 216, 229, 0.55);
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}

.tv-live {
  position: absolute;
  top: 14px;
  right: 14px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-family: var(--mono);
  font-size: 9px;
  font-weight: 500;
  color: white;
  background: rgba(0, 0, 0, 0.5);
  padding: 3px 7px;
  border-radius: 6px;
  letter-spacing: 0.06em;
  text-transform: uppercase;
}

.tv-live__dot {
  width: 5px;
  height: 5px;
  border-radius: 50%;
  background: var(--rec-red);
  box-shadow: 0 0 0 2px rgba(225, 75, 63, 0.3);
}

.tv-set {
  position: relative;
  width: 130px;
  height: 88px;
  background: linear-gradient(180deg, #1a3a4a, #0a2230);
  border-radius: 8px;
  border: 3px solid var(--pond-deep);
  box-shadow: 0 6px 14px -4px rgba(14, 79, 107, 0.4);
}

.tv-set::before {
  content: '';
  position: absolute;
  inset: 6px;
  border-radius: 4px;
  background:
    radial-gradient(circle at 30% 30%, rgba(255, 201, 60, 0.4) 0 22%, transparent 60%),
    linear-gradient(180deg, #1f5c79, #0a2a3d);
}

.tv-set::after {
  content: '';
  position: absolute;
  bottom: -14px;
  left: 50%;
  transform: translateX(-50%);
  width: 46px;
  height: 6px;
  background: var(--pond-deep);
  border-radius: 0 0 6px 6px;
}

.tv-base {
  position: absolute;
  bottom: 18px;
  left: 50%;
  transform: translateX(-50%);
  width: 80px;
  height: 5px;
  background: #5a6b7a;
  border-radius: 2px;
}

.tv-duck {
  position: absolute;
  top: 14px;
  left: 50%;
  transform: translateX(-50%);
  z-index: 2;
}

/* ─── Explain cards ────────────────────────────────────────── */
.explain {
  padding: 12px 14px;
  background: var(--cream-deep);
  border: 1px solid var(--cream-line);
  border-radius: 14px;
}

.explain__head {
  display: flex;
  align-items: center;
  margin-bottom: 6px;
  gap: 8px;
}

.explain__num {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--pond-deep);
  color: white;
  font-family: var(--mono);
  font-weight: 500;
  font-size: 10px;
  flex-shrink: 0;
}

.explain__head h3 {
  margin: 0;
  font-family: var(--sans);
  font-weight: 600;
  font-size: 13px;
  color: var(--ink);
  line-height: 1.2;
}

.explain__body {
  margin: 0;
  font-family: var(--sans);
  font-size: 12px;
  color: var(--ink-soft);
  line-height: 1.4;
}

/* ─── QR ───────────────────────────────────────────────────── */
.qr-wrap {
  margin: 14px auto 10px;
  width: 220px;
  height: 220px;
  background: white;
  border: 1px solid var(--cream-line);
  border-radius: 12px;
  padding: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 4px 14px -8px rgba(14, 79, 107, 0.25);
}

.qr-canvas {
  display: block;
  /* canvas a une taille intrinsèque (200x200 depuis QRCode.toCanvas) ;
     on laisse le browser le rendre crisp à sa taille naturelle */
  width: auto;
  height: auto;
  max-width: 100%;
  max-height: 100%;
}

.qr-copy {
  display: block;
  margin: 0 auto;
  background: transparent;
  border: none;
  font-family: var(--sans);
  font-size: 10px;
  color: var(--ink-soft);
  cursor: pointer;
  text-align: center;
}

.qr-copy:hover {
  color: var(--pond-deep);
}

.qr-copy code {
  font-family: var(--mono);
  font-size: 10px;
  color: var(--pond-deep);
  word-break: break-all;
}
</style>
