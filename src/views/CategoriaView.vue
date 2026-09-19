<script setup>
import { ref, reactive, onMounted } from 'vue'
import { AgGridVue } from 'ag-grid-vue3'
import { themeQuartz } from 'ag-grid-community'
import Database from '@tauri-apps/plugin-sql'
import FloatLabel from '../components/FloatLabel.vue'
import FloatSelect from '../components/FloatSelect.vue'
import AppToggle from '../components/AppToggle.vue'

// ── Tema ──────────────────────────────────────────────────────────────────
const tema = themeQuartz.withParams({
  backgroundColor:          '#1e1e1e',
  foregroundColor:          '#cccccc',
  borderColor:              '#333333',
  headerBackgroundColor:    '#2a2a2a',
  headerTextColor:          '#ffffff',
  rowHoverColor:            '#2d2d2d',
  selectedRowBackgroundColor: '#0d3a5c',
  oddRowBackgroundColor:    '#222222',
  fontFamily:               '"Lucida Sans", Geneva, Verdana, sans-serif',
  fontSize:                 12,
  cellHorizontalPaddingScale: 1,
  rowHeight:                34,
  headerHeight:             36,
})

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
    'SELECT * FROM categoria WHERE deletado_em IS NULL ORDER BY nome'
  )
}

onMounted(carregar)

// ── Grid ──────────────────────────────────────────────────────────────────
const colunas = ref([
  { field: 'nome',          headerName: 'Nome',          flex: 2, cellStyle: { display: 'flex', alignItems: 'center' } },
  { field: 'tipo',          headerName: 'Tipo',          flex: 1, cellStyle: { display: 'flex', alignItems: 'center' } },
  { field: 'disponivel_em', headerName: 'Disponível em', flex: 1, cellStyle: { display: 'flex', alignItems: 'center' } },
  {
    field: 'comprovante_esperado',
    headerName: 'Comprovante',
    flex: 1,
    cellStyle: { display: 'flex', alignItems: 'center' },
    valueFormatter: p => p.value ? 'Sim' : 'Não'
  },
  {
    headerName: '',
    width: 48,
    sortable: false,
    cellStyle: { display: 'flex', alignItems: 'center', justifyContent: 'center' },
    cellRenderer: () => {
      const btn = document.createElement('button')
      btn.innerHTML = '<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>'
      btn.className = 'btn-editar-linha'
      return btn
    },
    onCellClicked: (e) => abrirEdicao(e.data),
  },
])

const gridOptions = {
  theme: tema,
  rowHeight: 34,
  headerHeight: 36,
  rowSelection: 'single',
  onRowDoubleClicked: (e) => abrirEdicao(e.data),
}

// ── Modal ─────────────────────────────────────────────────────────────────
const modalAberto = ref(false)
const modoEdicao  = ref(false)

const form = reactive({
  id: null,
  nome: '',
  tipo: '',
  disponivel_em: '',
  comprovante_esperado: true,
})

const opTipo = [
  { value: 'entrada', label: 'Entrada' },
  { value: 'saida',   label: 'Saída'   },
  { value: 'neutro',  label: 'Neutro'  },
]

const opDisponivelEm = [
  { value: 'direto',   label: 'Direto'   },
  { value: 'indireto', label: 'Indireto' },
  { value: 'ambos',    label: 'Ambos'    },
]

function abrirNovo() {
  Object.assign(form, { id: null, nome: '', tipo: '', disponivel_em: '', comprovante_esperado: true })
  modoEdicao.value = false
  modalAberto.value = true
}

function abrirEdicao(reg) {
  Object.assign(form, {
    id: reg.id,
    nome: reg.nome,
    tipo: reg.tipo,
    disponivel_em: reg.disponivel_em,
    comprovante_esperado: !!reg.comprovante_esperado,
  })
  modoEdicao.value = true
  modalAberto.value = true
}

function fecharModal() {
  modalAberto.value = false
}

async function salvar() {
  const d = await conectar()
  if (modoEdicao.value) {
    await d.execute(
      `UPDATE categoria SET nome=?, tipo=?, disponivel_em=?, comprovante_esperado=? WHERE id=?`,
      [form.nome, form.tipo, form.disponivel_em, form.comprovante_esperado ? 1 : 0, form.id]
    )
  } else {
    await d.execute(
      `INSERT INTO categoria (nome, tipo, disponivel_em, comprovante_esperado) VALUES (?,?,?,?)`,
      [form.nome, form.tipo, form.disponivel_em, form.comprovante_esperado ? 1 : 0]
    )
  }
  fecharModal()
  await carregar()
}

async function excluir() {
  if (!form.id) return
  const d = await conectar()
  await d.execute(
    `UPDATE categoria SET deletado_em=datetime('now'), ativa=0 WHERE id=?`,
    [form.id]
  )
  fecharModal()
  await carregar()
}

function trapFoco(e) {
  if (!modalAberto.value) return
  const modal = document.querySelector('.modal')
  if (!modal) return
  const focaveis = modal.querySelectorAll('button, input, [tabindex="0"]')
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
  <div class="categoria-view" @keydown="trapFoco">

    <div class="toolbar">
      <span class="titulo">Categorias</span>
      <button class="btn-novo" @click="abrirNovo">+ Nova Categoria</button>
    </div>

    <AgGridVue
      :rowData="registros"
      :columnDefs="colunas"
      :gridOptions="gridOptions"
      style="flex: 1; width: 100%;"
    />

    <div v-if="modalAberto" class="modal-overlay" @click.self="fecharModal">
      <div class="modal" role="dialog" aria-modal="true">
        <div class="modal-header">
          <span>{{ modoEdicao ? 'Editar Categoria' : 'Nova Categoria' }}</span>
          <button class="btn-fechar" @click="fecharModal">✕</button>
        </div>

        <div class="modal-body">
          <FloatLabel label="Nome" v-model="form.nome" required />
          <FloatSelect label="Tipo" v-model="form.tipo" :options="opTipo" />
          <FloatSelect label="Disponível em" v-model="form.disponivel_em" :options="opDisponivelEm" />
          <AppToggle label="Comprovante esperado" v-model="form.comprovante_esperado" />
        </div>

        <div class="modal-footer">
          <button v-if="modoEdicao" class="btn-excluir" @click="excluir">Excluir</button>
          <button class="btn-cancelar" @click="fecharModal">Cancelar</button>
          <button class="btn-salvar" @click="salvar">Salvar</button>
        </div>
      </div>
    </div>

  </div>
</template>

<style>
.btn-editar-linha {
  background: none;
  border: none;
  color: #888888;
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}
.btn-editar-linha:hover { color: #4a9eff; }
</style>

<style scoped>
.categoria-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 12px;
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
  gap: 16px;
}

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
