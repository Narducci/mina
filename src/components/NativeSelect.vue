<script setup>
defineProps({
  label: { type: String, required: true },
  modelValue: { type: String, default: '' },
  options: { type: Array, required: true },
})

defineEmits(['update:modelValue'])
</script>

<template>
  <div class="native-select">
    <select
      :value="modelValue"
      @change="$emit('update:modelValue', $event.target.value)"
      :class="{ filled: modelValue }"
    >
      <option value="" disabled hidden></option>
      <option v-for="opt in options" :key="opt.value" :value="opt.value">
        {{ opt.label }}
      </option>
    </select>
    <label :class="{ active: modelValue }">{{ label }}</label>
  </div>
</template>

<style scoped>
.native-select {
  position: relative;
  width: 100%;
}

select {
  width: 100%;
  padding: 20px 12px 6px 12px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
  appearance: none;
  cursor: pointer;
  outline: none;
  min-height: 48px;
  box-sizing: border-box;
  transition: border-color 0.2s;
}

select:focus {
  border-color: #4a9eff;
}

select option {
  background-color: #2a2a2a;
  color: var(--cor-texto-forte);
}

label {
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

label.active {
  top: 8px;
  transform: translateY(0);
  font-size: 10px;
  color: #4a9eff;
}

/* Seta customizada */
.native-select::after {
  content: '';
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 5px solid var(--cor-texto-fraco);
  pointer-events: none;
}
</style>
