<script setup>
defineProps({
  label: { type: String, required: true },
  modelValue: { type: String, default: '' },
  options: { type: Array, required: true }, // [{ value, label }]
  required: { type: Boolean, default: false },
})
defineEmits(['update:modelValue'])
</script>

<template>
  <div class="app-select">
    <label class="select-label">{{ label }}</label>
    <select
      :value="modelValue"
      :required="required"
      @change="$emit('update:modelValue', $event.target.value)"
    >
      <option value="" disabled hidden selected>Selecione...</option>
      <option
        v-for="opt in options"
        :key="opt.value"
        :value="opt.value"
      >{{ opt.label }}</option>
    </select>
  </div>
</template>

<style scoped>
.app-select {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.select-label {
  font-size: 10px;
  color: #4a9eff;
  padding-left: 2px;
}

select {
  width: 100%;
  padding: 9px 12px;
  background-color: var(--cor-input-bg);
  color: var(--cor-texto-forte);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  cursor: pointer;
  transition: border-color 0.2s;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888888' stroke-width='2'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-color: var(--cor-input-bg);
  padding-right: 32px;
}

select:focus {
  border-color: #4a9eff;
}
</style>
