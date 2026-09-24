<script setup>
import { ref, computed, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'
import AppSelect from './AppSelect.vue'
import AppInputData from './AppInputData.vue'
import AppToggle from './AppToggle.vue'

const props = defineProps({
  periodoId: { type: Number, required: true },
  tipo:      { type: String, default: 'direto' },  // 'direto' | 'indireto'
  pessoaId:  { type: Number, default: null },       // pré-selecionada nos indiretos
})

const emit = defineEmits(['fechar', 'criado'])

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

// ── Dados auxiliares ──────────────────────────────────────
const categorias = ref([])
const pessoas    = ref([])
const contaAtiva = ref(null)

const opCategorias = computed(() =>
  categorias.value.map(c => ({ value: String(c.id), label: c.nome }))
)

const opPessoas = computed(() =>
  pessoas.value.map(p => ({ value: String(p.id), label: p.nome }))
)

const opFormasPagamento = [
  { value: 'a_vista',       label: 'À vista' },
  { value: 'pix',           label: 'Pix' },
  { value: 'transferencia', label: 'Transferência' },
  { value: 'cheque',        label: 'Cheque' },
  { value: 'faturado',      label: 'Faturado' },
]

async function carregarDados() {
  try {
    const banco = await getDb()

    const filtroDisp = props.tipo === 'indireto'
      ? `('indireto','ambos')`
      : `('direto','ambos')`
    categorias.value = await banco.select(
      `SELECT id, nome FROM categoria
        WHERE deletado_em IS NULL AND ativa = 1 AND tipo != 'sistema'
          AND disponivel_em IN ${filtroDisp}
        ORDER BY nome`
    )

    pessoas.value = await banco.select(
      `SELECT id, nome FROM pessoa
        WHERE deletado_em IS NULL
        ORDER BY nome`
    )

    const contas = await banco.select(
      `SELECT id FROM conta WHERE ativa = 1 AND deletado_em IS NULL LIMIT 1`
    )
    contaAtiva.value = contas.length > 0 ? contas[0].id : null
  } catch (err) {
    console.error('Erro ao carregar dados do modal:', err)
    erros.value.geral = 'Erro ao carregar dados. Tente novamente.'
  }
}

// ── Formulário ────────────────────────────────────────────
const hoje   = new Date()
const dataHoje = [
  String(hoje.getDate()).padStart(2, '0'),
  String(hoje.getMonth() + 1).padStart(2, '0'),
  String(hoje.getFullYear()),
].join('/')

const form = ref({
  data:            dataHoje,
  categoria_id:    '',
  descricao:       '',
  pessoa_id:       props.pessoaId ? String(props.pessoaId) : '',
  valor:           '',
  forma_pagamento: '',
})

const continuarDigitando = ref(false)
const erros   = ref({})
const salvando = ref(false)

function mascaraValor(e) {
  let v = e.target.value.replace(/\D/g, '')
  if (v.length === 0) { form.value.valor = ''; return }
  v = (parseInt(v) / 100).toFixed(2)
  form.value.valor = v.replace('.', ',')
  e.target.value = form.value.valor
}

function valorNumerico() {
  return parseFloat(form.value.valor.replace(',', '.')) || 0
}

function dataParaIso(dataStr) {
  const [d, m, a] = dataStr.split('/')
  return `${a}-${m}-${d}`
}

function validar() {
  const e = {}
  if (!form.value.data || form.value.data.length < 10) e.data = 'Informe a data'
  if (!form.value.categoria_id) e.categoria_id = 'Selecione a categoria'
  if (!form.value.descricao.trim()) e.descricao = 'Informe a descrição'
  if (form.value.valor === '') e.valor = 'Informe o valor'
  if (props.tipo === 'indireto' && !props.pessoaId && !form.value.pessoa_id)
    e.pessoa_id = 'Selecione o curador'
  erros.value = e
  return Object.keys(e).length === 0
}

function resetarForm() {
  form.value = {
    data:            form.value.data,
    categoria_id:    '',
    descricao:       '',
    pessoa_id:       props.pessoaId ? String(props.pessoaId) : '',
    valor:           '',
    forma_pagamento: '',
  }
  erros.value = {}
}

async function salvar() {
  if (!validar()) return
  if (!contaAtiva.value) {
    erros.value.geral = 'Nenhuma conta ativa encontrada.'
    return
  }

  salvando.value = true
  try {
    const banco = await getDb()

    // Nos indiretos, ordem começa em 3 (0,1,2 são linhas fixas)
    // A constraint UNIQUE(periodo_id, tipo, ordem) é global — não filtra por pessoa
    const ordemMin = props.tipo === 'indireto' ? 3 : 0
    const res = await banco.select(
      `SELECT COALESCE(MAX(ordem), ?) + 1 AS proxima
         FROM lancamento
        WHERE periodo_id = ? AND tipo = ? AND deletado_em IS NULL`,
      [ordemMin - 1, props.periodoId, props.tipo]
    )
    const proxima = Math.max(res[0].proxima, ordemMin)

    const result = await banco.execute(
      `INSERT INTO lancamento
         (periodo_id, categoria_id, conta_id, pessoa_id, data, descricao,
          tipo, valor, forma_pagamento, ordem)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
      [
        props.periodoId,
        parseInt(form.value.categoria_id),
        contaAtiva.value,
        form.value.pessoa_id ? parseInt(form.value.pessoa_id) : null,
        dataParaIso(form.value.data),
        form.value.descricao.trim(),
        props.tipo,
        valorNumerico(),
        form.value.forma_pagamento || null,
        proxima,
      ]
    )

    emit('criado', { id: result.lastInsertId })

    if (continuarDigitando.value) {
      resetarForm()
    } else {
      emit('fechar')
    }
  } catch (err) {
    console.error('Erro ao salvar lançamento:', err)
    erros.value.geral = 'Erro ao salvar. Tente novamente.'
  } finally {
    salvando.value = false
  }
}

onMounted(carregarDados)
</script>

<template>
  <AppModal
    :titulo="tipo === 'indireto' ? 'Novo Lançamento Indireto' : 'Novo Lançamento'"
    texto-confirmar="Salvar"
    largura="460px"
    @fechar="emit('fechar')"
    @confirmar="salvar"
  >
    <!-- Data + Categoria -->
    <div class="linha-2col">
      <div class="campo">
        <AppInputData label="Data" v-model="form.data" />
        <span v-if="erros.data" class="msg-erro">{{ erros.data }}</span>
      </div>
      <div class="campo flex-2">
        <AppSelect
          label="Categoria"
          v-model="form.categoria_id"
          :options="opCategorias"
          placeholder="Selecione a categoria"
        />
        <span v-if="erros.categoria_id" class="msg-erro">{{ erros.categoria_id }}</span>
      </div>
    </div>

    <!-- Descrição -->
    <div class="campo">
      <label class="input-label">Descrição</label>
      <input
        type="text"
        v-model="form.descricao"
        :class="{ erro: erros.descricao }"
        placeholder="Ex: Conta de luz"
        maxlength="200"
      />
      <span v-if="erros.descricao" class="msg-erro">{{ erros.descricao }}</span>
    </div>

    <!-- Responsável -->
    <div class="campo">
      <AppSelect
        :label="tipo === 'indireto' ? 'Curador' : 'Responsável (opcional)'"
        v-model="form.pessoa_id"
        :options="opPessoas"
        :placeholder="tipo === 'indireto' ? 'Selecione o curador' : 'Nenhum'"
      />
      <span v-if="erros.pessoa_id" class="msg-erro">{{ erros.pessoa_id }}</span>
    </div>

    <!-- Valor + Forma de pagamento -->
    <div class="linha-2col">
      <div class="campo">
        <label class="input-label">Valor</label>
        <input
          type="text"
          inputmode="numeric"
          :value="form.valor"
          :class="{ erro: erros.valor }"
          placeholder="0,00"
          @input="mascaraValor"
        />
        <span v-if="erros.valor" class="msg-erro">{{ erros.valor }}</span>
      </div>
      <div class="campo flex-2">
        <AppSelect
          label="Forma de pagamento"
          v-model="form.forma_pagamento"
          :options="opFormasPagamento"
          placeholder="Selecione"
        />
      </div>
    </div>

    <!-- Comprovante -->
    <div class="campo">
      <label class="input-label">Comprovante</label>
      <div class="comprovante-placeholder">
        <span class="comprovante-texto">Nenhum comprovante selecionado</span>
        <button class="btn-selecionar-comp" type="button" disabled>
          Selecionar comprovante
        </button>
      </div>
      <span class="hint">Disponível após cadastrar comprovantes no inbox.</span>
    </div>

    <!-- Erro geral -->
    <span v-if="erros.geral" class="msg-erro">{{ erros.geral }}</span>

    <!-- Rodapé com toggle -->
    <template #footer>
      <div class="footer-toggle" style="user-select: auto">
        <AppToggle label="Continuar digitando" v-model="continuarDigitando" />
      </div>
      <div class="footer-acoes">
        <button class="btn-cancelar" @click="emit('fechar')">Cancelar</button>
        <button class="btn-confirmar" :disabled="salvando" @click="salvar">
          {{ salvando ? 'Salvando...' : 'Salvar' }}
        </button>
      </div>
    </template>

  </AppModal>
</template>

<style scoped>
.linha-2col {
  display: flex;
  gap: 12px;
}

.linha-2col .campo { flex: 1; }
.flex-2 { flex: 2 !important; }

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

input[type="text"]:focus  { border-color: #4a9eff; }
input[type="text"].erro   { border-color: #cc4444; }
input[type="text"]::placeholder { color: var(--cor-texto-fraco); }

.msg-erro { font-size: 10px; color: #cc4444; }

/* ── Comprovante ── */
.comprovante-placeholder {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  min-height: 34px;
}

.comprovante-texto {
  flex: 1;
  font-size: 12px;
  color: var(--cor-texto-fraco);
}

.btn-selecionar-comp {
  background: none;
  border: 1px solid var(--cor-borda);
  color: var(--cor-texto-fraco);
  padding: 3px 10px;
  border-radius: 4px;
  font-size: 11px;
  font-family: inherit;
  cursor: not-allowed;
  flex-shrink: 0;
  opacity: 0.5;
}

.hint {
  font-size: 10px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

/* ── Rodapé ── */
.footer-toggle {
  flex: 1;
  max-width: 180px;
  margin-right: 48px;
}

.footer-acoes {
  display: flex;
  gap: 8px;
}

.btn-confirmar {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.btn-confirmar:disabled { opacity: 0.5; cursor: default; }

.btn-cancelar {
  background-color: transparent;
  color: var(--cor-texto);
  border: 1px solid var(--cor-borda);
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}
</style>
