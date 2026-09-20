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
const registros = ref([])

async function carregar() {
  const d = await conectar()
  registros.value = await d.select(
    'SELECT * FROM pessoa WHERE deletado_em IS NULL ORDER BY nome'
  )
}

onMounted(carregar)

// ── Opções ────────────────────────────────────────────────────────────────
const opTipo = [
  { value: 'curador',      label: 'Curador'      },
  { value: 'curatelado',   label: 'Curatelado'   },
  { value: 'contribuinte', label: 'Contribuinte' },
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

// ── Modal ─────────────────────────────────────────────────────────────────
const modalAberto  = ref(false)
const modoEdicao   = ref(false)
const erros        = reactive({})

const formInicial  = () => ({ id: null, nome: '', cpf: '', tipo: '', data_nascimento: '' })
const form         = reactive(formInicial())
const formOriginal = reactive(formInicial())

const dirty = computed(() =>
  form.nome            !== formOriginal.nome ||
  form.cpf             !== formOriginal.cpf  ||
  form.tipo            !== formOriginal.tipo ||
  form.data_nascimento !== formOriginal.data_nascimento
)

function abrirNovo() {
  Object.assign(form, formInicial())
  Object.assign(formOriginal, formInicial())
  Object.assign(erros, { nome: '', cpf: '', data_nascimento: '' })
  modoEdicao.value = false
  modalAberto.value = true
}

function abrirEdicao(reg) {
  // data armazenada em AAAA-MM-DD, exibir como DD/MM/AAAA
  const dataExib = reg.data_nascimento
    ? reg.data_nascimento.split('-').reverse().join('/')
    : ''
  const dados = {
    id: reg.id,
    nome: reg.nome,
    cpf: reg.cpf || '',
    tipo: reg.tipo,
    data_nascimento: dataExib,
  }
  Object.assign(form, dados)
  Object.assign(formOriginal, dados)
  Object.assign(erros, { nome: '', cpf: '', data_nascimento: '' })
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

// ── CPF ───────────────────────────────────────────────────────────────────
function mascaraCpf(e) {
  let v = e.target.value.replace(/\D/g, '').slice(0, 11)
  v = v
    .replace(/(\d{3})(\d)/, '$1.$2')
    .replace(/(\d{3})\.(\d{3})(\d)/, '$1.$2.$3')
    .replace(/(\d{3})\.(\d{3})\.(\d{3})(\d)/, '$1.$2.$3-$4')
  e.target.value = v
  form.cpf = v
  erros.cpf = ''
}

function validarCpf(cpf) {
  const n = cpf.replace(/\D/g, '')
  if (n.length === 0) return true
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

function blurCpf() {
  if (form.cpf && !validarCpf(form.cpf)) {
    erros.cpf = 'CPF inválido'
  } else {
    erros.cpf = ''
  }
}

// ── Data ──────────────────────────────────────────────────────────────────
function mascaraData(e) {
  // Preservar posição do cursor para não atrapalhar digitação
  const input = e.target
  const anterior = input.value
  let digits = anterior.replace(/\D/g, '').slice(0, 8)

  let v = digits
  if (digits.length > 2) v = digits.slice(0, 2) + '/' + digits.slice(2)
  if (digits.length > 4) v = digits.slice(0, 2) + '/' + digits.slice(2, 4) + '/' + digits.slice(4)

  input.value = v
  form.data_nascimento = v
  erros.data_nascimento = ''
}

function validarData(data) {
  if (!data) return true
  if (data.length !== 10) return false
  const [diaStr, mesStr, anoStr] = data.split('/')
  const dia = parseInt(diaStr, 10)
  const mes = parseInt(mesStr, 10)
  const ano = parseInt(anoStr, 10)
  if (isNaN(dia) || isNaN(mes) || isNaN(ano)) return false
  if (mes < 1 || mes > 12) return false
  if (ano < 1900 || ano > 2100) return false
  const diasNoMes = new Date(ano, mes, 0).getDate()
  return dia >= 1 && dia <= diasNoMes
}

// Converte DD/MM/AAAA → AAAA-MM-DD para salvar no banco
function dataParaIso(data) {
  if (!data || data.length !== 10) return null
  const [dia, mes, ano] = data.split('/')
  return `${ano}-${mes}-${dia}`
}

// ── Salvar ────────────────────────────────────────────────────────────────
async function salvar() {
  // Limpar erros anteriores
  Object.assign(erros, { nome: '', cpf: '', data_nascimento: '' })
  let valido = true

  if (!form.nome.trim()) {
    erros.nome = 'Nome é obrigatório'
    valido = false
  }

  if (form.cpf && !validarCpf(form.cpf)) {
    erros.cpf = 'CPF inválido'
    valido = false
  }

  if (form.data_nascimento && !validarData(form.data_nascimento)) {
    erros.data_nascimento = 'Data inválida — use DD/MM/AAAA'
    valido = false
  }

  if (!valido) return

  const d = await conectar()

  if (form.cpf) {
    const existe = await d.select(
      `SELECT id FROM pessoa WHERE cpf = ? AND deletado_em IS NULL AND id != ?`,
      [form.cpf, form.id || 0]
    )
    if (existe.length > 0) {
      erros.cpf = 'CPF já cadastrado'
      return
    }
  }

  const dataIso = dataParaIso(form.data_nascimento)

  try {
    if (modoEdicao.value) {
      await d.execute(
        `UPDATE pessoa SET nome=?, cpf=?, tipo=?, data_nascimento=? WHERE id=?`,
        [form.nome.trim(), form.cpf || null, form.tipo, dataIso, form.id]
      )
    } else {
      await d.execute(
        `INSERT INTO pessoa (nome, cpf, tipo, data_nascimento) VALUES (?,?,?,?)`,
        [form.nome.trim(), form.cpf || null, form.tipo, dataIso]
      )
    }
    modalAberto.value = false
    await carregar()
  } catch (e) {
    erros.cpf = 'Erro ao salvar. Verifique os dados e tente novamente.'
  }
}

// ── Excluir ───────────────────────────────────────────────────────────────
async function excluir() {
  const ok = await abrirConfirmacao(`Excluir "${form.nome}"? Esta ação não pode ser desfeita.`)
  if (!ok) return
  try {
    const d = await conectar()
    await d.execute(
      `UPDATE pessoa SET cpf=NULL, deletado_em=datetime('now') WHERE id=?`,
      [form.id]
    )
    modalAberto.value = false
    await carregar()
  } catch (e) {
    erros.nome = 'Erro ao excluir. Tente novamente.'
  }
}

// ── Avatar ────────────────────────────────────────────────────────────────
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
  <div class="pessoa-view" @keydown="trapFoco">

    <div class="toolbar">
      <span class="titulo">Pessoas</span>
      <button class="btn-novo" @click="abrirNovo">+ Nova Pessoa</button>
    </div>

    <div class="cards-grid">
      <div v-for="reg in registros" :key="reg.id" class="card" @dblclick="abrirEdicao(reg)">
        <div class="card-avatar">
          <span class="avatar-iniciais">{{ iniciais(reg.nome) }}</span>
        </div>
        <div class="card-nome">{{ reg.nome }}</div>
        <div class="card-tipo">{{ reg.tipo }}</div>
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
          <span>{{ modoEdicao ? 'Editar Pessoa' : 'Nova Pessoa' }}</span>
          <button class="btn-fechar" @click="fecharModal">✕</button>
        </div>

        <div class="modal-body">

          <!-- Nome -->
          <div class="campo">
            <label>Nome</label>
            <input
              type="text"
              v-model="form.nome"
              @input="erros.nome = ''"
              :class="{ erro: erros.nome }"
            />
            <span v-if="erros.nome" class="msg-erro">{{ erros.nome }}</span>
          </div>

          <!-- CPF -->
          <div class="campo">
            <label>CPF</label>
            <input
              type="text"
              inputmode="numeric"
              maxlength="14"
              placeholder="000.000.000-00"
              :value="form.cpf"
              @input="mascaraCpf"
              @blur="blurCpf"
              :class="{ erro: erros.cpf }"
            />
            <span v-if="erros.cpf" class="msg-erro">{{ erros.cpf }}</span>
          </div>

          <!-- Tipo -->
          <div class="campo">
            <label>Tipo</label>
            <select v-model="form.tipo">
              <option value="" disabled></option>
              <option v-for="op in opTipo" :key="op.value" :value="op.value">{{ op.label }}</option>
            </select>
          </div>

          <!-- Data de nascimento -->
          <div class="campo">
            <label>Data de nascimento</label>
            <input
              type="text"
              inputmode="numeric"
              maxlength="10"
              placeholder="DD/MM/AAAA"
              :value="form.data_nascimento"
              @input="mascaraData"
              :class="{ erro: erros.data_nascimento }"
            />
            <span v-if="erros.data_nascimento" class="msg-erro">{{ erros.data_nascimento }}</span>
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

  </div>
</template>

<style scoped>
.pessoa-view {
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

.titulo {
  font-size: 14px;
  font-weight: bold;
  color: var(--cor-texto-forte);
}

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
  width: 160px;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 12px 12px;
  gap: 8px;
  cursor: default;
  transition: border-color 0.2s;
}
.card:hover { border-color: #4a9eff; }

.card-avatar {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background-color: var(--cor-selecao);
  display: flex;
  align-items: center;
  justify-content: center;
}

.avatar-iniciais {
  font-size: 18px;
  font-weight: bold;
  color: #ffffff;
  letter-spacing: 1px;
}

.card-nome {
  font-size: 13px;
  font-weight: bold;
  color: var(--cor-texto-forte);
  text-align: center;
  word-break: break-word;
  height: 36px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.card-tipo {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  text-transform: capitalize;
}

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

.modal-confirm {
  width: 320px;
}

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

.confirm-msg {
  font-size: 13px;
  color: var(--cor-texto);
  line-height: 1.5;
}

/* ── Campos ── */
.campo {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campo label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
}

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

.campo input::placeholder {
  color: var(--cor-texto-fraco);
  opacity: 0.5;
}

.campo input:focus,
.campo select:focus {
  border-color: #4a9eff;
}

.campo input.erro {
  border-color: #cc4444;
}

.campo select option {
  background-color: #2a2a2a;
  color: var(--cor-texto-forte);
}

.msg-erro {
  font-size: 10px;
  color: #cc4444;
}

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
