<script setup lang="ts">
/**
 * FilterChips — sélecteur horizontal de type de média.
 *
 * Pattern repris de design/Mosaic 6 screens.html .filter-row : pills
 * monospace 10px, active en pond-deep blanc, inactive en transparent
 * avec border cream-line. Scrollable horizontalement si overflow.
 */
import type { MediaKind } from '../types/snapshot'

export type FilterValue = 'all' | MediaKind

defineProps<{
  modelValue: FilterValue
  /** Counts par type, peuplés depuis le snapshot par le parent. */
  counts: { all: number; photo: number; clip: number; voice: number }
}>()

defineEmits<{
  'update:modelValue': [value: FilterValue]
}>()
</script>

<template>
  <div class="filter-row">
    <button
      type="button"
      class="chip"
      :class="{ 'chip--active': modelValue === 'all' }"
      @click="$emit('update:modelValue', 'all')"
    >
      Tout · {{ counts.all }}
    </button>
    <button
      type="button"
      class="chip"
      :class="{ 'chip--active': modelValue === 'photo' }"
      @click="$emit('update:modelValue', 'photo')"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M14.5 4h-5l-1.5 2H4a2 2 0 0 0-2 2v10a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-4z" />
        <circle cx="12" cy="13" r="3" />
      </svg>
      {{ counts.photo }}
    </button>
    <button
      type="button"
      class="chip"
      :class="{ 'chip--active': modelValue === 'clip' }"
      @click="$emit('update:modelValue', 'clip')"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <rect x="2" y="6" width="14" height="12" rx="2" />
        <path d="M22 8 L16 12 L22 16 Z" />
      </svg>
      {{ counts.clip }}
    </button>
    <button
      type="button"
      class="chip"
      :class="{ 'chip--active': modelValue === 'voice' }"
      @click="$emit('update:modelValue', 'voice')"
    >
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <rect x="9" y="2" width="6" height="12" rx="3" />
        <path d="M5 11a7 7 0 0 0 14 0" />
      </svg>
      {{ counts.voice }}
    </button>
  </div>
</template>

<style scoped>
.filter-row {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  scrollbar-width: none;
  font-family: var(--mono);
  font-size: 10px;
  letter-spacing: 0.04em;
  padding-bottom: 2px;
}
.filter-row::-webkit-scrollbar {
  display: none;
}

.chip {
  padding: 5px 9px;
  border-radius: 999px;
  background: transparent;
  color: var(--ink-soft);
  border: 1px solid var(--cream-line);
  text-transform: uppercase;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.chip:hover:not(.chip--active) {
  border-color: var(--pond-mid);
  color: var(--pond-deep);
}

.chip--active {
  background: var(--pond-deep);
  color: #fff;
  border-color: var(--pond-deep);
}

.chip svg {
  width: 10px;
  height: 10px;
}
</style>
