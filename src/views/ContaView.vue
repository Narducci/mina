<script setup>
import { ref, reactive, computed, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'

// ── Banco ─────────────────────────────────────────────────────────────────
let db
async function conectar() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

// ── Dados ─────────────────────────────────────────────────────────────────
const registros   = ref([])
const curatelados = ref([])

async function carregar() {
  const d = await conectar()
  registros.value = await d.select(`
    SELECT c.*, p.nome as pessoa_nome
    FROM conta c
    JOIN pessoa p ON p.id = c.pessoa_id
    WHERE c.deletado_em IS NULL
    ORDER BY p.nome, c.banco
  `)
  curatelados.value = await d.select(
    `SELECT id, nome FROM pessoa WHERE tipo = 'curatelado' AND deletado_em IS NULL ORDER BY nome`
  )
}

onMounted(carregar)

// ── Opções ────────────────────────────────────────────────────────────────
const opTipo = [
  { value: 'corrente', label: 'Corrente' },
  { value: 'poupanca', label: 'Poupança' },
]

// ── Confirmação interna ───────────────────────────────────────────────────
const confirmar = ref({ visivel: false, mensagem: '', resolver: null })

function abrirConfirmacao(mensagem) {
  return new Promise((resolve) => {
    confirmar.value = { visivel: true, mensagem, resolver: resolve }
  })
}

function responderConfirmacao(resposta) {
  confirmar.value.visivel = false
  confirmar.value.resolver(resposta)
}

// ── Aviso interno ─────────────────────────────────────────────────────────
const aviso = ref({ visivel: false, mensagem: '' })

function abrirAviso(mensagem) {
  return new Promise((resolve) => {
    aviso.value = { visivel: true, mensagem, resolver: resolve }
  })
}

function fecharAviso() {
  aviso.value.visivel = false
  aviso.value.resolver()
}

// ── Modal ─────────────────────────────────────────────────────────────────
const modalAberto  = ref(false)
const modoEdicao   = ref(false)
const erros        = reactive({})

const formInicial  = () => ({
  id: null, pessoa_id: '', banco: '', agencia: '', numero: '', tipo: '', ativa: 0,
})

const form         = reactive(formInicial())
const formOriginal = reactive(formInicial())

const dirty = computed(() =>
  form.pessoa_id !== formOriginal.pessoa_id ||
  form.banco     !== formOriginal.banco     ||
  form.agencia   !== formOriginal.agencia   ||
  form.numero    !== formOriginal.numero    ||
  form.tipo      !== formOriginal.tipo
)

function abrirNovo() {
  Object.assign(form, formInicial())
  Object.assign(formOriginal, formInicial())
  Object.assign(erros, {})
  modoEdicao.value = false
  modalAberto.value = true
}

function abrirEdicao(reg) {
  const dados = {
    id: reg.id, pessoa_id: reg.pessoa_id, banco: reg.banco,
    agencia: reg.agencia, numero: reg.numero, tipo: reg.tipo, ativa: reg.ativa,
  }
  Object.assign(form, dados)
  Object.assign(formOriginal, dados)
  Object.assign(erros, {})
  modoEdicao.value = true
  modalAberto.value = true
}

async function fecharModal() {
  if (dirty.value) {
    const ok = await abrirConfirmacao('Há alterações não salvas. Deseja descartar?')
    if (!ok) return
  }
  modalAberto.value = false
}

// ── Salvar ────────────────────────────────────────────────────────────────
async function salvar() {
  Object.assign(erros, {})
  let valido = true

  if (!form.pessoa_id)     { erros.pessoa_id = 'Selecione o curatelado'; valido = false }
  if (!form.banco.trim())  { erros.banco     = 'Banco é obrigatório';    valido = false }
  if (!form.agencia.trim()){ erros.agencia   = 'Agência é obrigatória';  valido = false }
  if (!form.numero.trim()) { erros.numero    = 'Número é obrigatório';   valido = false }
  if (!form.tipo)          { erros.tipo      = 'Tipo é obrigatório';     valido = false }

  if (!valido) return

  try {
    const d = await conectar()
    if (modoEdicao.value) {
      await d.execute(
        `UPDATE conta SET pessoa_id=?, banco=?, agencia=?, numero=?, tipo=? WHERE id=?`,
        [form.pessoa_id, form.banco.trim(), form.agencia.trim(), form.numero.trim(), form.tipo, form.id]
      )
    } else {
      await d.execute(
        `INSERT INTO conta (pessoa_id, banco, agencia, numero, tipo, ativa) VALUES (?,?,?,?,?,0)`,
        [form.pessoa_id, form.banco.trim(), form.agencia.trim(), form.numero.trim(), form.tipo]
      )
    }
    modalAberto.value = false
    await carregar()
  } catch (e) {
    erros.numero = 'Erro ao salvar. Verifique se agência e número já estão cadastrados.'
  }
}

// ── Excluir ───────────────────────────────────────────────────────────────
async function excluir() {
  const d = await conectar()

  if (form.ativa) {
    await abrirAviso('Esta conta está ativa. Utilize o processo "Ativar Conta" para transferir a ativação antes de excluir.')
    return
  }

  const ok = await abrirConfirmacao(`Excluir conta ${form.banco} — Ag. ${form.agencia} / ${form.numero}? Esta ação não pode ser desfeita.`)
  if (!ok) return

  try {
    await d.execute(`UPDATE conta SET deletado_em=datetime('now') WHERE id=?`, [form.id])
    modalAberto.value = false
    await carregar()
  } catch (e) {
    erros.numero = 'Erro ao excluir. Tente novamente.'
  }
}

// ── Helpers ───────────────────────────────────────────────────────────────
function iniciais(nome) {
  return nome.split(' ').filter(Boolean).slice(0, 2).map(p => p[0].toUpperCase()).join('')
}

// ── Trap foco ─────────────────────────────────────────────────────────────
function trapFoco(e) {
  if (!modalAberto.value) return
  const modal = document.querySelector('.modal')
  if (!modal) return
  const focaveis = modal.querySelectorAll('button, input, select')
  const primeiro = focaveis[0]
  const ultimo   = focaveis[focaveis.length - 1]
  if (e.key === 'Tab') {
    if (e.shiftKey && document.activeElement === primeiro) {
      e.preventDefault(); ultimo.focus()
    } else if (!e.shiftKey && document.activeElement === ultimo) {
      e.preventDefault(); primeiro.focus()
    }
  }
  if (e.key === 'Escape') fecharModal()
}
</script>

<template>
  <div class="conta-view" @keydown="trapFoco">

    <div class="toolbar">
      <span class="titulo">Contas</span>
      <button class="btn-novo" @click="abrirNovo">+ Nova Conta</button>
    </div>

    <div class="cards-grid">
      <div v-for="reg in registros" :key="reg.id" class="card"
        :class="{ inativa: !reg.ativa }" @dblclick="abrirEdicao(reg)">
        <div class="card-avatar">
          <span class="avatar-iniciais">{{ iniciais(reg.pessoa_nome) }}</span>
        </div>
        <div class="card-pessoa">{{ reg.pessoa_nome }}</div>
        <div class="card-banco">{{ reg.banco }}</div>
        <div class="card-info">Ag. {{ reg.agencia }} / {{ reg.numero }}</div>
        <div class="card-tipo">{{ reg.tipo === 'corrente' ? 'Corrente' : 'Poupança' }}</div>
        <div class="card-status" :class="reg.ativa ? 'ativa' : 'inativa-tag'">
          {{ reg.ativa ? 'Ativa' : 'Inativa' }}
        </div>
        <div class="card-footer">
          <button class="btn-editar" @click="abrirEdicao(reg)" title="Editar">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24"
              fill="none" stroke="currentColor" stroke-width="2"
              stroke-linecap="round" stroke-linejoin="round">
              <path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Modal principal -->
    <div v-if="modalAberto" class="modal-overlay" @click.self="fecharModal">
      <div class="modal" role="dialog" aria-modal="true">
        <div class="modal-header">
          <span>{{ modoEdicao ? 'Editar Conta' : 'Nova Conta' }}</span>
          <button class="btn-fechar" @click="fecharModal">✕</button>
        </div>

        <div class="modal-body">

          <!-- Status (somente leitura na edição) -->
          <div v-if="modoEdicao" class="campo-info">
            <span class="info-label">Status</span>
            <span class="info-valor" :class="form.ativa ? 'ativa' : 'inativa-tag'">
              {{ form.ativa ? 'Ativa' : 'Inativa' }}
            </span>
          </div>

          <div class="campo">
            <label>Curatelado</label>
            <select v-model="form.pessoa_id" :class="{ erro: erros.pessoa_id }" :disabled="modoEdicao">
              <option value="" disabled></option>
              <option v-for="p in curatelados" :key="p.id" :value="p.id">{{ p.nome }}</option>
            </select>
            <span v-if="erros.pessoa_id" class="msg-erro">{{ erros.pessoa_id }}</span>
          </div>

          <div class="campo">
            <label>Banco</label>
            <input type="text" v-model="form.banco" @input="erros.banco = ''" :class="{ erro: erros.banco }" />
            <span v-if="erros.banco" class="msg-erro">{{ erros.banco }}</span>
          </div>

          <div class="campo">
            <label>Agência</label>
            <input type="text" v-model="form.agencia" @input="erros.agencia = ''" :class="{ erro: erros.agencia }" />
            <span v-if="erros.agencia" class="msg-erro">{{ erros.agencia }}</span>
          </div>

          <div class="campo">
            <label>Número</label>
            <input type="text" v-model="form.numero" @input="erros.numero = ''" :class="{ erro: erros.numero }" />
            <span v-if="erros.numero" class="msg-erro">{{ erros.numero }}</span>
          </div>

          <div class="campo">
            <label>Tipo</label>
            <select v-model="form.tipo" :class="{ erro: erros.tipo }">
              <option value="" disabled></option>
              <option v-for="op in opTipo" :key="op.value" :value="op.value">{{ op.label }}</option>
            </select>
            <span v-if="erros.tipo" class="msg-erro">{{ erros.tipo }}</span>
          </div>

        </div>

        <div class="modal-footer">
          <button v-if="modoEdicao" class="btn-excluir" @click="excluir">Excluir</button>
          <button class="btn-cancelar" @click="fecharModal">Cancelar</button>
          <button class="btn-salvar" @click="salvar">Salvar</button>
        </div>
      </div>
    </div>

    <!-- Modal de confirmação -->
    <div v-if="confirmar.visivel" class="modal-overlay" style="z-index: 9100;">
      <div class="modal modal-confirm" role="alertdialog">
        <div class="modal-body">
          <p class="confirm-msg">{{ confirmar.mensagem }}</p>
        </div>
        <div class="modal-footer">
          <button class="btn-cancelar" @click="responderConfirmacao(false)">Não</button>
          <button class="btn-salvar"   @click="responderConfirmacao(true)">Sim</button>
        </div>
      </div>
    </div>

    <!-- Modal de aviso -->
    <div v-if="aviso.visivel" class="modal-overlay" style="z-index: 9100;">
      <div class="modal modal-confirm" role="alertdialog">
        <div class="modal-body">
          <p class="confirm-msg">{{ aviso.mensagem }}</p>
        </div>
        <div class="modal-footer">
          <button class="btn-salvar" @click="fecharAviso">OK</button>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.conta-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 16px;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.titulo { font-size: 14px; font-weight: bold; color: var(--cor-texto-forte); }

.btn-novo {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 6px 14px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}
.btn-novo:hover { opacity: 0.85; }

/* ── Cards ── */
.cards-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-content: flex-start;
}

.card {
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 8px;
  width: 180px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 12px 12px;
  gap: 6px;
  cursor: default;
  transition: border-color 0.2s, opacity 0.2s;
}
.card:hover { border-color: #4a9eff; }
.card.inativa { opacity: 0.55; }

.card-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background-color: var(--cor-selecao);
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-iniciais { font-size: 15px; font-weight: bold; color: #ffffff; }

.card-pessoa {
  font-size: 12px;
  font-weight: bold;
  color: var(--cor-texto-forte);
  text-align: center;
  height: 32px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  width: 100%;
}

.card-banco {
  font-size: 13px;
  font-weight: bold;
  color: var(--cor-texto-forte);
  text-align: center;
  height: 18px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  width: 100%;
}

.card-info {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  text-align: center;
  height: 16px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  width: 100%;
}

.card-tipo { font-size: 11px; color: var(--cor-texto-fraco); }

.card-status {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: bold;
}
.card-status.ativa       { background-color: #1a4a1a; color: #4caf50; }
.card-status.inativa-tag { background-color: #3a1a1a; color: #cc4444; }

.card-footer {
  margin-top: 4px;
  width: 100%;
  display: flex;
  justify-content: flex-end;
}

.btn-editar {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
}
.btn-editar:hover { color: #4a9eff; }

/* ── Modal ── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
}

.modal {
  background-color: #2a2a2a;
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
  width: 380px;
  display: flex;
  flex-direction: column;
}

.modal-confirm { width: 320px; }

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--cor-borda);
  font-size: 13px;
  font-weight: bold;
  color: var(--cor-texto-forte);
}

.btn-fechar {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 14px;
}

.modal-body {
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.confirm-msg { font-size: 13px; color: var(--cor-texto); line-height: 1.5; }

/* ── Info somente leitura ── */
.campo-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
}

.info-label { font-size: 11px; color: var(--cor-texto-fraco); }

.info-valor {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  font-weight: bold;
}
.info-valor.ativa       { background-color: #1a4a1a; color: #4caf50; }
.info-valor.inativa-tag { background-color: #3a1a1a; color: #cc4444; }

/* ── Campos ── */
.campo { display: flex; flex-direction: column; gap: 4px; }

.campo label { font-size: 11px; color: var(--cor-texto-fraco); }

.campo input,
.campo select {
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
  padding: 7px 10px;
  outline: none;
  transition: border-color 0.2s;
  appearance: none;
  -webkit-appearance: none;
}

.campo input:focus,
.campo select:focus { border-color: #4a9eff; }
.campo input.erro,
.campo select.erro  { border-color: #cc4444; }
.campo select:disabled { opacity: 0.5; cursor: not-allowed; }

.campo select option { background-color: #2a2a2a; color: var(--cor-texto-forte); }

.msg-erro { font-size: 10px; color: #cc4444; }

/* ── Footer ── */
.modal-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--cor-borda);
}

.btn-salvar {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

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

.btn-excluir {
  background-color: transparent;
  color: #cc4444;
  border: 1px solid #cc4444;
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  margin-right: auto;
}
</style>
