<script setup lang="ts">
/**
 * InstallSheet — sheet bottom-up qui explique aux utilisateurs iOS
 * comment installer la PWA manuellement (Safari ne déclenche pas
 * `beforeinstallprompt`).
 *
 * Apparaît à la demande depuis WelcomeView quand
 * `useInstallPrompt.install()` retourne `'ios'`. Backdrop tap = close,
 * Escape = close, slide-up animation. 3 étapes numérotées avec
 * illustrations SVG natives iOS (icône Share + icône Add).
 */
import { onMounted, onBeforeUnmount } from 'vue'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ close: [] }>()

function close() { emit('close') }

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape' && props.visible) close()
}

onMounted(() => window.addEventListener('keydown', onKey))
onBeforeUnmount(() => window.removeEventListener('keydown', onKey))
</script>

<template>
  <Teleport to="body">
    <Transition name="sheet">
      <div v-if="visible" class="install-sheet">
        <div class="backdrop" @click="close" />
        <section class="sheet" role="dialog" aria-modal="true" aria-labelledby="install-title">
          <div class="grabber" />

          <header class="sheet__head">
            <h2 id="install-title">Installer The Pond. sur iPhone</h2>
            <button type="button" class="close-btn" aria-label="Fermer" @click="close">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
                <line x1="6" y1="6" x2="18" y2="18" />
                <line x1="6" y1="18" x2="18" y2="6" />
              </svg>
            </button>
          </header>

          <p class="sheet__intro">
            Safari ne propose pas l'installation automatique. Trois petits
            gestes et tu auras la mare sur ton écran d'accueil :
          </p>

          <ol class="steps">
            <li class="step">
              <span class="step__num">1</span>
              <div class="step__body">
                <p class="step__text">
                  Touche le bouton <b>Partager</b> en bas de Safari.
                </p>
                <span class="step__icon">
                  <svg width="22" height="28" viewBox="0 0 22 28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="11" width="16" height="14" rx="2" />
                    <line x1="11" y1="2" x2="11" y2="17" />
                    <polyline points="6 7 11 2 16 7" />
                  </svg>
                </span>
              </div>
            </li>

            <li class="step">
              <span class="step__num">2</span>
              <div class="step__body">
                <p class="step__text">
                  Fais défiler et touche <b>« Sur l'écran d'accueil »</b>.
                </p>
                <span class="step__icon step__icon--add">
                  <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <rect x="3" y="3" width="18" height="18" rx="3" />
                    <line x1="12" y1="8" x2="12" y2="16" />
                    <line x1="8" y1="12" x2="16" y2="12" />
                  </svg>
                </span>
              </div>
            </li>

            <li class="step">
              <span class="step__num">3</span>
              <div class="step__body">
                <p class="step__text">
                  Touche <b>Ajouter</b>. La mare apparaît avec son icône
                  canard 🐤 — tu n'auras plus jamais à chercher l'URL.
                </p>
              </div>
            </li>
          </ol>

          <button type="button" class="dismiss-btn" @click="close">
            Compris
          </button>
        </section>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.install-sheet {
  position: fixed;
  inset: 0;
  z-index: 200;
  display: flex;
  align-items: flex-end;
  justify-content: center;
}

.backdrop {
  position: absolute;
  inset: 0;
  background: rgba(14, 79, 107, 0.55);
  backdrop-filter: blur(2px);
}

.sheet {
  position: relative;
  width: 100%;
  max-width: 460px;
  background: var(--cream);
  border-radius: 24px 24px 0 0;
  padding: 12px 22px 28px;
  box-shadow: 0 -10px 40px -10px rgba(14, 79, 107, 0.45);
  /* Safe area iOS */
  padding-bottom: calc(28px + env(safe-area-inset-bottom));
}

.grabber {
  width: 44px;
  height: 4px;
  background: var(--cream-line);
  border-radius: 2px;
  margin: 0 auto 14px;
}

.sheet__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  margin-bottom: 8px;
}

.sheet__head h2 {
  margin: 0;
  font-family: var(--display);
  font-size: 22px;
  color: var(--pond-deep);
  line-height: 1.1;
  flex: 1;
}

.close-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background: var(--cream-deep);
  color: var(--ink-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.sheet__intro {
  margin: 0 0 18px;
  font-family: var(--sans);
  font-size: 14px;
  color: var(--ink-soft);
  line-height: 1.4;
}

.steps {
  list-style: none;
  padding: 0;
  margin: 0 0 20px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.step {
  display: flex;
  gap: 12px;
  padding: 14px;
  background: var(--cream-deep);
  border: 1px solid var(--cream-line);
  border-radius: 14px;
}

.step__num {
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background: var(--pond-deep);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--mono);
  font-weight: 600;
  font-size: 13px;
  flex-shrink: 0;
}

.step__body {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 12px;
}

.step__text {
  margin: 0;
  font-family: var(--sans);
  font-size: 14px;
  color: var(--ink);
  line-height: 1.4;
  flex: 1;
}

.step__text b {
  color: var(--pond-deep);
  font-weight: 600;
}

.step__icon {
  flex-shrink: 0;
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: white;
  border: 1px solid var(--cream-line);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--pond-mid);
}

.step__icon--add {
  color: var(--duck-deep);
}

.dismiss-btn {
  display: block;
  width: 100%;
  padding: 14px;
  background: var(--duck);
  color: var(--ink);
  border: none;
  border-radius: 16px;
  font-family: var(--sans);
  font-weight: 700;
  font-size: 15px;
  cursor: pointer;
  box-shadow: 0 4px 0 var(--duck-deep);
  transition: transform 0.1s ease, box-shadow 0.1s ease;
}

.dismiss-btn:active {
  transform: translateY(2px);
  box-shadow: 0 2px 0 var(--duck-deep);
}

/* ─── Slide-up animation ─────────────────────────────────────── */
.sheet-enter-active,
.sheet-leave-active {
  transition: opacity 0.25s ease;
}
.sheet-enter-active .sheet,
.sheet-leave-active .sheet {
  transition: transform 0.3s cubic-bezier(0.2, 0.9, 0.3, 1.05);
}

.sheet-enter-from,
.sheet-leave-to {
  opacity: 0;
}
.sheet-enter-from .sheet,
.sheet-leave-to .sheet {
  transform: translateY(100%);
}
</style>
