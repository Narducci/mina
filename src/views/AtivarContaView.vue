<script setup>
import { ref, computed, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'

// ── Banco ─────────────────────────────────────────────────────────────────
let db
async function conectar() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

// ── Dados ─────────────────────────────────────────────────────────────────
const contas    = ref([])
const selecionada = ref(null)

async function carregar() {
  const d = await conectar()
  contas.value = await d.select(`
    SELECT c.*, p.nome as pessoa_nome
    FROM conta c
    JOIN pessoa p ON p.id = c.pessoa_id
    WHERE c.deletado_em IS NULL
    ORDER BY c.banco
  `)
  // Pré-selecionar a conta já ativa, se houver
  const ativa = contas.value.find(c => c.ativa)
  selecionada.value = ativa ? ativa.id : null
}

onMounted(carregar)

// ── Ativar ────────────────────────────────────────────────────────────────
const podeAtivar = computed(() => {
  if (!selecionada.value) return false
  const atual = contas.value.find(c => c.id === selecionada.value)
  return atual && !atual.ativa
})

const aviso = ref({ visivel: false, mensagem: '', resolver: null })

function abrirAviso(mensagem) {
  return new Promise((resolve) => {
    aviso.value = { visivel: true, mensagem, resolver: resolve }
  })
}

function fecharAviso() {
  aviso.value.visivel = false
  aviso.value.resolver()
}

async function ativar() {
  if (!selecionada.value) return
  try {
    const d = await conectar()
    // Desativar todas as contas do curatelado
    await d.execute(
      `UPDATE conta SET ativa=0 WHERE deletado_em IS NULL`
    )
    // Ativar a selecionada
    await d.execute(
      `UPDATE conta SET ativa=1 WHERE id=?`,
      [selecionada.value]
    )
    await carregar()
    await abrirAviso('Conta ativada com sucesso.')
  } catch (e) {
    await abrirAviso('Erro ao ativar a conta. Tente novamente.')
  }
}
</script>

<template>
  <div class="ativar-conta-view">

    <div class="toolbar">
      <span class="titulo">Ativar Conta</span>
    </div>

    <div class="lista">
      <div
        v-for="conta in contas"
        :key="conta.id"
        class="item"
        :class="{
          selecionado: selecionada === conta.id,
          ativa: conta.ativa
        }"
        @click="selecionada = conta.id"
      >
        <div class="radio">
          <div class="radio-inner" v-if="selecionada === conta.id" />
        </div>
        <div class="item-info">
          <span class="item-banco">{{ conta.banco }}</span>
          <span class="item-detalhe">Ag. {{ conta.agencia }} / {{ conta.numero }} — {{ conta.tipo === 'corrente' ? 'Corrente' : 'Poupança' }}</span>
        </div>
        <div v-if="conta.ativa" class="badge-ativa">Ativa</div>
      </div>
    </div>

    <div class="rodape">
      <button
        class="btn-ativar"
        :disabled="!podeAtivar"
        @click="ativar"
      >
        Ativar
      </button>
    </div>

    <!-- Aviso -->
    <div v-if="aviso.visivel" class="modal-overlay">
      <div class="modal-aviso" role="alertdialog">
        <div class="aviso-body">
          <p>{{ aviso.mensagem }}</p>
        </div>
        <div class="aviso-footer">
          <button class="btn-ativar" @click="fecharAviso">OK</button>
        </div>
      </div>
    </div>

  </div>
</template>

<style scoped>
.ativar-conta-view {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 16px;
  gap: 16px;
}

.toolbar {
  display: flex;
  align-items: center;
}

.titulo {
  font-size: 14px;
  font-weight: bold;
  color: var(--cor-texto-forte);
}

/* ── Lista ── */
.lista {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 560px;
}

.item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 14px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
  cursor: pointer;
  transition: border-color 0.15s;
}

.item:hover { border-color: #4a9eff; }

.item.selecionado { border-color: #4a9eff; }

.item.ativa { opacity: 1; }

/* Radio visual */
.radio {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid var(--cor-borda);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: border-color 0.15s;
}

.item.selecionado .radio { border-color: #4a9eff; }

.radio-inner {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background-color: #4a9eff;
}

/* Info */
.item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.item-banco {
  font-size: 13px;
  font-weight: bold;
  color: var(--cor-texto-forte);
}

.item-detalhe {
  font-size: 11px;
  color: var(--cor-texto-fraco);
}

.badge-ativa {
  font-size: 10px;
  padding: 2px 8px;
  border-radius: 10px;
  background-color: #1a4a1a;
  color: #4caf50;
  font-weight: bold;
  flex-shrink: 0;
}

/* ── Rodapé ── */
.rodape {
  display: flex;
  max-width: 560px;
  justify-content: flex-end;
}

.btn-ativar {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 6px 20px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
  transition: opacity 0.2s;
}

.btn-ativar:hover:not(:disabled) { opacity: 0.85; }
.btn-ativar:disabled { opacity: 0.35; cursor: not-allowed; }

/* ── Aviso ── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
}

.modal-aviso {
  background-color: #2a2a2a;
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
  width: 300px;
  display: flex;
  flex-direction: column;
}

.aviso-body {
  padding: 24px 16px;
  font-size: 13px;
  color: var(--cor-texto);
  line-height: 1.5;
}

.aviso-footer {
  display: flex;
  justify-content: flex-end;
  padding: 12px 16px;
  border-top: 1px solid var(--cor-borda);
}
</style>
