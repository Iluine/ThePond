<script setup lang="ts">
/**
 * VoiceCaptureView — capture vocale in-app via MediaRecorder.
 *
 * Flux : permission micro → idle (waveform live) → record (max 60s)
 * → upload → confirmation.
 *
 * Waveform live : on branche le stream sur un AudioContext + AnalyserNode
 * et on dessine 60 barres en frequency-domain à 30 fps. Pendant le record
 * comme à l'idle on dessine — visualiser ce qu'on capture rassure.
 *
 * Codec : Safari mp4/aac, Chrome/Firefox webm/opus, fallback navigateur.
 * Le backend (POST /api/voice) accepte les deux familles.
 */
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '../stores/user'
import { useUploadQueueStore } from '../stores/uploadQueue'

const router = useRouter()
const userStore = useUserStore()
const uploadQueue = useUploadQueueStore()

const MAX_SECONDS = 60
const WAVEFORM_BARS = 60

// ─── State ──────────────────────────────────────────────────────
type State = 'asking' | 'idle' | 'recording' | 'uploading' | 'error'
const state = ref<State>('asking')
const errorMessage = ref<string | null>(null)
const caption = ref('')

const canvasEl = ref<HTMLCanvasElement | null>(null)
let stream: MediaStream | null = null
let recorder: MediaRecorder | null = null
let chunks: Blob[] = []
let timerId: number | null = null
let rafId: number | null = null
let startedAt = 0
const elapsed = ref(0)

let audioCtx: AudioContext | null = null
let analyser: AnalyserNode | null = null
let freqBuffer: Uint8Array | null = null

const remaining = computed(() => Math.max(0, MAX_SECONDS - elapsed.value))

// ─── Codec selection ────────────────────────────────────────────
function pickMimeType(): string {
  const candidates = [
    'audio/mp4;codecs=mp4a.40.2',
    'audio/webm;codecs=opus',
    'audio/webm',
    'audio/ogg;codecs=opus',
  ]
  for (const t of candidates) {
    if (MediaRecorder.isTypeSupported(t)) return t
  }
  return ''
}

function extensionFor(mimeType: string): string {
  if (mimeType.startsWith('audio/mp4')) return 'm4a'
  if (mimeType.startsWith('audio/webm')) return 'webm'
  if (mimeType.startsWith('audio/ogg')) return 'ogg'
  return 'webm'
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
    stream = await navigator.mediaDevices.getUserMedia({ audio: true })

    audioCtx = new (window.AudioContext ||
      (window as unknown as { webkitAudioContext: typeof AudioContext })
        .webkitAudioContext)()
    const source = audioCtx.createMediaStreamSource(stream)
    analyser = audioCtx.createAnalyser()
    analyser.fftSize = 128
    freqBuffer = new Uint8Array(analyser.frequencyBinCount)
    source.connect(analyser)

    state.value = 'idle'
    startWaveformLoop()
  } catch (err) {
    errorMessage.value =
      err instanceof Error ? err.message : 'Impossible d’accéder au micro'
    state.value = 'error'
  }
}

// ─── Waveform live ─────────────────────────────────────────────
function startWaveformLoop() {
  function tick() {
    drawWaveform()
    rafId = requestAnimationFrame(tick)
  }
  rafId = requestAnimationFrame(tick)
}

function drawWaveform() {
  const canvas = canvasEl.value
  if (!canvas || !analyser || !freqBuffer) return
  const ctx = canvas.getContext('2d')
  if (!ctx) return

  // Resize handle (display-CSS pixels vs internal — keeps it crisp)
  const dpr = window.devicePixelRatio || 1
  const cssW = canvas.clientWidth
  const cssH = canvas.clientHeight
  if (canvas.width !== cssW * dpr || canvas.height !== cssH * dpr) {
    canvas.width = cssW * dpr
    canvas.height = cssH * dpr
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0)
  ctx.clearRect(0, 0, cssW, cssH)

  analyser.getByteFrequencyData(freqBuffer)

  // On garde WAVEFORM_BARS échantillons depuis le début du buffer,
  // distribués linéairement.
  const barWidth = cssW / WAVEFORM_BARS
  const barGap = 3
  const drawWidth = Math.max(2, barWidth - barGap)
  const isRec = state.value === 'recording'
  const colorActive = '#0E4F6B' // pond-deep
  const colorIdle = '#A9D8E5' // pond-light

  for (let i = 0; i < WAVEFORM_BARS; i++) {
    const idx = Math.floor((i / WAVEFORM_BARS) * freqBuffer.length)
    const v = freqBuffer[idx] / 255 // 0..1
    // Hauteur min 4px pour rester visible même au silence
    const h = Math.max(4, v * cssH * 0.9)
    const x = i * barWidth + barGap / 2
    const y = (cssH - h) / 2
    ctx.fillStyle = isRec ? colorActive : colorIdle
    ctx.beginPath()
    const r = Math.min(drawWidth / 2, h / 2)
    roundedRect(ctx, x, y, drawWidth, h, r)
    ctx.fill()
  }
}

function roundedRect(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  w: number,
  h: number,
  r: number,
) {
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + w, y, x + w, y + h, r)
  ctx.arcTo(x + w, y + h, x, y + h, r)
  ctx.arcTo(x, y + h, x, y, r)
  ctx.arcTo(x, y, x + w, y, r)
  ctx.closePath()
}

// ─── Recording ──────────────────────────────────────────────────
function startRecording() {
  if (!stream || state.value !== 'idle') return
  errorMessage.value = null
  chunks = []
  const mimeType = pickMimeType()
  try {
    recorder = new MediaRecorder(stream, mimeType ? { mimeType } : {})
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
    errorMessage.value = `Erreur (${(ev as ErrorEvent).message ?? 'inconnu'})`
    state.value = 'error'
  }

  recorder.start()
  state.value = 'recording'
  startedAt = Date.now()
  elapsed.value = 0
  timerId = window.setInterval(() => {
    elapsed.value = (Date.now() - startedAt) / 1000
    if (elapsed.value >= MAX_SECONDS) stopRecording()
  }, 100)
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

function handleStop() {
  // Optimistic UI : enqueue + navigation immédiate
  const mimeType = recorder?.mimeType ?? 'audio/webm'
  const ext = extensionFor(mimeType)
  const blob = new Blob(chunks, { type: mimeType })
  const file = new File([blob], `voice-${Date.now()}.${ext}`, { type: mimeType })
  const duration = elapsed.value
  const captionText = caption.value.trim() || undefined

  const item = uploadQueue.enqueue(file, 'voice', {
    durationSeconds: duration,
    caption: captionText,
  })
  cleanup()
  router.push(`/confirmation?type=voice&itemId=${item.id}`)
}

function cleanup() {
  if (timerId !== null) {
    clearInterval(timerId)
    timerId = null
  }
  if (rafId !== null) {
    cancelAnimationFrame(rafId)
    rafId = null
  }
  if (recorder && recorder.state !== 'inactive') {
    recorder.stop()
  }
  recorder = null
  if (audioCtx) {
    audioCtx.close().catch(() => {
      /* deja fermé */
    })
    audioCtx = null
  }
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
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="flex items-center justify-between px-5 pt-4 pb-2">
      <button
        class="font-sans text-pond-mid text-sm inline-flex items-center gap-1 hover:text-pond-deep"
        @click="cancel"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <polyline points="15 6 9 12 15 18" />
        </svg>
        Retour
      </button>
      <h1 class="font-display text-xl text-pond-deep">Faire coin-coin</h1>
      <span class="w-10" /> <!-- spacer -->
    </header>

    <!-- ─── Asking permission ─────────────────────────────────── -->
    <div
      v-if="state === 'asking'"
      class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-3"
    >
      <p class="font-display text-2xl text-pond-deep">Accès au micro…</p>
      <p class="font-mono text-xs text-ink-soft uppercase tracking-wider">
        autorise quand le navigateur demande
      </p>
    </div>

    <!-- ─── Error ────────────────────────────────────────────── -->
    <div
      v-else-if="state === 'error'"
      class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-4"
    >
      <p class="font-display text-2xl text-coral-deep">Hop, problème.</p>
      <p class="font-sans text-sm text-ink-soft max-w-xs">{{ errorMessage }}</p>
      <div class="flex gap-3 mt-2">
        <button
          class="font-sans text-sm font-semibold text-ink-soft underline"
          @click="cancel"
        >
          Retour
        </button>
        <button
          class="font-sans text-sm font-bold text-ink bg-duck px-4 py-2 rounded-xl"
          style="box-shadow: 0 4px 0 var(--duck-deep);"
          @click="requestPermission"
        >
          Réessayer
        </button>
      </div>
    </div>

    <!-- ─── Idle / Recording ─────────────────────────────────── -->
    <template v-else-if="state === 'idle' || state === 'recording'">
      <!-- Waveform live -->
      <div class="mx-5 mt-4 mb-2 h-32 rounded-2xl bg-gradient-to-b from-pond-pale to-[#d6ecf3] border border-[rgba(169,216,229,.6)] flex items-center justify-center px-4 relative overflow-hidden">
        <canvas
          ref="canvasEl"
          class="w-full h-20"
        />
        <div
          v-if="state === 'recording'"
          class="absolute top-3 right-3 inline-flex items-center gap-1.5 font-mono text-[11px] text-pond-deep bg-cream/85 px-2 py-1 rounded-full"
        >
          <span class="w-1.5 h-1.5 rounded-full bg-rec rec-pulse" />
          rec
        </div>
      </div>

      <!-- Timer -->
      <div class="text-center font-mono text-sm text-ink-soft tracking-wider">
        {{ formatTime(elapsed) }}
        <span class="opacity-50">/ 1:00</span>
        <span v-if="state === 'recording'" class="text-pond-deep ml-2">
          ({{ Math.ceil(remaining) }}s restantes)
        </span>
      </div>

      <!-- Caption optionnelle -->
      <div class="mx-5 mt-5">
        <label
          for="voice-caption"
          class="block font-sans text-sm font-medium text-ink mb-2"
        >
          Une légende&nbsp;? (optionnel)
        </label>
        <textarea
          id="voice-caption"
          v-model="caption"
          rows="2"
          maxlength="280"
          placeholder="Un mot pour les mariées, un contexte…"
          class="w-full px-3 py-2 rounded-xl border-[1.5px] border-cream-line bg-cream font-sans text-[14px] text-ink outline-none focus:border-pond-mid focus:shadow-[0_0_0_3px_rgba(62,138,168,0.18)] transition resize-none placeholder:italic placeholder:text-[#9AA5B0]"
        />
      </div>

      <!-- Record / Stop button -->
      <div class="mt-auto pb-10 flex flex-col items-center gap-3">
        <button
          v-if="state === 'idle'"
          class="capture__record"
          aria-label="Démarrer l’enregistrement"
          @click="startRecording"
        >
          <svg width="40" height="40" viewBox="0 0 24 24" fill="white" stroke="white" stroke-width="0">
            <rect x="9" y="2" width="6" height="12" rx="3" />
            <path d="M5 11a7 7 0 0 0 14 0" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
            <line x1="12" y1="18" x2="12" y2="22" stroke="white" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
        <button
          v-else
          class="capture__stop"
          aria-label="Arrêter l’enregistrement"
          @click="stopRecording"
        />
        <p class="font-mono text-[11px] uppercase tracking-wider text-ink-soft">
          {{
            state === 'idle'
              ? 'Appuie pour faire coin-coin'
              : 'Appuie pour arrêter'
          }}
        </p>
      </div>
    </template>

    <!-- ─── Uploading ────────────────────────────────────────── -->
    <div
      v-else-if="state === 'uploading'"
      class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-3"
    >
      <p class="font-display text-3xl text-pond-deep">
        Splash<span class="text-duck-deep">&nbsp;!</span>
      </p>
      <p class="font-mono text-xs text-ink-soft uppercase tracking-wider">
        envoi en cours…
      </p>
    </div>
  </main>
</template>

<style scoped>
.capture__record {
  width: 84px;
  height: 84px;
  border-radius: 50%;
  background: var(--coral-deep);
  border: 5px solid var(--coral-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: 0 6px 20px -6px rgba(184, 90, 63, 0.6);
  transition: transform 0.1s ease;
}
.capture__record:active { transform: scale(0.95); }

.capture__stop {
  width: 84px;
  height: 84px;
  border-radius: 22px;
  background: var(--coral-deep);
  border: none;
  cursor: pointer;
  box-shadow: 0 6px 20px -6px rgba(184, 90, 63, 0.6);
  transition: transform 0.1s ease;
}
.capture__stop:active { transform: scale(0.95); }

@keyframes rec-pulse {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.85); }
}
.rec-pulse { animation: rec-pulse 1.2s ease-in-out infinite; }
</style>
