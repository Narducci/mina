<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  modelValue: { type: String, default: '' },
})

const emit = defineEmits(['update:modelValue'])
const focado = ref(false)

function mascarar(valor) {
  return valor
    .replace(/\D/g, '')
    .slice(0, 11)
    .replace(/(\d{3})(\d)/, '$1.$2')
    .replace(/(\d{3})\.(\d{3})(\d)/, '$1.$2.$3')
    .replace(/(\d{3})\.(\d{3})\.(\d{3})(\d)/, '$1.$2.$3-$4')
}

function validar(cpf) {
  const n = cpf.replace(/\D/g, '')
  if (n.length !== 11 || /^(\d)\1{10}$/.test(n)) return false
  let soma = 0
  for (let i = 0; i < 9; i++) soma += parseInt(n[i]) * (10 - i)
  let r = (soma * 10) % 11
  if (r === 10 || r === 11) r = 0
  if (r !== parseInt(n[9])) return false
  soma = 0
  for (let i = 0; i < 10; i++) soma += parseInt(n[i]) * (11 - i)
  r = (soma * 10) % 11
  if (r === 10 || r === 11) r = 0
  return r === parseInt(n[10])
}

function onInput(e) {
  const masked = mascarar(e.target.value)
  e.target.value = masked
  emit('update:modelValue', masked)
}

const invalido = computed(() => {
  const n = props.modelValue.replace(/\D/g, '')
  return n.length === 11 && !validar(props.modelValue)
})
</script>

<template>
  <div class="input-cpf">
    <input
      type="text"
      inputmode="numeric"
      maxlength="14"
      :value="modelValue"
      @input="onInput"
      @focus="focado = true"
      @blur="focado = false"
      :class="{ erro: invalido }"
    />
    <label :class="{ active: focado || modelValue }">CPF</label>
    <span v-if="invalido" class="msg-erro">CPF inválido</span>
  </div>
</template>

<style scoped>
.input-cpf {
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
