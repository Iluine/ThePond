<script setup lang="ts">
/**
 * OrchestrationView — page témoins, protégée par WITNESS_SECRET.
 *
 * Le token vient de l'URL (?token=…) que l'admin partage via QR.
 * À l'arrivée on le sauve en sessionStorage. Toutes les mutations
 * incluent le token en query.
 *
 * Pour l'instant, le drag-drop des paliers est remplacé par des
 * boutons ↑↓ (mobile-friendly, zéro lib). L'inline edit ouvre des
 * inputs name + datetime-local pour name + target_time.
 *
 * Microcopies hardcodées V1 (pas encore de strings.ron côté front).
 *
 * Note V1 : les toggles Paramètres (défis, mode nuit) sont locaux,
 * pas encore persistés en DB — la mécanique défis arrive en P1.
 */
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useWitnessStore } from '../stores/witness'
import { useSnapshotStore } from '../stores/snapshot'
import type { Phase } from '../types/snapshot'

const route = useRoute()
const witnessStore = useWitnessStore()
const snapshotStore = useSnapshotStore()

// ─── Auth bootstrap ────────────────────────────────────────────
const authChecked = ref(false)
const authError = ref<string | null>(null)
const manualToken = ref('')

onMounted(async () => {
  const fromQuery = (route.query.token as string) ?? ''
  if (fromQuery) {
    witnessStore.setToken(fromQuery)
  }
  await checkAuth()
})

async function checkAuth() {
  authError.value = null
  if (!witnessStore.isAuthenticated) {
    authChecked.value = true
    return
  }
  try {
    const res = await fetch(witnessStore.withToken('/api/orchestration/state'))
    if (res.status === 401) {
      authError.value = 'Token invalide. Demande à l’admin une nouvelle URL.'
      witnessStore.clear()
    } else if (!res.ok) {
      authError.value = `Erreur serveur (HTTP ${res.status})`
    }
  } catch (err) {
    authError.value = err instanceof Error ? err.message : String(err)
  } finally {
    authChecked.value = true
  }
}

function submitManualToken() {
  const t = manualToken.value.trim()
  if (!t) return
  witnessStore.setToken(t)
  void checkAuth()
}

// ─── Données live depuis le snapshot SSE ──────────────────────
const phases = computed<Phase[]>(() => snapshotStore.phasesAll)
const counts = computed(() => snapshotStore.counts)

// État de chaque palier (done / live / locked) — calcule depuis serverTime
type PhaseStatus = 'done' | 'live' | 'locked'

function phaseStatus(p: Phase): PhaseStatus {
  const cur = snapshotStore.phaseCurrent
  if (!cur) {
    // Pas de phase courante : tout est verrouillé sauf les passées
    return p.target_time <= (snapshotStore.serverTime ?? '')
      ? 'done'
      : 'locked'
  }
  if (p.id === cur.id) return 'live'
  if (p.phase_order < cur.phase_order) return 'done'
  return 'locked'
}

function formatTimeShort(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return iso
  const h = d.getHours()
  const m = d.getMinutes()
  return m === 0 ? `${h}h` : `${h.toString().padStart(2, '0')}h${m.toString().padStart(2, '0')}`
}

/** Convertit ISO → YYYY-MM-DDTHH:MM pour <input type=datetime-local>. */
function isoToDatetimeLocal(iso: string): string {
  const d = new Date(iso)
  if (isNaN(d.getTime())) return ''
  const pad = (n: number) => n.toString().padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/** Convertit YYYY-MM-DDTHH:MM (datetime-local local time) → ISO UTC. */
function datetimeLocalToIso(dl: string): string {
  if (!dl) return ''
  const d = new Date(dl)
  return d.toISOString()
}

// ─── Inline edit ───────────────────────────────────────────────
const editingId = ref<number | null>(null)
const editName = ref('')
const editTime = ref('')

function startEdit(p: Phase) {
  editingId.value = p.id
  editName.value = p.name
  editTime.value = isoToDatetimeLocal(p.target_time)
}

function cancelEdit() {
  editingId.value = null
}

const mutationPending = ref(false)
const mutationError = ref<string | null>(null)

async function saveEdit(p: Phase) {
  if (mutationPending.value) return
  mutationPending.value = true
  mutationError.value = null
  try {
    const newName = editName.value.trim()
    if (!newName) throw new Error('Le nom ne peut pas être vide')
    const targetTime = editTime.value
      ? datetimeLocalToIso(editTime.value)
      : undefined

    const res = await fetch(
      witnessStore.withToken(`/api/orchestration/phases/${p.id}`),
      {
        method: 'PATCH',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ name: newName, target_time: targetTime }),
      },
    )
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || `HTTP ${res.status}`)
    }
    editingId.value = null
  } catch (err) {
    mutationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    mutationPending.value = false
  }
}

async function triggerPhase(p: Phase) {
  if (mutationPending.value) return
  if (!confirm(`Forcer le déclenchement du palier "${p.name}" maintenant ?`)) return
  mutationPending.value = true
  mutationError.value = null
  try {
    const res = await fetch(
      witnessStore.withToken(`/api/orchestration/phases/${p.id}/trigger`),
      { method: 'POST' },
    )
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || `HTTP ${res.status}`)
    }
  } catch (err) {
    mutationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    mutationPending.value = false
  }
}

async function deletePhase(p: Phase) {
  if (mutationPending.value) return
  if (!confirm(`Supprimer le palier "${p.name}" ? Cette action est irréversible.`)) return
  mutationPending.value = true
  mutationError.value = null
  try {
    const res = await fetch(
      witnessStore.withToken(`/api/orchestration/phases/${p.id}`),
      { method: 'DELETE' },
    )
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || `HTTP ${res.status}`)
    }
  } catch (err) {
    mutationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    mutationPending.value = false
  }
}

async function moveUp(idx: number) {
  if (idx === 0) return
  const reorder = phases.value.map((p, i) => ({
    id: p.id,
    phase_order: i === idx ? i - 1 : i === idx - 1 ? i + 1 : i,
  }))
  await postReorder(reorder)
}

async function moveDown(idx: number) {
  if (idx === phases.value.length - 1) return
  const reorder = phases.value.map((p, i) => ({
    id: p.id,
    phase_order: i === idx ? i + 1 : i === idx + 1 ? i - 1 : i,
  }))
  await postReorder(reorder)
}

async function postReorder(items: { id: number; phase_order: number }[]) {
  if (mutationPending.value) return
  mutationPending.value = true
  mutationError.value = null
  try {
    const res = await fetch(
      witnessStore.withToken('/api/orchestration/phases/reorder'),
      {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify(items),
      },
    )
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || `HTTP ${res.status}`)
    }
  } catch (err) {
    mutationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    mutationPending.value = false
  }
}

// ─── Add palier ────────────────────────────────────────────────
const showAddForm = ref(false)
const addName = ref('')
const addTime = ref('')

function openAdd() {
  showAddForm.value = true
  addName.value = ''
  // Default à maintenant + 1h
  const now = new Date(Date.now() + 60 * 60 * 1000)
  addTime.value = isoToDatetimeLocal(now.toISOString())
}

async function submitAdd() {
  if (mutationPending.value) return
  mutationPending.value = true
  mutationError.value = null
  try {
    const name = addName.value.trim()
    if (!name) throw new Error('Le nom est obligatoire')
    if (!addTime.value) throw new Error('L’heure est obligatoire')
    const target_time = datetimeLocalToIso(addTime.value)

    const res = await fetch(witnessStore.withToken('/api/orchestration/phases'), {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ name, target_time }),
    })
    if (!res.ok) {
      const body = await res.json().catch(() => ({}))
      throw new Error(body.error || `HTTP ${res.status}`)
    }
    showAddForm.value = false
  } catch (err) {
    mutationError.value = err instanceof Error ? err.message : String(err)
  } finally {
    mutationPending.value = false
  }
}

// ─── Pass to next palier (= trigger le premier locked) ────────
const nextLocked = computed(() =>
  phases.value.find((p) => phaseStatus(p) === 'locked'),
)

async function passToNext() {
  if (!nextLocked.value) return
  await triggerPhase(nextLocked.value)
}

// ─── Settings (V1 décoratifs) ──────────────────────────────────
const settingsChallenges = ref(false)
const settingsNightMode = ref(true)

// ─── Export ────────────────────────────────────────────────────
function exportZip() {
  // Le navigateur télécharge directement via l'URL avec token en query
  const url = witnessStore.withToken('/api/orchestration/export')
  window.location.href = url
}

const witnessName = ref('Marc') // V1 : pas de notion de "qui est ce témoin", placeholder
</script>

<template>
  <main class="min-h-screen flex flex-col max-w-[420px] mx-auto bg-cream">
    <!-- ─── Header ──────────────────────────────────────────── -->
    <header class="flex items-center justify-between px-5 pt-4 pb-1.5">
      <div class="font-sans font-bold text-lg text-pond-deep">
        The <span class="text-duck-deep">Pond</span>.
      </div>
      <span class="pill-shield">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 2L4 5v7c0 5 3.5 9 8 10 4.5-1 8-5 8-10V5z" />
        </svg>
        Orchestration
      </span>
    </header>

    <!-- ═══ Auth gate ═════════════════════════════════════════ -->
    <template v-if="!authChecked">
      <div class="flex-1 flex items-center justify-center">
        <p class="font-mono text-xs text-ink-soft uppercase tracking-wider">…</p>
      </div>
    </template>

    <template v-else-if="!witnessStore.isAuthenticated">
      <div class="flex-1 flex flex-col items-center justify-center px-8 text-center gap-4">
        <p class="font-display text-2xl text-pond-deep">Token témoin requis</p>
        <p class="font-sans text-sm text-ink-soft max-w-xs">
          Cette page est réservée aux témoins. Demande à l’admin l’URL avec
          le token, ou colle-le ci-dessous.
        </p>
        <div class="flex flex-col gap-2 w-full max-w-xs">
          <input
            v-model="manualToken"
            type="text"
            placeholder="token…"
            class="manual-input"
          />
          <button class="primary-btn" type="button" @click="submitManualToken">
            Valider
          </button>
        </div>
        <p v-if="authError" class="font-mono text-[11px] text-coral-deep">
          {{ authError }}
        </p>
      </div>
    </template>

    <template v-else>
      <!-- ─── Welcome + stats ──────────────────────────────── -->
      <div class="px-5 pt-2 pb-1">
        <h1 class="font-display text-[22px] text-pond-deep">Bonsoir {{ witnessName }}</h1>
        <p class="font-mono text-[10px] text-ink-soft tracking-wider uppercase mt-0.5">
          Témoin · accès orchestration
        </p>
      </div>

      <div v-if="counts" class="o-state mx-4 mt-3">
        <div class="o-seg">
          <span class="o-num">{{ counts.total_posts }}</span>
          <span class="o-lab">partagés</span>
        </div>
        <div class="o-div" />
        <div class="o-seg">
          <span class="o-num">{{ counts.posts_visible }}</span>
          <span class="o-lab">révélés</span>
        </div>
        <div class="o-div" />
        <div class="o-seg">
          <span class="o-num">{{ counts.posts_pending }}</span>
          <span class="o-lab">en attente</span>
        </div>
      </div>

      <!-- ─── Paliers ──────────────────────────────────────── -->
      <div class="o-section-title">
        <h3>Paliers</h3>
        <span class="hint">↑↓ pour réordonner</span>
      </div>

      <div class="palier-list mx-4">
        <div
          v-for="(p, idx) in phases"
          :key="p.id"
          class="palier"
          :class="{ 'palier--live': phaseStatus(p) === 'live', 'palier--faded': phaseStatus(p) === 'done' }"
        >
          <!-- Reorder buttons -->
          <div class="reorder">
            <button
              type="button"
              class="reorder-btn"
              :disabled="idx === 0 || mutationPending"
              aria-label="Monter"
              @click="moveUp(idx)"
            >↑</button>
            <button
              type="button"
              class="reorder-btn"
              :disabled="idx === phases.length - 1 || mutationPending"
              aria-label="Descendre"
              @click="moveDown(idx)"
            >↓</button>
          </div>

          <!-- Status icon -->
          <span
            class="stat-icon"
            :class="`stat-icon--${phaseStatus(p)}`"
            :aria-label="phaseStatus(p)"
          >
            <svg v-if="phaseStatus(p) === 'done'" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="20 6 9 17 4 12" />
            </svg>
            <svg v-else-if="phaseStatus(p) === 'locked'" width="9" height="9" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4">
              <rect x="5" y="11" width="14" height="10" rx="2" />
              <path d="M8 11V7a4 4 0 0 1 8 0v4" />
            </svg>
          </span>

          <!-- Inline edit ou affichage normal -->
          <template v-if="editingId === p.id">
            <div class="edit-fields">
              <input
                v-model="editName"
                type="text"
                maxlength="80"
                class="edit-name"
                placeholder="Nom du palier"
              />
              <input
                v-model="editTime"
                type="datetime-local"
                class="edit-time"
              />
            </div>
            <div class="edit-actions">
              <button type="button" class="edit-ok" :disabled="mutationPending" @click="saveEdit(p)">✓</button>
              <button type="button" class="edit-cancel" @click="cancelEdit">×</button>
            </div>
          </template>

          <template v-else>
            <span class="name">
              {{ p.name }}
              <button type="button" class="edit-pencil" aria-label="Éditer" @click="startEdit(p)">
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M16 3l5 5-12 12H4v-5z" />
                </svg>
              </button>
            </span>
            <span class="when">{{ formatTimeShort(p.target_time) }}</span>
            <div class="row-actions">
              <button
                v-if="phaseStatus(p) === 'locked'"
                type="button"
                class="row-act"
                :disabled="mutationPending"
                title="Forcer le déclenchement"
                @click="triggerPhase(p)"
              >
                ⚡
              </button>
              <button
                type="button"
                class="row-act row-act--danger"
                :disabled="mutationPending"
                title="Supprimer"
                @click="deletePhase(p)"
              >
                ×
              </button>
            </div>
          </template>
        </div>

        <p
          v-if="phases.length === 0"
          class="font-mono text-xs text-ink-soft text-center py-6"
        >
          Aucun palier configuré. Ajoute le premier ci-dessous.
        </p>
      </div>

      <!-- Add palier -->
      <div v-if="!showAddForm" class="add-palier mx-4 mt-2" @click="openAdd">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round">
          <line x1="12" y1="5" x2="12" y2="19" />
          <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
        Ajouter un palier
      </div>

      <div v-else class="add-form mx-4 mt-2">
        <input
          v-model="addName"
          type="text"
          placeholder="Nom (ex. Premières danses)"
          maxlength="80"
          class="manual-input mb-2"
        />
        <input
          v-model="addTime"
          type="datetime-local"
          class="manual-input mb-2"
        />
        <div class="flex gap-2">
          <button type="button" class="font-sans text-sm font-bold py-2 px-3 rounded-xl bg-duck text-ink flex-1" :disabled="mutationPending" @click="submitAdd">
            Créer
          </button>
          <button type="button" class="font-sans text-sm py-2 px-3 rounded-xl border border-cream-line bg-cream" @click="showAddForm = false">
            Annuler
          </button>
        </div>
      </div>

      <!-- Pass to next CTA -->
      <button
        v-if="nextLocked"
        type="button"
        class="o-cta mx-4 mt-3"
        :disabled="mutationPending"
        @click="passToNext"
      >
        Passer au prochain palier ({{ nextLocked.name }})
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round">
          <polyline points="9 18 15 12 9 6" />
        </svg>
      </button>

      <p v-if="mutationError" class="font-mono text-[11px] text-coral-deep px-4 mt-2 text-center">
        {{ mutationError }}
      </p>

      <!-- ─── Settings (V1 décoratifs) ─────────────────────── -->
      <div class="o-section-title">
        <h3>Paramètres</h3>
        <span class="hint" style="text-transform: none; letter-spacing: 0; color: var(--ink-soft); font-size: 9px;">
          (V1 décoratifs)
        </span>
      </div>

      <div class="o-toggle mx-4">
        <div class="o-toggle-row">
          <div class="lbl">Défis activés</div>
          <button
            type="button"
            class="switch"
            :class="{ 'switch--on': settingsChallenges }"
            :aria-pressed="settingsChallenges"
            @click="settingsChallenges = !settingsChallenges"
          />
        </div>
        <div class="o-toggle-desc">Propose des micro-missions aux invités (P1)</div>
      </div>

      <div class="o-toggle mx-4">
        <div class="o-toggle-row">
          <div class="lbl">Mode nuit après 23h</div>
          <button
            type="button"
            class="switch"
            :class="{ 'switch--on': settingsNightMode }"
            :aria-pressed="settingsNightMode"
            @click="settingsNightMode = !settingsNightMode"
          />
        </div>
        <div class="o-toggle-desc">Couleurs adoucies pour la fin de soirée</div>
      </div>

      <!-- ─── Actions ──────────────────────────────────────── -->
      <div class="o-actions mx-4">
        <h3>Actions</h3>
        <button type="button" class="o-export" @click="exportZip">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
          </svg>
          Exporter toute la mare
          <span class="sub">.zip</span>
        </button>
      </div>

      <div class="h-6" />
    </template>
  </main>
</template>

<style scoped>
.pill-shield {
  font-family: var(--mono);
  font-size: 11px;
  font-weight: 500;
  padding: 5px 10px;
  border-radius: 999px;
  background: rgba(62, 138, 168, 0.12);
  border: 1px solid rgba(62, 138, 168, 0.45);
  color: var(--pond-deep);
  display: inline-flex;
  align-items: center;
  gap: 4px;
  letter-spacing: 0.02em;
}
.pill-shield svg { color: var(--pond-mid); }

.manual-input {
  width: 100%;
  height: 44px;
  padding: 0 14px;
  border-radius: 12px;
  border: 1.5px solid var(--cream-line);
  background: var(--cream);
  font-family: var(--sans);
  font-size: 14px;
  color: var(--ink);
  outline: none;
}
.manual-input:focus {
  border-color: var(--pond-mid);
  box-shadow: 0 0 0 3px rgba(62, 138, 168, 0.18);
}

.primary-btn {
  width: 100%;
  height: 48px;
  background: var(--duck);
  color: var(--ink);
  font-family: var(--sans);
  font-weight: 700;
  font-size: 14px;
  border: none;
  border-radius: 14px;
  box-shadow: 0 4px 0 var(--duck-deep);
  cursor: pointer;
}

/* ─── Stats ──────────────────────────────────────────────── */
.o-state {
  padding: 12px;
  background: var(--pond-pale);
  border: 1px solid rgba(62, 138, 168, 0.25);
  border-radius: 12px;
  display: flex;
  align-items: center;
  gap: 10px;
}
.o-seg {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}
.o-num {
  font-family: var(--display);
  font-size: 18px;
  color: var(--pond-deep);
}
.o-lab {
  font-family: var(--mono);
  font-size: 8px;
  letter-spacing: 0.06em;
  color: var(--ink-soft);
  text-transform: uppercase;
}
.o-div {
  width: 1px;
  height: 22px;
  background: rgba(62, 138, 168, 0.25);
}

/* ─── Section titles ─────────────────────────────────────── */
.o-section-title {
  margin: 14px 16px 6px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.o-section-title h3 {
  margin: 0;
  font-family: var(--display);
  font-size: 16px;
  color: var(--pond-deep);
}
.o-section-title .hint {
  font-family: var(--mono);
  font-size: 9px;
  color: var(--ink-soft);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

/* ─── Palier list ────────────────────────────────────────── */
.palier-list {
  background: var(--cream);
  border: 1px solid var(--cream-line);
  border-radius: 14px;
  overflow: hidden;
}
.palier {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 10px;
  border-bottom: 1px solid var(--cream-line);
}
.palier:last-child { border-bottom: none; }

.reorder {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex-shrink: 0;
}
.reorder-btn {
  width: 18px;
  height: 14px;
  border: none;
  background: transparent;
  color: var(--ink-soft);
  font-family: var(--mono);
  font-size: 11px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
}
.reorder-btn:disabled { opacity: 0.25; cursor: not-allowed; }
.reorder-btn:hover:not(:disabled) { color: var(--pond-deep); }

.stat-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.stat-icon--done {
  background: var(--green);
  color: white;
}
.stat-icon--live {
  background: var(--duck);
  color: var(--ink);
  box-shadow: 0 0 0 3px rgba(255, 201, 60, 0.35);
  animation: pulse-live 1.6s ease-in-out infinite;
}
@keyframes pulse-live {
  0%, 100% { box-shadow: 0 0 0 3px rgba(255, 201, 60, 0.35); }
  50%      { box-shadow: 0 0 0 6px rgba(255, 201, 60, 0);    }
}
.stat-icon--locked {
  background: var(--cream-deep);
  color: var(--ink-soft);
  border: 1px solid var(--cream-line);
}

.name {
  flex: 1;
  min-width: 0;
  font-family: var(--sans);
  font-size: 12px;
  font-weight: 600;
  color: var(--ink);
  display: flex;
  align-items: center;
  gap: 5px;
}
.palier--faded .name { color: var(--ink-soft); }

.edit-pencil {
  width: 14px;
  height: 14px;
  background: transparent;
  border: none;
  color: var(--ink-soft);
  opacity: 0.45;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}
.edit-pencil:hover { opacity: 1; color: var(--pond-deep); }

.when {
  font-family: var(--mono);
  font-size: 9px;
  color: var(--ink-soft);
  flex-shrink: 0;
  letter-spacing: 0.04em;
}
.palier--live .when { color: var(--duck-deep); font-weight: 500; }

.row-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}
.row-act {
  width: 22px;
  height: 22px;
  border: 1px solid var(--cream-line);
  background: var(--cream);
  border-radius: 6px;
  font-size: 11px;
  cursor: pointer;
  color: var(--ink-soft);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}
.row-act:hover:not(:disabled) {
  background: var(--cream-deep);
  color: var(--pond-deep);
}
.row-act--danger:hover:not(:disabled) {
  border-color: var(--coral-line);
  background: var(--coral-soft);
  color: var(--coral-deep);
}
.row-act:disabled { opacity: 0.5; cursor: not-allowed; }

.edit-fields {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}
.edit-name, .edit-time {
  width: 100%;
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--cream-line);
  background: var(--cream);
  font-family: var(--sans);
  font-size: 12px;
  padding: 0 6px;
  outline: none;
}
.edit-name:focus, .edit-time:focus {
  border-color: var(--pond-mid);
}
.edit-actions {
  display: flex;
  gap: 3px;
  flex-shrink: 0;
}
.edit-ok, .edit-cancel {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 6px;
  font-family: var(--mono);
  font-size: 14px;
  cursor: pointer;
}
.edit-ok {
  background: var(--green);
  color: white;
}
.edit-cancel {
  background: var(--cream-deep);
  color: var(--ink-soft);
}

/* ─── Add palier ─────────────────────────────────────────── */
.add-palier {
  padding: 8px 10px;
  border: 1px dashed var(--cream-line);
  border-radius: 10px;
  text-align: center;
  font-family: var(--sans);
  font-size: 11px;
  color: var(--pond-mid);
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  cursor: pointer;
}
.add-palier:hover {
  background: var(--cream-deep);
  color: var(--pond-deep);
}

.add-form {
  padding: 12px;
  background: var(--cream-deep);
  border: 1px solid var(--cream-line);
  border-radius: 12px;
}

/* ─── O-CTA passer au prochain ────────────────────────────── */
.o-cta {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: calc(100% - 32px);
  height: 48px;
  background: var(--duck);
  color: var(--ink);
  font-family: var(--sans);
  font-weight: 700;
  font-size: 12px;
  letter-spacing: 0.04em;
  border: none;
  border-radius: 14px;
  cursor: pointer;
  box-shadow:
    0 3px 0 var(--duck-deep),
    0 8px 18px -8px rgba(242, 180, 0, 0.5);
}
.o-cta:disabled { opacity: 0.5; cursor: not-allowed; }

/* ─── Toggles ───────────────────────────────────────────── */
.o-toggle {
  margin-bottom: 6px;
  padding: 9px 11px;
  background: var(--cream);
  border: 1px solid var(--cream-line);
  border-radius: 12px;
}
.o-toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}
.lbl {
  font-family: var(--sans);
  font-weight: 600;
  font-size: 12px;
  color: var(--ink);
}
.o-toggle-desc {
  margin-top: 2px;
  font-family: var(--mono);
  font-size: 9px;
  color: var(--ink-soft);
  line-height: 1.35;
}
.switch {
  width: 32px;
  height: 18px;
  flex-shrink: 0;
  border-radius: 999px;
  background: #cfd2d6;
  position: relative;
  border: none;
  cursor: pointer;
  transition: background 0.15s;
  padding: 0;
}
.switch::after {
  content: '';
  position: absolute;
  top: 2px;
  left: 2px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.25);
  transition: left 0.15s;
}
.switch--on { background: var(--pond-deep); }
.switch--on::after { left: 16px; }

/* ─── Actions ───────────────────────────────────────────── */
.o-actions {
  margin-top: 14px;
  padding-top: 10px;
  border-top: 1px solid var(--cream-line);
}
.o-actions h3 {
  margin: 0 0 8px;
  font-family: var(--sans);
  font-weight: 600;
  font-size: 12px;
  color: var(--pond-deep);
  letter-spacing: 0.02em;
}
.o-export {
  width: 100%;
  padding: 10px 12px;
  background: var(--cream);
  border: 1px solid var(--cream-line);
  color: var(--ink);
  font-family: var(--sans);
  font-size: 12px;
  font-weight: 500;
  border-radius: 11px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}
.o-export:hover { background: var(--cream-deep); }
.o-export svg { color: var(--pond-mid); flex-shrink: 0; }
.o-export .sub {
  margin-left: auto;
  font-family: var(--mono);
  font-size: 9px;
  color: var(--ink-soft);
  letter-spacing: 0.04em;
}
</style>
