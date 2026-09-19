<script setup>
defineProps({
  label: { type: String, required: true },
  modelValue: { type: String, default: '' },
  type: { type: String, default: 'text' },
  required: { type: Boolean, default: false },
})
defineEmits(['update:modelValue'])
</script>

<template>
  <div class="float-label">
    <input
      :type="type"
      :value="modelValue"
      :required="required"
      placeholder=" "
      @input="$emit('update:modelValue', $event.target.value)"
    />
    <label>{{ label }}</label>
  </div>
</template>

<style scoped>
.float-label {
  position: relative;
  width: 100%;
}

.float-label input {
  width: 100%;
  padding: 22px 12px 8px 12px;
  background-color: var(--cor-input-bg);
  color: var(--cor-texto-forte);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
}

.float-label input:focus {
  border-color: #4a9eff;
}

.float-label label {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  font-size: 13px;
  color: var(--cor-texto-fraco);
  pointer-events: none;
  transition: all 0.2s ease;
}

.float-label input:focus + label,
.float-label input:not(:placeholder-shown) + label {
  top: 8px;
  transform: translateY(0);
  font-size: 10px;
  color: #4a9eff;
}
</style>
