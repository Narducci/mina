<script setup>
import { ref, computed, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'
import AppSelect from './AppSelect.vue'

const props = defineProps({
  periodo: { type: Object, default: null },
})

const emit = defineEmits(['fechar', 'criado', 'atualizado'])

const modoEdicao = computed(() => !!props.periodo)

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

// Controles de travamento
const travaMesAno = ref(false)      // true se há lançamentos
const travaSaldo = ref(false)       // true se período anterior está conciliado
const travaExtrato = ref(false)     // true se período está conciliado
// Nome do extrato já salvo no banco (modo edição)
const nomeExtratoSalvo = ref('')
const removerExtratoSalvo = ref(false)

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

// Nome exibido no campo de extrato
const nomeExtrato = computed(() => {
  if (form.value.extrato) return form.value.extrato.name
  if (!removerExtratoSalvo.value && nomeExtratoSalvo.value) return nomeExtratoSalvo.value
  return ''
})

// Há algum extrato ativo (novo ou salvo não removido)
const temExtrato = computed(() =>
  !!form.value.extrato || (!removerExtratoSalvo.value && !!nomeExtratoSalvo.value)
)

function selecionarExtrato() {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.pdf,.png,.jpg,.jpeg'
  input.onchange = (e) => {
    form.value.extrato = e.target.files[0] ?? null
    removerExtratoSalvo.value = false
  }
  input.click()
}

function removerExtrato() {
  form.value.extrato = null
  removerExtratoSalvo.value = true
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
  if (!travaMesAno.value) {
    if (!form.value.mes) e.mes = 'Selecione o mês'
    const ano = Number(form.value.ano)
    if (!form.value.ano || isNaN(ano) || ano < 1900 || ano > 2100) e.ano = 'Ano inválido'
  }
  if (!travaSaldo.value && form.value.saldo_inicial === '') {
    e.saldo_inicial = 'Informe o saldo inicial'
  }
  erros.value = e
  return Object.keys(e).length === 0
}

function valorNumerico() {
  return parseFloat(String(form.value.saldo_inicial).replace(',', '.')) || 0
}

// ── Modo criação ───────────────────────────────────────────
async function criar() {
  if (!validar()) return
  salvando.value = true
  try {
    const banco = await getDb()

    const existe = await banco.select(
      'SELECT id FROM periodo WHERE mes = ? AND ano = ? AND deletado_em IS NULL',
      [parseInt(form.value.mes), parseInt(form.value.ano)]
    )
    if (existe.length > 0) {
      erros.value.mes = 'Período já cadastrado'
      return
    }

    let comprovante_id = null
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

    const result = await banco.execute(
      `INSERT INTO periodo (mes, ano, saldo_inicial, comprovante_id)
       VALUES (?, ?, ?, ?)`,
      [parseInt(form.value.mes), parseInt(form.value.ano), valorNumerico(), comprovante_id]
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

// ── Modo edição ────────────────────────────────────────────
async function salvar() {
  if (!validar()) return
  salvando.value = true
  try {
    const banco = await getDb()

    // Verificar duplicata de mês/ano (excluindo o próprio período)
    if (!travaMesAno.value) {
      const existe = await banco.select(
        'SELECT id FROM periodo WHERE mes = ? AND ano = ? AND deletado_em IS NULL AND id != ?',
        [parseInt(form.value.mes), parseInt(form.value.ano), props.periodo.id]
      )
      if (existe.length > 0) {
        erros.value.mes = 'Período já cadastrado'
        return
      }
    }

    let comprovante_id = props.periodo.comprovante_id ?? null

    // Novo extrato selecionado
    if (form.value.extrato) {
      const nome = form.value.extrato.name
      const hash = `${nome}_${form.value.extrato.size}_${form.value.extrato.lastModified}`
      const result = await banco.execute(
        `INSERT INTO comprovante (hash_arquivo, caminho_relativo, nome, status)
         VALUES (?, ?, ?, 'inbox')`,
        [hash, nome, nome]
      )
      comprovante_id = result.lastInsertId
    } else if (removerExtratoSalvo.value) {
      comprovante_id = null
    }

    await banco.execute(
      `UPDATE periodo SET mes = ?, ano = ?, saldo_inicial = ?, comprovante_id = ?
       WHERE id = ?`,
      [
        travaMesAno.value ? props.periodo.mes : parseInt(form.value.mes),
        travaMesAno.value ? props.periodo.ano : parseInt(form.value.ano),
        travaSaldo.value ? props.periodo.saldo_inicial : valorNumerico(),
        comprovante_id,
        props.periodo.id,
      ]
    )

    emit('atualizado')
    emit('fechar')
  } catch (err) {
    console.error('Erro ao editar período:', err)
    erros.value.geral = 'Erro ao salvar. Tente novamente.'
  } finally {
    salvando.value = false
  }
}

function confirmar() {
  if (modoEdicao.value) salvar()
  else criar()
}

// ── Inicialização (modo edição) ────────────────────────────
onMounted(async () => {
  if (!modoEdicao.value) return
  try {
    const banco = await getDb()
    const p = props.periodo

    // Preenche form com dados atuais
    form.value.mes = String(p.mes).padStart(2, '0')
    form.value.ano = String(p.ano)
    const saldo = Math.abs(p.saldo_inicial).toFixed(2).replace('.', ',')
    form.value.saldo_inicial = saldo

    // Nome do extrato salvo
    if (p.comprovante_id) {
      const rows = await banco.select(
        'SELECT nome FROM comprovante WHERE id = ?', [p.comprovante_id]
      )
      nomeExtratoSalvo.value = rows[0]?.nome ?? ''
    }

    // Trava mês/ano: há lançamentos?
    const lanc = await banco.select(
      'SELECT id FROM lancamento WHERE periodo_id = ? AND deletado_em IS NULL LIMIT 1',
      [p.id]
    )
    travaMesAno.value = lanc.length > 0

    // Trava saldo: período anterior conciliado?
    const anterior = await banco.select(
      `SELECT status FROM periodo
       WHERE deletado_em IS NULL AND (ano < ? OR (ano = ? AND mes < ?))
       ORDER BY ano DESC, mes DESC LIMIT 1`,
      [p.ano, p.ano, p.mes]
    )
    travaSaldo.value = anterior.length > 0 && anterior[0].status === 'conciliado'

    // Trava extrato: período conciliado?
    travaExtrato.value = p.status === 'conciliado'

  } catch (err) {
    console.error('Erro ao carregar dados do período:', err)
    erros.value.geral = 'Erro ao carregar dados.'
  }
})
</script>

<template>
  <AppModal
    :titulo="modoEdicao ? 'Editar Período' : 'Novo Período'"
    :texto-confirmar="modoEdicao ? 'Salvar' : 'Criar'"
    @fechar="emit('fechar')"
    @confirmar="confirmar"
  >
    <div class="campo-linha">
      <div class="campo-mes">
        <AppSelect
          label="Mês"
          v-model="form.mes"
          :options="opMeses"
          :disabled="travaMesAno"
        />
        <span v-if="erros.mes" class="msg-erro">{{ erros.mes }}</span>
        <span v-if="travaMesAno" class="msg-trava">Período com lançamentos</span>
      </div>
      <div class="campo-ano">
        <label class="input-label">Ano</label>
        <input
          type="text"
          inputmode="numeric"
          maxlength="4"
          v-model="form.ano"
          :disabled="travaMesAno"
          :class="{ erro: erros.ano, desabilitado: travaMesAno }"
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
        :disabled="travaSaldo"
        :class="{ erro: erros.saldo_inicial, desabilitado: travaSaldo }"
        placeholder="0,00"
        @input="mascaraValor"
      />
      <span v-if="erros.saldo_inicial" class="msg-erro">{{ erros.saldo_inicial }}</span>
      <span v-if="travaSaldo" class="msg-trava">Período anterior conciliado</span>
    </div>

    <div class="campo">
      <label class="input-label">Extrato <span class="opcional">(opcional)</span></label>
      <div class="extrato-box" :class="{ desabilitado: travaExtrato }">
        <span class="extrato-nome">{{ nomeExtrato || 'Nenhum arquivo selecionado' }}</span>
        <button
          class="btn-extrato"
          :disabled="travaExtrato"
          @click="selecionarExtrato"
        >Selecionar</button>
        <button
          v-if="temExtrato && !travaExtrato"
          class="btn-remover"
          @click="removerExtrato"
        >✕</button>
      </div>
      <span v-if="travaExtrato" class="msg-trava">Período conciliado</span>
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

.msg-trava {
  font-size: 10px;
  color: var(--cor-texto-fraco);
  font-style: italic;
}

input.desabilitado,
input:disabled {
  opacity: 0.45;
  cursor: default;
}

.extrato-box.desabilitado {
  opacity: 0.45;
  cursor: default;
}

.extrato-box.desabilitado .btn-extrato {
  cursor: default;
  pointer-events: none;
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
