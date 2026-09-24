<script setup>
import { ref } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'

const props = defineProps({
  lancamentoId: { type: Number, required: true },
  valorAtual:   { type: Number, default: 0 },
})

const emit = defineEmits(['fechar', 'atualizado'])

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

const valor    = ref(
  Math.abs(props.valorAtual).toLocaleString('pt-BR', { minimumFractionDigits: 2 })
)
const erro     = ref('')
const salvando = ref(false)

function mascaraValor(e) {
  let v = e.target.value.replace(/\D/g, '')
  if (v.length === 0) { valor.value = ''; return }
  v = (parseInt(v) / 100).toFixed(2)
  valor.value = v.replace('.', ',')
  e.target.value = valor.value
}

function valorNumerico() {
  return parseFloat(String(valor.value).replace(',', '.')) || 0
}

async function salvar() {
  if (valor.value === '') {
    erro.value = 'Informe o valor'
    return
  }
  salvando.value = true
  try {
    const banco = await getDb()
    await banco.execute(
      `UPDATE lancamento SET valor = ? WHERE id = ?`,
      [valorNumerico(), props.lancamentoId]
    )
    emit('atualizado')
    emit('fechar')
  } catch (err) {
    console.error('Erro ao salvar cota:', err)
    erro.value = 'Erro ao salvar. Tente novamente.'
  } finally {
    salvando.value = false
  }
}
</script>

<template>
  <AppModal
    titulo="Editar Cota do Período"
    texto-confirmar="Salvar"
    largura="300px"
    @fechar="emit('fechar')"
    @confirmar="salvar"
  >
    <div class="campo">
      <label class="input-label">Valor</label>
      <input
        type="text"
        inputmode="numeric"
        :value="valor"
        :class="{ erro: !!erro }"
        placeholder="0,00"
        @input="mascaraValor"
      />
      <span v-if="erro" class="msg-erro">{{ erro }}</span>
    </div>
  </AppModal>
</template>

<style scoped>
.campo {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

input[type="text"] {
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

input[type="text"]:focus { border-color: #4a9eff; }
input[type="text"].erro  { border-color: #cc4444; }
input[type="text"]::placeholder { color: var(--cor-texto-fraco); }

.msg-erro { font-size: 10px; color: #cc4444; }
</style>
