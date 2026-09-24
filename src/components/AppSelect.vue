<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ChevronDown } from '@lucide/vue'

const props = defineProps({
  label:        { type: String, required: true },
  modelValue:   { type: String, default: '' },
  options:      { type: Array,  required: true }, // [{ value, label }]
  placeholder:  { type: String, default: '' },
  disabled:     { type: Boolean, default: false },
})

const emit = defineEmits(['update:modelValue'])

const aberto     = ref(false)
const containerRef = ref(null)

const labelSelecionado = computed(() => {
  const op = props.options.find(o => o.value === props.modelValue)
  return op ? op.label : ''
})

function selecionar(value) {
  emit('update:modelValue', value)
  aberto.value = false
}

function toggle() {
  if (props.disabled) return
  aberto.value = !aberto.value
}

function handleClickFora(e) {
  if (containerRef.value && !containerRef.value.contains(e.target)) {
    aberto.value = false
  }
}

function handleKeydown(e) {
  if (e.key === 'Escape') aberto.value = false
}

onMounted(() => {
  document.addEventListener('click', handleClickFora)
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickFora)
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div class="app-select" ref="containerRef">
    <label class="select-label">{{ label }}</label>
    <div
      class="select-box"
      :class="{ open: aberto, desabilitado: disabled }"
      @click="toggle"
      :tabindex="disabled ? -1 : 0"
      @keydown.enter="toggle"
      @keydown.space.prevent="toggle"
    >
      <span class="select-value" :class="{ placeholder: !modelValue }">
        {{ labelSelecionado || placeholder }}
      </span>
      <ChevronDown class="select-arrow" :class="{ rotated: aberto }" />
    </div>

    <ul v-if="aberto" class="select-dropdown">
      <li
        v-for="opt in options"
        :key="opt.value"
        :class="{ selected: opt.value === modelValue }"
        @click="selecionar(opt.value)"
      >
        {{ opt.label }}
      </li>
    </ul>
  </div>
</template>

<style scoped>
.app-select {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
  position: relative;
}

.select-label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

.select-box {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 7px 10px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  cursor: pointer;
  outline: none;
  transition: border-color 0.2s;
  min-height: 32px;
}

.select-box:focus,
.select-box.open {
  border-color: #4a9eff;
}

.select-box.desabilitado {
  opacity: 0.45;
  cursor: default;
}

.select-value {
  font-size: 13px;
  color: var(--cor-texto-forte);
}

.select-value.placeholder {
  color: var(--cor-texto-fraco);
}

.select-arrow {
  width: 14px;
  height: 14px;
  color: var(--cor-texto-fraco);
  flex-shrink: 0;
  transition: transform 0.2s;
}

.select-arrow.rotated {
  transform: rotate(180deg);
}

.select-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  list-style: none;
  padding: 4px 0;
  z-index: 9999;
  max-height: 200px;
  overflow-y: auto;
}

.select-dropdown li {
  padding: 7px 10px;
  font-size: 13px;
  color: var(--cor-texto);
  cursor: pointer;
  transition: background-color 0.1s;
}

.select-dropdown li:hover {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}

.select-dropdown li.selected {
  color: #4a9eff;
}
</style>
