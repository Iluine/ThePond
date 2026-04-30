<script setup lang="ts">
/**
 * ClipCaptureView — capture vidéo in-app via MediaRecorder.
 *
 * Flux : permission → preview live → record (max 15s) → upload → confirmation.
 *
 * Compression côté client : on contraint getUserMedia à 1080p max et le
 * MediaRecorder à 5 Mbps. Un clip 15s tient ainsi sous ~10 Mo, bien sous
 * la limite serveur de 100 Mo (cf. backend/src/routes/clips.rs).
 *
 * Codec : Safari accepte mp4/avc1, Chrome/Firefox webm/vp9 ou vp8. On
 * essaie dans cet ordre via MediaRecorder.isTypeSupported et on retombe
 * sur la valeur par défaut du navigateur si rien ne matche.
 */
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '../stores/user'
import { useUploadQueueStore } from '../stores/uploadQueue'

const router = useRouter()
const userStore = useUserStore()
const uploadQueue = useUploadQueueStore()

const MAX_SECONDS = 15
const VIDEO_BITRATE = 5_000_000

// ─── State machine ──────────────────────────────────────────────
type State = 'asking' | 'idle' | 'recording' | 'uploading' | 'error'
const state = ref<State>('asking')
const errorMessage = ref<string | null>(null)

// ─── Refs DOM + media ──────────────────────────────────────────
const videoEl = ref<HTMLVideoElement | null>(null)
let stream: MediaStream | null = null
let recorder: MediaRecorder | null = null
let chunks: Blob[] = []
let timerId: number | null = null
let startedAt = 0
const elapsed = ref(0) // secondes écoulées dans l'enregistrement courant

const remaining = computed(() => Math.max(0, MAX_SECONDS - elapsed.value))
const progressPercent = computed(() =>
  Math.min(100, (elapsed.value / MAX_SECONDS) * 100),
)

// ─── Codec selection ────────────────────────────────────────────
function pickMimeType(): string {
  const candidates = [
    'video/mp4;codecs=avc1,mp4a.40.2',
    'video/webm;codecs=vp9,opus',
    'video/webm;codecs=vp8,opus',
    'video/webm',
  ]
  for (const t of candidates) {
    if (MediaRecorder.isTypeSupported(t)) return t
  }
  return ''
}

function extensionFor(mimeType: string): string {
  if (mimeType.startsWith('video/mp4')) return 'mp4'
  if (mimeType.startsWith('video/webm')) return 'webm'
  return 'mp4'
}

// ─── Lifecycle ──────────────────────────────────────────────────
onMounted(async () => {
  if (!userStore.isAuthenticated) {
    router.replace('/')
    return
  }
  await requestPermission()
})

onBeforeUnmount(() => {
  cleanup()
})

async function requestPermission() {
  try {
    state.value = 'asking'
    elapsed.value = 0
    stream = await navigator.mediaDevices.getUserMedia({
      video: {
        facingMode: 'environment',
        width: { ideal: 1920, max: 1920 },
        height: { ideal: 1080, max: 1080 },
      },
      audio: true,
    })
    if (videoEl.value) {
      videoEl.value.srcObject = stream
      // Mute le preview pour éviter le larsen — l'audio est capté
      // par le MediaRecorder via le stream, pas via l'output.
      videoEl.value.muted = true
      await videoEl.value.play().catch(() => {
        /* autoplay parfois bloqué — on laisse le user cliquer */
      })
    }
    state.value = 'idle'
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : 'Impossible d’accéder à la caméra'
    state.value = 'error'
  }
}

function startRecording() {
  if (!stream || state.value !== 'idle') return
  errorMessage.value = null
  chunks = []
  const mimeType = pickMimeType()
  try {
    recorder = new MediaRecorder(
      stream,
      mimeType
        ? { mimeType, videoBitsPerSecond: VIDEO_BITRATE }
        : { videoBitsPerSecond: VIDEO_BITRATE },
    )
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : 'MediaRecorder indisponible'
    state.value = 'error'
    return
  }

  recorder.ondataavailable = (e) => {
    if (e.data && e.data.size > 0) chunks.push(e.data)
  }
  recorder.onstop = handleStop
  recorder.onerror = (ev) => {
    errorMessage.value = `Erreur d’enregistrement (${(ev as ErrorEvent).message ?? 'inconnu'})`
    state.value = 'error'
  }

  recorder.start()
  state.value = 'recording'
  startedAt = Date.now()
  elapsed.value = 0
  timerId = window.setInterval(tick, 100)
}

function stopRecording() {
  if (state.value !== 'recording') return
  if (timerId !== null) {
    clearInterval(timerId)
    timerId = null
  }
  if (recorder && recorder.state !== 'inactive') {
    recorder.stop()
  }
}

function tick() {
  elapsed.value = (Date.now() - startedAt) / 1000
  if (elapsed.value >= MAX_SECONDS) {
    stopRecording()
  }
}

function handleStop() {
  // Optimistic UI : on enqueue dans la queue store et on file vers
  // /confirmation tout de suite. Le store gère le POST en background.
  const mimeType = recorder?.mimeType ?? 'video/mp4'
  const ext = extensionFor(mimeType)
  const blob = new Blob(chunks, { type: mimeType })
  const file = new File([blob], `clip-${Date.now()}.${ext}`, { type: mimeType })
  const duration = elapsed.value

  const item = uploadQueue.enqueue(file, 'clip', { durationSeconds: duration })
  cleanup()
  router.push(`/confirmation?type=clip&itemId=${item.id}`)
}

function cleanup() {
  if (timerId !== null) {
    clearInterval(timerId)
    timerId = null
  }
  if (recorder && recorder.state !== 'inactive') {
    recorder.stop()
  }
  recorder = null
  if (stream) {
    for (const track of stream.getTracks()) track.stop()
    stream = null
  }
}

function cancel() {
  cleanup()
  router.push('/upload')
}

function formatTime(s: number): string {
  return `0:${Math.floor(s).toString().padStart(2, '0')}`
}
</script>

<template>
  <div class="capture">
    <!-- Live preview (toujours mounté pour que srcObject persiste) -->
    <video
      ref="videoEl"
      class="capture__video"
      :class="{ 'capture__video--hidden': state === 'asking' || state === 'error' }"
      playsinline
      autoplay
      muted
    />

    <!-- État ASKING permission -->
    <div v-if="state === 'asking'" class="capture__overlay">
      <p class="font-display text-2xl text-cream">Accès à la caméra…</p>
      <p class="font-mono text-xs text-cream/70 mt-2 uppercase tracking-wider">
        autorise quand le navigateur demande
      </p>
    </div>

    <!-- État ERROR -->
    <div v-else-if="state === 'error'" class="capture__overlay">
      <p class="font-display text-2xl text-cream">Hop, problème.</p>
      <p class="font-sans text-sm text-cream/80 mt-2 max-w-xs text-center">
        {{ errorMessage }}
      </p>
      <div class="flex gap-3 mt-6">
        <button class="btn-secondary-overlay" @click="cancel">Retour</button>
        <button class="btn-primary-overlay" @click="requestPermission">
          Réessayer
        </button>
      </div>
    </div>

    <!-- HUD top — close + countdown ring -->
    <div v-if="state === 'idle' || state === 'recording'" class="capture__hud-top">
      <button
        class="capture__close"
        aria-label="Annuler"
        @click="cancel"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <line x1="6" y1="6" x2="18" y2="18" />
          <line x1="6" y1="18" x2="18" y2="6" />
        </svg>
      </button>
      <div v-if="state === 'recording'" class="capture__timer">
        <span class="capture__rec-dot" /> {{ formatTime(elapsed) }} · {{ MAX_SECONDS }}s
      </div>
    </div>

    <!-- Progress bar -->
    <div
      v-if="state === 'recording'"
      class="capture__progress"
      :style="{ width: progressPercent + '%' }"
    />

    <!-- HUD bottom — record / stop -->
    <div v-if="state === 'idle' || state === 'recording'" class="capture__hud-bottom">
      <button
        v-if="state === 'idle'"
        class="capture__record"
        aria-label="Démarrer l’enregistrement"
        @click="startRecording"
      />
      <button
        v-else
        class="capture__stop"
        aria-label="Arrêter l’enregistrement"
        @click="stopRecording"
      />
      <p class="capture__hint font-mono text-[11px] uppercase tracking-wider">
        {{
          state === 'idle'
            ? 'Appuie pour cancaner'
            : `${remaining.toFixed(0)}s restantes`
        }}
      </p>
    </div>

    <!-- État UPLOADING -->
    <div v-if="state === 'uploading'" class="capture__overlay">
      <p class="font-display text-2xl text-cream">Splash !</p>
      <p class="font-mono text-xs text-cream/70 mt-2 uppercase tracking-wider">
        envoi en cours…
      </p>
    </div>
  </div>
</template>

<style scoped>
.capture {
  position: fixed;
  inset: 0;
  background: #000;
  overflow: hidden;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
}

.capture__video {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  background: #111;
}
.capture__video--hidden {
  visibility: hidden;
}

.capture__overlay {
  position: relative;
  z-index: 2;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  background: rgba(14, 79, 107, 0.85);
  border-radius: 22px;
  padding: 28px 32px;
  max-width: 88%;
}

.capture__hud-top {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  padding: 18px 16px;
  background: linear-gradient(180deg, rgba(0, 0, 0, 0.55) 0%, transparent 100%);
  display: flex;
  justify-content: space-between;
  align-items: center;
  z-index: 2;
}

.capture__close {
  width: 38px;
  height: 38px;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.15);
  border: none;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  backdrop-filter: blur(6px);
}

.capture__timer {
  font-family: var(--mono);
  font-size: 13px;
  letter-spacing: 0.04em;
  background: rgba(0, 0, 0, 0.5);
  padding: 6px 12px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.capture__rec-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--rec-red);
  animation: rec-pulse 1.2s ease-in-out infinite;
}

@keyframes rec-pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.85); }
}

.capture__progress {
  position: absolute;
  top: 0;
  left: 0;
  height: 3px;
  background: var(--rec-red);
  transition: width 0.1s linear;
  z-index: 3;
}

.capture__hud-bottom {
  position: absolute;
  bottom: 0;
  left: 0;
  right: 0;
  padding: 24px 16px 36px;
  background: linear-gradient(0deg, rgba(0, 0, 0, 0.6) 0%, transparent 100%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  z-index: 2;
}

.capture__record {
  width: 76px;
  height: 76px;
  border-radius: 50%;
  background: var(--rec-red);
  border: 5px solid white;
  cursor: pointer;
  transition: transform 0.1s ease;
}
.capture__record:active { transform: scale(0.94); }

.capture__stop {
  width: 76px;
  height: 76px;
  border-radius: 16px;
  background: white;
  border: 5px solid var(--rec-red);
  cursor: pointer;
  transition: transform 0.1s ease;
}
.capture__stop:active { transform: scale(0.94); }

.capture__hint {
  color: rgba(255, 255, 255, 0.85);
}

.btn-primary-overlay {
  background: var(--duck);
  color: var(--ink);
  font-family: var(--sans);
  font-weight: 700;
  font-size: 14px;
  border: none;
  border-radius: 14px;
  padding: 12px 22px;
  box-shadow: 0 4px 0 var(--duck-deep);
  cursor: pointer;
}

.btn-secondary-overlay {
  background: rgba(255, 255, 255, 0.15);
  color: white;
  font-family: var(--sans);
  font-weight: 600;
  font-size: 14px;
  border: 1px solid rgba(255, 255, 255, 0.25);
  border-radius: 14px;
  padding: 12px 22px;
  cursor: pointer;
}
</style>
