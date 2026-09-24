<script setup>
import { ref, computed, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'
import AppSelect from './AppSelect.vue'
import AppInputData from './AppInputData.vue'

const props = defineProps({
  lancamento: { type: Object, required: true },
})

const emit = defineEmits(['fechar', 'atualizado', 'excluido'])

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

// ── Modos: 'leitura' | 'edicao' | 'exclusao' | 'confirmar-saida' ──
const modo       = ref('leitura')
const modoAntes  = ref('leitura') // modo de onde veio a confirmação de saída

// formOriginal é snapshot tirado ao entrar no modo edição
const formOriginal = ref({})

function isDirty() {
  return JSON.stringify(form.value) !== JSON.stringify(formOriginal.value)
}

function tentarFechar() {
  if (modo.value === 'edicao' && isDirty()) {
    modoAntes.value = 'fechar'
    modo.value = 'confirmar-saida'
  } else {
    emit('fechar')
  }
}

function tentarCancelarEdicao() {
  if (isDirty()) {
    modoAntes.value = 'leitura'
    modo.value = 'confirmar-saida'
  } else {
    cancelarEdicao()
  }
}

function confirmarSaida() {
  if (modoAntes.value === 'fechar') {
    emit('fechar')
  } else {
    cancelarEdicao()
  }
}

// ── Dados auxiliares ──────────────────────────────────────
const categorias = ref([])
const pessoas    = ref([])

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

function labelCategoria(id) {
  const c = categorias.value.find(c => String(c.id) === String(id))
  return c?.nome ?? '—'
}

function labelPessoa(id) {
  if (!id) return '—'
  const p = pessoas.value.find(p => String(p.id) === String(id))
  return p?.nome ?? '—'
}

function labelFormaPagamento(v) {
  if (!v) return '—'
  const op = opFormasPagamento.find(o => o.value === v)
  return op?.label ?? v
}

function isoParaDisplay(iso) {
  if (!iso) return '—'
  const [a, m, d] = iso.split('-')
  return `${d}/${m}/${a}`
}

async function carregarDados() {
  try {
    const banco = await getDb()
    const filtroDisp = props.lancamento.tipo === 'indireto'
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
        WHERE deletado_em IS NULL ORDER BY nome`
    )
  } catch (err) {
    console.error('Erro ao carregar dados:', err)
    erros.value.geral = 'Erro ao carregar dados.'
  }
}

// ── Formulário de edição ──────────────────────────────────
const form   = ref({})
const erros  = ref({})
const salvando  = ref(false)
const excluindo = ref(false)

function isoParaForm(iso) {
  if (!iso) return ''
  const [a, m, d] = iso.split('-')
  return `${d}/${m}/${a}`
}

function dataParaIso(dataStr) {
  const [d, m, a] = dataStr.split('/')
  return `${a}-${m}-${d}`
}

function valorParaForm(valor) {
  return Number(valor).toLocaleString('pt-BR', { minimumFractionDigits: 2 })
}

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

function entrarModoEdicao() {
  const inicial = {
    data:            isoParaForm(props.lancamento.data),
    categoria_id:    String(props.lancamento.categoria_id),
    descricao:       props.lancamento.descricao ?? '',
    pessoa_id:       props.lancamento.pessoa_id ? String(props.lancamento.pessoa_id) : '',
    valor:           valorParaForm(props.lancamento.valor),
    forma_pagamento: props.lancamento.forma_pagamento ?? '',
  }
  form.value         = { ...inicial }
  formOriginal.value = { ...inicial }
  erros.value = {}
  modo.value = 'edicao'
}

function cancelarEdicao() {
  erros.value = {}
  modo.value = 'leitura'
}

function validar() {
  const e = {}
  if (!form.value.data || form.value.data.length < 10) e.data = 'Informe a data'
  if (!form.value.categoria_id) e.categoria_id = 'Selecione a categoria'
  if (!form.value.descricao.trim()) e.descricao = 'Informe a descrição'
  if (form.value.valor === '') e.valor = 'Informe o valor'
  erros.value = e
  return Object.keys(e).length === 0
}

async function salvar() {
  if (!validar()) return
  salvando.value = true
  try {
    const banco = await getDb()
    await banco.execute(
      `UPDATE lancamento SET
         data            = ?,
         categoria_id    = ?,
         pessoa_id       = ?,
         descricao       = ?,
         valor           = ?,
         forma_pagamento = ?
       WHERE id = ?`,
      [
        dataParaIso(form.value.data),
        parseInt(form.value.categoria_id),
        form.value.pessoa_id ? parseInt(form.value.pessoa_id) : null,
        form.value.descricao.trim(),
        valorNumerico(),
        form.value.forma_pagamento || null,
        props.lancamento.id,
      ]
    )
    emit('atualizado')
    emit('fechar')
  } catch (err) {
    console.error('Erro ao salvar:', err)
    erros.value.geral = 'Erro ao salvar. Tente novamente.'
  } finally {
    salvando.value = false
  }
}

// ── Exclusão ──────────────────────────────────────────────
async function confirmarExclusao() {
  excluindo.value = true
  try {
    const banco = await getDb()
    await banco.execute(
      `UPDATE lancamento SET deletado_em = datetime('now') WHERE id = ?`,
      [props.lancamento.id]
    )
    emit('excluido')
    emit('fechar')
  } catch (err) {
    console.error('Erro ao excluir:', err)
    erros.value.geral = 'Erro ao excluir. Tente novamente.'
    modo.value = 'leitura'
  } finally {
    excluindo.value = false
  }
}

const titulos = {
  leitura:          'Lançamento',
  edicao:           'Editar Lançamento',
  exclusao:         'Excluir Lançamento',
  'confirmar-saida': 'Alterações não salvas',
}

onMounted(carregarDados)
</script>

<template>
  <AppModal
    :titulo="titulos[modo]"
    largura="460px"
    @fechar="tentarFechar"
  >

    <!-- ── Modo leitura ── -->
    <template v-if="modo === 'leitura'">
      <div class="grade-leitura">

        <div class="linha-leitura">
          <div class="campo-leitura">
            <span class="rl-label">Data</span>
            <span class="rl-valor">{{ isoParaDisplay(lancamento.data) }}</span>
          </div>
          <div class="campo-leitura flex-2">
            <span class="rl-label">Categoria</span>
            <span class="rl-valor">{{ labelCategoria(lancamento.categoria_id) }}</span>
          </div>
        </div>

        <div class="campo-leitura">
          <span class="rl-label">Descrição</span>
          <span class="rl-valor">{{ lancamento.descricao || '—' }}</span>
        </div>

        <div class="campo-leitura">
          <span class="rl-label">Responsável</span>
          <span class="rl-valor">{{ labelPessoa(lancamento.pessoa_id) }}</span>
        </div>

        <div class="linha-leitura">
          <div class="campo-leitura">
            <span class="rl-label">Valor</span>
            <span class="rl-valor destaque">{{ valorParaForm(lancamento.valor) }}</span>
          </div>
          <div class="campo-leitura flex-2">
            <span class="rl-label">Forma de pagamento</span>
            <span class="rl-valor">{{ labelFormaPagamento(lancamento.forma_pagamento) }}</span>
          </div>
        </div>

        <div class="campo-leitura">
          <span class="rl-label">Comprovante</span>
          <span class="rl-valor fraco">Nenhum comprovante anexado</span>
        </div>

      </div>
      <span v-if="erros.geral" class="msg-erro">{{ erros.geral }}</span>
    </template>

    <!-- ── Modo edição ── -->
    <template v-else-if="modo === 'edicao'">
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

      <div class="campo">
        <label class="input-label">Descrição</label>
        <input
          type="text"
          v-model="form.descricao"
          :class="{ erro: erros.descricao }"
          maxlength="200"
        />
        <span v-if="erros.descricao" class="msg-erro">{{ erros.descricao }}</span>
      </div>

      <div class="campo">
        <AppSelect
          label="Responsável (opcional)"
          v-model="form.pessoa_id"
          :options="opPessoas"
          placeholder="Nenhum"
        />
      </div>

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

      <div class="campo">
        <label class="input-label">Comprovante</label>
        <div class="comprovante-placeholder">
          <span class="comprovante-texto">Nenhum comprovante selecionado</span>
          <button class="btn-selecionar-comp" type="button" disabled>
            Selecionar comprovante
          </button>
        </div>
      </div>

      <span v-if="erros.geral" class="msg-erro">{{ erros.geral }}</span>
    </template>

    <!-- ── Modo confirmar saída (dirty) ── -->
    <template v-else-if="modo === 'confirmar-saida'">
      <div class="exclusao-corpo">
        <p class="exclusao-msg">Há alterações não salvas. Deseja descartar as alterações?</p>
      </div>
    </template>

    <!-- ── Modo exclusão ── -->
    <template v-else-if="modo === 'exclusao'">
      <div class="exclusao-corpo">
        <p class="exclusao-msg">
          Tem certeza que deseja excluir este lançamento?
        </p>
        <div class="exclusao-resumo">
          <span class="rl-label">Data</span>
          <span class="rl-valor">{{ isoParaDisplay(lancamento.data) }}</span>
          <span class="rl-label">Descrição</span>
          <span class="rl-valor">{{ lancamento.descricao || '—' }}</span>
          <span class="rl-label">Valor</span>
          <span class="rl-valor destaque">{{ valorParaForm(lancamento.valor) }}</span>
        </div>
        <span v-if="erros.geral" class="msg-erro">{{ erros.geral }}</span>
      </div>
    </template>

    <!-- ── Rodapé ── -->
    <template #footer>

      <!-- Leitura -->
      <template v-if="modo === 'leitura'">
        <button class="btn-perigo" @click="modo = 'exclusao'">Excluir</button>
        <div class="footer-acoes">
          <button class="btn-cancelar" @click="emit('fechar')">Fechar</button>
          <button class="btn-confirmar" @click="entrarModoEdicao">Editar</button>
        </div>
      </template>

      <!-- Edição -->
      <template v-else-if="modo === 'edicao'">
        <button class="btn-cancelar" @click="tentarCancelarEdicao">Cancelar edição</button>
        <div class="footer-acoes">
          <button class="btn-cancelar" @click="tentarFechar">Fechar</button>
          <button class="btn-confirmar" :disabled="salvando" @click="salvar">
            {{ salvando ? 'Salvando...' : 'Salvar' }}
          </button>
        </div>
      </template>

      <!-- Confirmar saída (dirty) -->
      <template v-else-if="modo === 'confirmar-saida'">
        <button class="btn-cancelar" @click="modo = 'edicao'">Voltar à edição</button>
        <div class="footer-acoes">
          <button class="btn-perigo" @click="confirmarSaida">Descartar alterações</button>
        </div>
      </template>

      <!-- Exclusão -->
      <template v-else-if="modo === 'exclusao'">
        <button class="btn-cancelar" @click="modo = 'leitura'">Cancelar</button>
        <div class="footer-acoes">
          <button class="btn-perigo" :disabled="excluindo" @click="confirmarExclusao">
            {{ excluindo ? 'Excluindo...' : 'Confirmar exclusão' }}
          </button>
        </div>
      </template>

    </template>

  </AppModal>
</template>

<style scoped>
/* ── Leitura ── */
.grade-leitura {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.linha-leitura {
  display: flex;
  gap: 16px;
}

.linha-leitura .campo-leitura { flex: 1; }
.flex-2 { flex: 2 !important; }

.campo-leitura {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.rl-label {
  font-size: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--cor-texto-fraco);
}

.rl-valor {
  font-size: 13px;
  color: var(--cor-texto-forte);
  padding: 5px 0;
  border-bottom: 1px solid #2a2a2a;
}

.rl-valor.destaque { color: #4a9eff; font-weight: bold; }
.rl-valor.fraco    { color: var(--cor-texto-fraco); font-style: italic; }

/* ── Edição ── */
.linha-2col {
  display: flex;
  gap: 12px;
}

.linha-2col .campo { flex: 1; }

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

/* ── Exclusão ── */
.exclusao-corpo {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.exclusao-msg {
  font-size: 13px;
  color: var(--cor-texto);
  margin: 0;
}

.exclusao-resumo {
  display: grid;
  grid-template-columns: 80px 1fr;
  gap: 6px 12px;
  align-items: baseline;
  padding: 12px;
  background-color: #1a1a1a;
  border-radius: 4px;
  border: 1px solid var(--cor-borda);
}

/* ── Rodapé ── */
.footer-acoes {
  display: flex;
  gap: 8px;
  margin-left: auto;
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

.btn-perigo {
  background-color: transparent;
  color: #cc6666;
  border: 1px solid #cc6666;
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.btn-perigo:hover:not(:disabled) { background-color: #3a1a1a; color: #ff6666; border-color: #ff6666; }
.btn-perigo:disabled { opacity: 0.5; cursor: default; }

.msg-erro { font-size: 10px; color: #cc4444; }
</style>
