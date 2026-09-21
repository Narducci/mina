<script setup>
import { ref, computed } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'
import AppSelect from './AppSelect.vue'

const emit = defineEmits(['fechar', 'criado'])

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

const hoje = new Date()

const form = ref({
  mes: String(hoje.getMonth() + 1).padStart(2, '0'),
  ano: String(hoje.getFullYear()),
  saldo_inicial: '',
  extrato: null,
})

const erros = ref({})
const salvando = ref(false)

const opMeses = [
  { value: '01', label: 'Janeiro' },
  { value: '02', label: 'Fevereiro' },
  { value: '03', label: 'Março' },
  { value: '04', label: 'Abril' },
  { value: '05', label: 'Maio' },
  { value: '06', label: 'Junho' },
  { value: '07', label: 'Julho' },
  { value: '08', label: 'Agosto' },
  { value: '09', label: 'Setembro' },
  { value: '10', label: 'Outubro' },
  { value: '11', label: 'Novembro' },
  { value: '12', label: 'Dezembro' },
]

const nomeExtrato = computed(() => form.value.extrato?.name ?? '')

function selecionarExtrato() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.pdf,.png,.jpg,.jpeg'
  input.onchange = (e) => {
    form.value.extrato = e.target.files[0] ?? null
  }
  input.click()
}

function removerExtrato() {
  form.value.extrato = null
}

function mascaraValor(e) {
  let v = e.target.value.replace(/\D/g, '')
  if (v.length === 0) { form.value.saldo_inicial = ''; return }
  v = (parseInt(v) / 100).toFixed(2)
  form.value.saldo_inicial = v.replace('.', ',')
  e.target.value = form.value.saldo_inicial
}

function validar() {
  const e = {}
  if (!form.value.mes) e.mes = 'Selecione o mês'
  const ano = Number(form.value.ano)
  if (!form.value.ano || isNaN(ano) || ano < 1900 || ano > 2100) e.ano = 'Ano inválido'
  if (form.value.saldo_inicial === '') e.saldo_inicial = 'Informe o saldo inicial'
  erros.value = e
  return Object.keys(e).length === 0
}

function valorNumerico() {
  return parseFloat(form.value.saldo_inicial.replace(',', '.')) || 0
}

async function criar() {
  if (!validar()) return
  salvando.value = true
  try {
    const banco = await getDb()

    // Verificar duplicata
    const existe = await banco.select(
      'SELECT id FROM periodo WHERE mes = ? AND ano = ? AND deletado_em IS NULL',
      [parseInt(form.value.mes), parseInt(form.value.ano)]
    )
    if (existe.length > 0) {
      erros.value.mes = 'Período já cadastrado'
      return
    }

    let comprovante_id = null

    // Inserir extrato se fornecido
    if (form.value.extrato) {
      const nome = form.value.extrato.name
      const hash = `${nome}_${form.value.extrato.size}_${form.value.extrato.lastModified}`
      const result = await banco.execute(
        `INSERT INTO comprovante (hash_arquivo, caminho_relativo, nome, status)
         VALUES (?, ?, ?, 'inbox')`,
        [hash, nome, nome]
      )
      comprovante_id = result.lastInsertId
    }

    // Inserir período
    const result = await banco.execute(
      `INSERT INTO periodo (mes, ano, saldo_inicial, comprovante_id)
       VALUES (?, ?, ?, ?)`,
      [
        parseInt(form.value.mes),
        parseInt(form.value.ano),
        valorNumerico(),
        comprovante_id,
      ]
    )

    emit('criado', { id: result.lastInsertId, mes: form.value.mes, ano: form.value.ano })
    emit('fechar')
  } catch (err) {
    console.error('Erro ao criar período:', err)
    erros.value.geral = 'Erro ao salvar. Tente novamente.'
  } finally {
    salvando.value = false
  }
}
</script>

<template>
  <AppModal
    titulo="Novo Período"
    texto-confirmar="Criar"
    @fechar="emit('fechar')"
    @confirmar="criar"
  >
    <div class="campo-linha">
      <div class="campo-mes">
        <AppSelect
          label="Mês"
          v-model="form.mes"
          :options="opMeses"
        />
        <span v-if="erros.mes" class="msg-erro">{{ erros.mes }}</span>
      </div>
      <div class="campo-ano">
        <label class="input-label">Ano</label>
        <input
          type="text"
          inputmode="numeric"
          maxlength="4"
          v-model="form.ano"
          :class="{ erro: erros.ano }"
        />
        <span v-if="erros.ano" class="msg-erro">{{ erros.ano }}</span>
      </div>
    </div>

    <div class="campo">
      <label class="input-label">Saldo Inicial</label>
      <input
        type="text"
        inputmode="numeric"
        :value="form.saldo_inicial"
        :class="{ erro: erros.saldo_inicial }"
        placeholder="0,00"
        @input="mascaraValor"
      />
      <span v-if="erros.saldo_inicial" class="msg-erro">{{ erros.saldo_inicial }}</span>
    </div>

    <div class="campo">
      <label class="input-label">Extrato <span class="opcional">(opcional)</span></label>
      <div class="extrato-box">
        <span class="extrato-nome">{{ nomeExtrato || 'Nenhum arquivo selecionado' }}</span>
        <button class="btn-extrato" @click="selecionarExtrato">Selecionar</button>
        <button v-if="form.extrato" class="btn-remover" @click="removerExtrato">✕</button>
      </div>
    </div>

    <span v-if="erros.geral" class="msg-erro">{{ erros.geral }}</span>

  </AppModal>
</template>

<style scoped>
.campo {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campo-linha {
  display: flex;
  gap: 12px;
}

.campo-mes {
  flex: 2;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campo-ano {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.input-label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

.opcional {
  font-size: 10px;
  color: var(--cor-texto-fraco);
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

.extrato-box {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  min-height: 32px;
}

.extrato-nome {
  flex: 1;
  font-size: 12px;
  color: var(--cor-texto-fraco);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-extrato {
  background: none;
  border: 1px solid var(--cor-borda);
  color: var(--cor-texto);
  padding: 3px 10px;
  border-radius: 4px;
  font-size: 11px;
  font-family: inherit;
  cursor: pointer;
  flex-shrink: 0;
}

.btn-extrato:hover {
  border-color: #4a9eff;
  color: var(--cor-texto-forte);
}

.btn-remover {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 13px;
  flex-shrink: 0;
  padding: 0 2px;
}

.btn-remover:hover {
  color: #cc4444;
}
</style>
