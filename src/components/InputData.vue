<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  label: { type: String, required: true },
  modelValue: { type: String, default: '' },
})

const emit = defineEmits(['update:modelValue'])
const focado = ref(false)

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
  if (v.length < 10) return false // não validar enquanto incompleto
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
  <div class="input-data">
    <input
      type="text"
      inputmode="numeric"
      maxlength="10"
      :value="modelValue"
      @input="onInput"
      @focus="focado = true"
      @blur="focado = false"
      :class="{ erro: invalido }"
    />
    <label :class="{ active: focado || modelValue }">{{ label }}</label>
    <span v-if="invalido" class="msg-erro">Data inválida</span>
  </div>
</template>

<style scoped>
.input-data {
  position: relative;
  width: 100%;
}

input {
  width: 100%;
  padding: 20px 12px 6px 12px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
  outline: none;
  min-height: 48px;
  box-sizing: border-box;
  transition: border-color 0.2s;
}

input:focus {
  border-color: #4a9eff;
}

input.erro {
  border-color: #cc4444;
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

.msg-erro {
  font-size: 10px;
  color: #cc4444;
  margin-top: 2px;
  display: block;
}
</style>
