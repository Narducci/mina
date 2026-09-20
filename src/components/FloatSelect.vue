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
      <label class="select-label" :class="{ active: aberto || modelValue }">{{ label }}</label>
      <span class="select-value">{{ labelSelecionado }}</span>
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
.float-select {
  position: relative;
  width: 100%;
}

.select-box {
  position: relative;
  width: 100%;
  padding: 20px 36px 6px 12px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: flex-end;
  min-height: 48px;
  box-sizing: border-box;
  transition: border-color 0.2s;
  outline: none;
}

.select-box:focus,
.select-box.open {
  border-color: #4a9eff;
}

.select-label {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 13px;
  color: var(--cor-texto-fraco);
  pointer-events: none;
  transition: all 0.15s ease;
  line-height: 1;
}

.select-label.active {
  top: 8px;
  transform: translateY(0);
  font-size: 10px;
  color: #4a9eff;
}

.select-value {
  font-size: 13px;
  color: var(--cor-texto-forte);
  line-height: 1;
}

.select-arrow {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 16px;
  height: 16px;
  color: var(--cor-texto-fraco);
  transition: transform 0.2s;
  flex-shrink: 0;
}

.select-arrow.rotated {
  transform: translateY(-50%) rotate(180deg);
}

.select-dropdown {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  background-color: var(--cor-input-bg);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
  list-style: none;
  margin: 0;
  padding: 4px 0;
  z-index: 9999;
  max-height: 200px;
  overflow-y: auto;
}

.select-dropdown li {
  padding: 8px 12px;
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
