<script setup>
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { ChevronDown } from '@lucide/vue'

const props = defineProps({
  label: { type: String, required: true },
  modelValue: { type: String, default: '' },
  options: { type: Array, required: true },
})

const emit = defineEmits(['update:modelValue'])
const aberto = ref(false)
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
  aberto.value = !aberto.value
}

function handleClickFora(e) {
  if (containerRef.value && !containerRef.value.contains(e.target)) {
    aberto.value = false
  }
}

onMounted(() => document.addEventListener('click', handleClickFora))
onUnmounted(() => document.removeEventListener('click', handleClickFora))
</script>

<template>
  <div class="float-select" ref="containerRef">
    <div
      class="select-box"
      :class="{ open: aberto, filled: modelValue }"
      @click="toggle"
      tabindex="0"
      @keydown.enter="toggle"
      @keydown.space.prevent="toggle"
      @keydown.escape="aberto = false"
    >
      <span class="select-value">{{ labelSelecionado }}</span>
      <ChevronDown class="select-arrow" :class="{ rotated: aberto }" />
    </div>
    <label class="select-label" :class="{ active: aberto || modelValue }">{{ label }}</label>

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
.float-select {
  position: relative;
  width: 100%;
}

.select-box {
  width: 100%;
  padding: 22px 12px 8px 12px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 48px;
  transition: border-color 0.2s;
  outline: none;
}

.select-box:focus {
  border-color: #4a9eff;
}

.select-box.open {
  border-color: #4a9eff;
  border-radius: 4px 4px 0 0;
}

.select-value {
  font-size: 13px;
  color: var(--cor-texto-forte);
}

.select-arrow {
  width: 16px;
  height: 16px;
  color: var(--cor-texto-fraco);
  transition: transform 0.2s;
  flex-shrink: 0;
}

.select-arrow.rotated {
  transform: rotate(180deg);
}

.select-label {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 13px;
  color: var(--cor-texto-fraco);
  pointer-events: none;
  transition: all 0.2s ease;
}

.select-label.active {
  top: 8px;
  transform: translateY(0);
  font-size: 10px;
  color: #4a9eff;
}

.select-dropdown {
  position: absolute;
  top: 100%;
  left: 0;
  right: 0;
  background-color: var(--cor-input-bg);
  border: 1px solid #4a9eff;
  border-top: none;
  border-radius: 0 0 4px 4px;
  list-style: none;
  z-index: 9999;
  max-height: 200px;
  overflow-y: auto;
}

.select-dropdown li {
  padding: 9px 12px;
  font-size: 13px;
  color: var(--cor-texto);
  cursor: pointer;
}

.select-dropdown li:hover {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}

.select-dropdown li.selected {
  color: #4a9eff;
}
</style>
