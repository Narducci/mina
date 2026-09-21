<script setup>
import { computed } from 'vue'

const props = defineProps({
  label:      { type: String, required: true },
  modelValue: { type: String, default: '' },
})

const emit = defineEmits(['update:modelValue'])

function mascarar(valor) {
  return valor
    .replace(/\D/g, '')
    .slice(0, 8)
    .replace(/(\d{2})(\d)/, '$1/$2')
    .replace(/(\d{2})\/(\d{2})(\d)/, '$1/$2/$3')
}

function onInput(e) {
  const masked = mascarar(e.target.value)
  e.target.value = masked
  emit('update:modelValue', masked)
}

const invalido = computed(() => {
  const v = props.modelValue
  if (v.length < 10) return false
  const [dia, mes, ano] = v.split('/').map(Number)
  if (!dia || !mes || !ano) return true
  if (mes < 1 || mes > 12) return true
  const anoStr = props.modelValue.split('/')[2] || ''
  if (anoStr.length !== 4 || ano < 1900 || ano > 2100) return true
  const diasNoMes = new Date(ano, mes, 0).getDate()
  return dia < 1 || dia > diasNoMes
})
</script>

<template>
  <div class="app-input-data">
    <label class="input-label">{{ label }}</label>
    <input
      type="text"
      inputmode="numeric"
      maxlength="10"
      :value="modelValue"
      :class="{ erro: invalido }"
      @input="onInput"
      placeholder="DD/MM/AAAA"
    />
    <span v-if="invalido" class="msg-erro">Data inválida</span>
  </div>
</template>

<style scoped>
.app-input-data {
  display: flex;
  flex-direction: column;
  gap: 4px;
  width: 100%;
}

.input-label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

input {
  padding: 7px 10px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  transition: border-color 0.2s;
}

input:focus {
  border-color: #4a9eff;
}

input.erro {
  border-color: #cc4444;
}

input::placeholder {
  color: var(--cor-texto-fraco);
}

.msg-erro {
  font-size: 10px;
  color: #cc4444;
}
</style>
