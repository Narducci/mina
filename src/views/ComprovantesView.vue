<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useToolbarStore } from "../stores/toolbar.js";
import {
  PanelRightClose,
  PanelRightOpen,
  Search,
  Pencil,
  Archive,
  Trash2,
} from "@lucide/vue";
import Database from "@tauri-apps/plugin-sql";

const toolbar = useToolbarStore();
const sidePanelAberto = ref(false);
const comprovantes = ref([]);
const termoBusca = ref("");
const linhaSelecionadaId = ref(null);

const abaAtiva = ref("identificar");
const form = ref({
  nome: "",
  descricao: "",
  numero_documento: "",
  data_documento: "",
  status: "inbox",
});
const salvando = ref(false);

let db = null;
async function getDb() {
  if (!db) db = await Database.load("sqlite:mina.db");
  return db;
}

async function carregarComprovantes() {
  try {
    const banco = await getDb();
    const filtroSql =
      toolbar.filtro === "todos"
        ? ""
        : `AND c.status = '${toolbar.filtro === "inbox" ? "inbox" : "disponivel"}'`;

    comprovantes.value = await banco.select(`
      SELECT c.id, c.nome, c.descricao, c.numero_documento,
             c.data_documento, c.status,
             COUNT(cc.categoria_id) as total_categorias
      FROM comprovante c
      LEFT JOIN comprovante_categoria cc ON cc.comprovante_id = c.id
      WHERE c.deletado_em IS NULL ${filtroSql}
      GROUP BY c.id
      ORDER BY c.data_documento DESC
    `);
  } catch (err) {
    console.error("Erro ao carregar comprovantes:", err);
  }
}

async function carregarDetalhe(id) {
  if (!id) return;
  const banco = await getDb();
  const rows = await banco.select(
    "SELECT nome, descricao, numero_documento, data_documento, status FROM comprovante WHERE id = $1",
    [id],
  );
  if (rows[0]) Object.assign(form.value, rows[0]);
}

async function salvarIdentificacao() {
  if (!linhaSelecionadaId.value) return;
  salvando.value = true;
  try {
    const banco = await getDb();
    await banco.execute(
      `UPDATE comprovante
         SET nome = $1, descricao = $2, numero_documento = $3,
             data_documento = $4, status = $5
       WHERE id = $6`,
      [
        form.value.nome,
        form.value.descricao,
        form.value.numero_documento,
        form.value.data_documento,
        form.value.status,
        linhaSelecionadaId.value,
      ],
    );
    await carregarComprovantes();
  } finally {
    salvando.value = false;
  }
}

const compovantesFiltrados = computed(() => {
  const termo = termoBusca.value.trim().toLowerCase();
  if (!termo) return comprovantes.value;
  return comprovantes.value.filter(
    (c) =>
      (c.nome ?? "").toLowerCase().includes(termo) ||
      (c.descricao ?? "").toLowerCase().includes(termo) ||
      (c.numero_documento ?? "").toLowerCase().includes(termo),
  );
});

function formatarData(iso) {
  if (!iso) return "";
  const [a, m, d] = iso.split("-");
  return `${d}/${m}/${a}`;
}

function selecionarLinha(c) {
  linhaSelecionadaId.value = c.id;
  carregarDetalhe(c.id);
}

function handleTeclado(e) {
  const tag = document.activeElement?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA") return;

  const lista = compovantesFiltrados.value;
  if (!lista.length) return;

  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    const idx = lista.findIndex((c) => c.id === linhaSelecionadaId.value);
    let novo;
    if (e.key === "ArrowDown") {
      if (idx >= lista.length - 1) return;
      novo = lista[idx + 1];
    } else {
      if (idx <= 0) return;
      novo = lista[idx - 1];
    }
    linhaSelecionadaId.value = novo.id;
    carregarDetalhe(novo.id);
    const el = document.querySelector(`[data-id="${novo.id}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }
}

function alternarSidePanel() {
  sidePanelAberto.value = !sidePanelAberto.value;
}

// Habilita/desabilita botão Exibir conforme seleção
watch(linhaSelecionadaId, (id) => {
  toolbar.exibirAtivo = id !== null;
  if (id === null) toolbar.exibirPdf = false;
});

watch(
  () => toolbar.filtro,
  () => {
    linhaSelecionadaId.value = null;
    carregarComprovantes();
  },
);

onMounted(async () => {
  toolbar.ativarComprovantes({
    importar: () => console.log("importar"),
    excluir: () => console.log("excluir"),
    exibir: (ativo) => console.log("exibir pdf:", ativo),
  });
  // Exibir começa desabilitado até selecionar uma linha
  toolbar.exibirAtivo = false;
  await carregarComprovantes();
  document.addEventListener("keydown", handleTeclado);
});

onUnmounted(() => {
  toolbar.desativar();
  document.removeEventListener("keydown", handleTeclado);
});
</script>

<template>
  <div class="main-panel">
    <div class="shell">
      <div class="area-central">
        <!-- Conteúdo principal -->
        <div class="conteudo-principal">
          <!-- Painel superior -->
          <div class="painel-superior">
            <div class="busca-wrapper">
              <Search :size="14" class="busca-icone" />
              <input
                class="busca"
                type="text"
                placeholder="Buscar..."
                v-model="termoBusca"
              />
              <button
                v-if="termoBusca"
                class="busca-limpar"
                @click="termoBusca = ''"
              >
                ✕
              </button>
            </div>
          </div>

          <!-- Cabeçalho datagrid -->
          <div class="datagrid-header">
            <span class="col-nome">Nome e Descrição</span>
            <span class="col-cat">Categorias</span>
            <span class="col-data">Data</span>
            <span class="col-status">Status</span>
            <span class="col-acoes">Ações</span>
          </div>

          <!-- Linhas -->
          <div class="datagrid-body">
            <div
              v-for="c in compovantesFiltrados"
              :key="c.id"
              :data-id="c.id"
              class="linha"
              :class="{ selecionada: c.id === linhaSelecionadaId }"
              @click="selecionarLinha(c)"
            >
              <div class="col-nome">
                <span class="nome-arquivo">{{
                  c.numero_documento
                    ? `${c.numero_documento}-${c.nome}`
                    : c.nome
                }}</span>
                <span class="descricao">{{ c.descricao }}</span>
              </div>
              <div class="col-cat">{{ c.total_categorias }}</div>
              <div class="col-data">{{ formatarData(c.data_documento) }}</div>
              <div class="col-status">
                <span class="badge" :class="c.status">
                  {{ c.status === "inbox" ? "Inbox" : "Disponível" }}
                </span>
              </div>
              <div class="col-acoes">
                <button class="btn-acao"><Pencil :size="13" /></button>
                <button class="btn-acao" v-if="c.status === 'inbox'">
                  <Archive :size="13" />
                </button>
                <button class="btn-acao btn-excluir">
                  <Trash2 :size="13" />
                </button>
              </div>
            </div>

            <div v-if="compovantesFiltrados.length === 0" class="estado-vazio">
              Nenhum comprovante encontrado.
            </div>
          </div>

          <!-- Painel inferior -->
          <div class="painel-inferior">
            <span class="rodape-contador">
              {{ compovantesFiltrados.length }} documento{{
                compovantesFiltrados.length !== 1 ? "s" : ""
              }}
            </span>
          </div>
        </div>

        <!-- Side panel -->
        <div class="side-panel" :class="{ aberto: sidePanelAberto }">
          <div class="side-panel-cabecalho">
            <div v-if="sidePanelAberto" class="abas">
              <button
                class="aba"
                :class="{ ativa: abaAtiva === 'identificar' }"
                @click="abaAtiva = 'identificar'"
              >
                Identificar
              </button>
            </div>
            <button class="btn-toggle-panel" @click="alternarSidePanel">
              <PanelRightClose v-if="sidePanelAberto" :size="16" />
              <PanelRightOpen v-else :size="16" />
            </button>
          </div>

          <div v-if="sidePanelAberto" class="side-panel-corpo">
            <!-- Aba: Identificar -->
            <template v-if="abaAtiva === 'identificar'">
              <div v-if="!linhaSelecionadaId" class="painel-vazio">
                Selecione um comprovante na lista.
              </div>
              <div v-else class="form-identificar">
                <div class="campo">
                  <label>Nome do arquivo</label>
                  <input v-model="form.nome" type="text" />
                </div>

                <div class="campo">
                  <label>Número do documento</label>
                  <input
                    v-model="form.numero_documento"
                    type="text"
                    placeholder="Ex: NF-001"
                  />
                </div>

                <div class="campo">
                  <label>Descrição</label>
                  <textarea
                    v-model="form.descricao"
                    rows="3"
                    placeholder="Descrição opcional…"
                  />
                </div>

                <div class="campo">
                  <label>Data do documento</label>
                  <input v-model="form.data_documento" type="date" />
                </div>

                <div class="campo">
                  <label>Status</label>
                  <select v-model="form.status">
                    <option value="inbox">Inbox</option>
                    <option value="disponivel">Disponível</option>
                  </select>
                </div>

                <button
                  class="btn-salvar"
                  :disabled="salvando"
                  @click="salvarIdentificacao"
                >
                  {{ salvando ? "Salvando…" : "Salvar" }}
                </button>
              </div>
            </template>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.main-panel {
  height: 100%;
  padding: 12px;
  box-sizing: border-box;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background-color: var(--cor-menu-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
}

.area-central {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.conteudo-principal {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* Painel superior */
.painel-superior {
  flex-shrink: 0;
  border-bottom: 1px solid var(--cor-borda);
  padding: 7px 12px;
}

.busca-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: #111111;
  border: 1px solid var(--cor-borda);
  border-radius: 20px;
  padding: 5px 16px;
  max-width: 520px;
}

.busca-icone {
  color: var(--cor-texto-fraco);
  flex-shrink: 0;
}

.busca {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
}

.busca::placeholder {
  color: var(--cor-texto-fraco);
}

.busca-limpar {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 12px;
  padding: 0;
}

.busca-limpar:hover {
  color: #cc4444;
}

/* Datagrid */
.datagrid-header,
.linha {
  display: grid;
  grid-template-columns: 1fr 90px 100px 110px 90px;
  align-items: center;
  padding: 0 10px;
}

.datagrid-header {
  padding: 6px 10px;
  border-bottom: 1px solid var(--cor-borda);
  font-size: 10px;
  font-weight: bold;
  color: var(--cor-texto-fraco);
  letter-spacing: 0.5px;
  text-transform: uppercase;
  flex-shrink: 0;
}

.datagrid-body {
  flex: 1;
  overflow-y: auto;
}

.linha {
  min-height: 44px;
  border-bottom: 1px solid #2a2a2a;
  cursor: pointer;
  transition: background-color 0.1s;
}

.linha:hover {
  background-color: var(--cor-menu-hover);
}

.linha.selecionada {
  background-color: #1a3a5a;
}

.linha.selecionada:hover {
  background-color: #1e4268;
}

.nome-arquivo {
  display: block;
  font-size: 13px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.descricao {
  display: block;
  font-size: 11px;
  color: var(--cor-texto-fraco);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.col-cat,
.col-data,
.col-status,
.col-acoes {
  font-size: 12px;
  color: var(--cor-texto);
}

.col-acoes {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
}

.btn-acao {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  opacity: 0.6;
}

.btn-acao:hover {
  opacity: 1;
}

.btn-excluir:hover {
  color: #cc4444;
}

.badge {
  font-size: 10px;
  font-weight: bold;
  padding: 3px 10px;
  border-radius: 10px;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.badge.inbox {
  background-color: #2a2a1a;
  color: #ccaa44;
}

.badge.disponivel {
  background-color: #1a3a5a;
  color: #4a9eff;
}

/* Painel inferior */
.painel-inferior {
  flex-shrink: 0;
  height: 32px;
  border-top: 1px solid var(--cor-borda);
  display: flex;
  align-items: center;
  padding: 0 12px;
}

.rodape-contador {
  font-size: 12px;
  color: var(--cor-texto-fraco);
}

/* Side panel */
.side-panel {
  flex-shrink: 0;
  width: 36px;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--cor-borda);
  transition: width 0.2s ease;
  overflow: hidden;
}

.side-panel.aberto {
  width: 400px;
}

.side-panel-cabecalho {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 8px;
  border-bottom: 1px solid var(--cor-borda);
  flex-shrink: 0;
  min-height: 34px;
  white-space: nowrap;
}

.btn-toggle-panel {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.btn-toggle-panel:hover {
  color: var(--cor-texto-forte);
}

.side-panel-corpo {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

/* Estado vazio */
.estado-vazio {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--cor-texto-fraco);
  font-size: 13px;
}

/* Abas do side panel */
.abas {
  display: flex;
  gap: 2px;
  flex: 1;
}

.aba {
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  padding: 4px 10px 3px;
  font-family: inherit;
  white-space: nowrap;
}

.aba.ativa {
  color: var(--cor-texto-forte);
  border-bottom-color: #4a9eff;
}

/* Formulário Identificar */
.painel-vazio {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--cor-texto-fraco);
  font-size: 12px;
  text-align: center;
}

.form-identificar {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.campo {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campo label {
  font-size: 10px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--cor-texto-fraco);
}

.campo input,
.campo textarea,
.campo select {
  background-color: #111;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-family: inherit;
  font-size: 13px;
  padding: 6px 8px;
  outline: none;
  resize: vertical;
}

.campo input:focus,
.campo textarea:focus,
.campo select:focus {
  border-color: #4a9eff;
}

.btn-salvar {
  margin-top: 4px;
  background-color: #1a4a7a;
  border: none;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 8px;
  transition: background-color 0.15s;
}

.btn-salvar:hover:not(:disabled) {
  background-color: #1e5a94;
}

.btn-salvar:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
