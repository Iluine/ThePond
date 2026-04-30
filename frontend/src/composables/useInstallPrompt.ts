/**
 * useInstallPrompt — gère le bouton "Installer sur mon téléphone".
 *
 * Trois cas :
 *   - Chrome / Android : capture beforeinstallprompt, bouton actif,
 *     click → prompt natif, on suit l'acceptation via appinstalled
 *   - iOS Safari : pas d'événement, pas de prompt programmatique. Le
 *     bouton ouvrira la sheet d'instructions (à implémenter au prompt 14)
 *   - Déjà installé (mode standalone) : on cache le bouton via canInstall
 *
 * Le composable monte les listeners au premier appel et les laisse
 * en place pour toute la durée de vie de l'app — il n'y a pas de
 * unmount nécessaire (events globaux qui survivent au navigation).
 */
import { ref, computed, onMounted } from 'vue'

// Type minimal — l'événement n'est pas dans les TypeScript libs standard
interface BeforeInstallPromptEvent extends Event {
  readonly platforms: string[]
  prompt(): Promise<void>
  readonly userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>
}

const deferredPrompt = ref<BeforeInstallPromptEvent | null>(null)
const installed = ref(false)
let listenersAttached = false

function attachListeners() {
  if (listenersAttached) return
  listenersAttached = true

  window.addEventListener('beforeinstallprompt', (e) => {
    e.preventDefault()
    deferredPrompt.value = e as BeforeInstallPromptEvent
  })

  window.addEventListener('appinstalled', () => {
    installed.value = true
    deferredPrompt.value = null
  })

  // Détecte si on tourne déjà en mode standalone (PWA installée).
  const standaloneQuery = window.matchMedia('(display-mode: standalone)')
  if (standaloneQuery.matches) {
    installed.value = true
  }
  standaloneQuery.addEventListener('change', (e) => {
    if (e.matches) installed.value = true
  })
}

export function useInstallPrompt() {
  onMounted(attachListeners)

  /** L'install programmatique est dispo (Chrome/Android typiquement). */
  const canInstallProgrammatically = computed(
    () => deferredPrompt.value !== null && !installed.value,
  )

  /** Sur iOS Safari : pas de prompt mais on veut quand même montrer
   *  le bouton (qui ouvrira une sheet d'instructions au prompt 14). */
  const isIosLike = computed(() => {
    if (typeof navigator === 'undefined') return false
    // navigator.standalone existe sur iOS Safari
    const nav = navigator as Navigator & { standalone?: boolean }
    if (nav.standalone === true) return false // déjà installée
    const ua = navigator.userAgent
    return /iPad|iPhone|iPod/.test(ua) && !('MSStream' in window)
  })

  /** Le bouton doit-il s'afficher ? Vrai si : install programmatique
   *  dispo, OU iOS (sheet manuelle), ET pas déjà installé. */
  const canInstall = computed(
    () => !installed.value && (canInstallProgrammatically.value || isIosLike.value),
  )

  async function install(): Promise<'accepted' | 'dismissed' | 'unavailable' | 'ios'> {
    if (installed.value) return 'unavailable'
    if (deferredPrompt.value) {
      const ev = deferredPrompt.value
      await ev.prompt()
      const choice = await ev.userChoice
      deferredPrompt.value = null
      return choice.outcome
    }
    if (isIosLike.value) return 'ios'
    return 'unavailable'
  }

  return { canInstall, installed, install, isIosLike }
}
