<script setup>
defineProps({
  label: { type: String, required: true },
  modelValue: { type: Boolean, default: false },
})
defineEmits(['update:modelValue'])
</script>

<template>
  <div class="app-toggle">
    <span class="toggle-label">{{ label }}</span>
    <button
      type="button"
      class="toggle-track"
      :class="{ on: modelValue }"
      @click="$emit('update:modelValue', !modelValue)"
    >
      <span class="toggle-thumb" />
    </button>
  </div>
</template>

<style scoped>
.app-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
}

.toggle-label {
  font-size: 13px;
  color: var(--cor-texto);
}

.toggle-track {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background-color: var(--cor-borda);
  border: none;
  cursor: pointer;
  position: relative;
  transition: background-color 0.2s;
  flex-shrink: 0;
}

.toggle-track.on {
  background-color: var(--cor-selecao);
}

.toggle-thumb {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background-color: #ffffff;
  transition: left 0.2s;
}

.toggle-track.on .toggle-thumb {
  left: 19px;
}
</style>
