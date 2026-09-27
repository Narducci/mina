<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useToolbarStore } from "../stores/toolbar.js";
import { PanelRightClose, PanelRightOpen, Search, Pencil, Archive, Trash2, Bell } from "@lucide/vue";
import Database from "@tauri-apps/plugin-sql";
import AppSelect from "../components/AppSelect.vue";
import { convertFileSrc } from "@tauri-apps/api/core";

const toolbar = useToolbarStore();
const sidePanelAberto = ref(false);
const comprovantes = ref([]);
const termoBusca = ref("");
const linhaSelecionadaId = ref(null);

const abaAtiva = ref("identificar");
const form = ref({
  nome: "",
  nome_curto: "",
  descricao: "",
  numero_documento: "",
  sem_numero: false,
  data_documento: "",
});
const salvando = ref(false);

// ── Classificar ──────────────────────────────────────────────
const categorias = ref([]);
const categoriasSelecionadas = ref(new Set());
const filtroTipo = ref("todos");
const termoBuscaCategoria = ref("");

const tipoOpcoes = [
  { value: "todos",   label: "Todos"  },
  { value: "entrada", label: "Ent."   },
  { value: "saida",   label: "Saída"  },
  { value: "neutro",  label: "Neutro" },
];

const categoriasFiltradas = computed(() => {
  let lista = categorias.value;
  if (filtroTipo.value !== "todos") {
    lista = lista.filter((c) => c.tipo === filtroTipo.value);
  }
  const termo = termoBuscaCategoria.value.trim().toLowerCase();
  if (termo) {
    lista = lista.filter((c) => c.nome.toLowerCase().includes(termo));
  }
  return lista;
});
// ─────────────────────────────────────────────────────────────

const totalInbox = computed(() =>
  comprovantes.value.filter((c) => c.status === "inbox").length
);

let db = null;
async function getDb() {
  if (!db) db = await Database.load("sqlite:mina.db");
  return db;
}

async function carregarComprovantes() {
  try {
    const banco = await getDb();
    const filtroSql = toolbar.filtro === "todos"
      ? ""
      : `AND c.status = '${toolbar.filtro === "inbox" ? "inbox" : "disponivel"}'`;

    comprovantes.value = await banco.select(`
      SELECT c.id, c.nome, c.nome_curto, c.descricao, c.numero_documento,
             c.data_documento, c.status, c.caminho_arquivo,
             COUNT(cc.categoria_id) as total_categorias
      FROM comprovante c
      LEFT JOIN comprovante_categoria cc ON cc.comprovante_id = c.id
      WHERE c.deletado_em IS NULL ${filtroSql}
      GROUP BY c.id
      ORDER BY c.data_documento DESC NULLS LAST, c.id DESC
    `);
  } catch (err) {
    console.error("Erro ao carregar comprovantes:", err);
  }
}

async function carregarCategorias() {
  try {
    const banco = await getDb();
    categorias.value = await banco.select(
      `SELECT id, nome, tipo FROM categoria
       WHERE ativa = 1 AND deletado_em IS NULL
         AND nome NOT IN ('Aporte Período', 'Aporte Realizado')
       ORDER BY tipo, nome`
    );
  } catch (err) {
    console.error("Erro ao carregar categorias:", err);
  }
}

async function carregarCategoriasComprovante(id) {
  if (!id) return;
  try {
    const banco = await getDb();
    const rows = await banco.select(
      `SELECT categoria_id FROM comprovante_categoria WHERE comprovante_id = $1`,
      [id]
    );
    categoriasSelecionadas.value = new Set(rows.map((r) => r.categoria_id));
  } catch (err) {
    console.error("Erro ao carregar categorias do comprovante:", err);
  }
}

async function toggleCategoria(catId) {
  if (!linhaSelecionadaId.value) return;
  try {
    const banco = await getDb();
    if (categoriasSelecionadas.value.has(catId)) {
      await banco.execute(
        `DELETE FROM comprovante_categoria WHERE comprovante_id = $1 AND categoria_id = $2`,
        [linhaSelecionadaId.value, catId]
      );
      const next = new Set(categoriasSelecionadas.value);
      next.delete(catId);
      categoriasSelecionadas.value = next;
    } else {
      await banco.execute(
        `INSERT OR IGNORE INTO comprovante_categoria (comprovante_id, categoria_id) VALUES ($1, $2)`,
        [linhaSelecionadaId.value, catId]
      );
      const next = new Set(categoriasSelecionadas.value);
      next.add(catId);
      categoriasSelecionadas.value = next;
    }
    await carregarComprovantes();
  } catch (err) {
    console.error("Erro ao alternar categoria:", err);
  }
}

function isoParaBr(iso) {
  if (!iso) return "";
  const [a, m, d] = iso.split("-");
  if (!d) return iso;
  return `${d}/${m}/${a}`;
}

function brParaIso(br) {
  if (!br) return "";
  const [d, m, a] = br.split("/");
  if (!a) return br;
  return `${a}-${m.padStart(2, "0")}-${d.padStart(2, "0")}`;
}

async function carregarDetalhe(id) {
  if (!id) return;
  const banco = await getDb();
  const rows = await banco.select(
    `SELECT nome, nome_curto, descricao, numero_documento, data_documento
     FROM comprovante WHERE id = $1`,
    [id]
  );
  if (rows[0]) {
    const r = rows[0];
    Object.assign(form.value, {
      nome: r.nome ?? "",
      nome_curto: r.nome_curto ?? "",
      descricao: r.descricao ?? "",
      numero_documento: r.numero_documento ?? "",
      sem_numero: false,
      data_documento: isoParaBr(r.data_documento),
    });
  }
}

// Preview do nome normalizado em tempo real
const nomeNormalizado = computed(() => {
  const ext = (form.value.nome ?? "").includes(".")
    ? form.value.nome.split(".").pop()
    : "";

  let num;
  if (form.value.sem_numero) {
    num = "S????????";
  } else {
    const n = (form.value.numero_documento ?? "").trim();
    num = n ? n.padStart(9, "0").slice(0, 9) : "_________";
  }

  const nomeC = (form.value.nome_curto ?? "").trim().replace(/\s+/g, "-") || "_______";
  return `${num}_${nomeC}${ext ? "." + ext : ""}`;
});

function onSemNumero() {
  if (form.value.sem_numero) {
    form.value.numero_documento = "";
  }
}

async function gerarNumeroSequencial(banco) {
  const rows = await banco.select("SELECT proximo FROM seq_documento WHERE id = 1");
  const proximo = rows[0]?.proximo ?? 1;
  await banco.execute("UPDATE seq_documento SET proximo = proximo + 1 WHERE id = 1");
  return "S" + String(proximo).padStart(8, "0");
}

async function salvarIdentificacao() {
  if (!linhaSelecionadaId.value) return;
  salvando.value = true;
  try {
    const banco = await getDb();

    let numero = form.value.numero_documento.trim();

    if (form.value.sem_numero) {
      numero = await gerarNumeroSequencial(banco);
      form.value.numero_documento = numero;
      form.value.sem_numero = false;
    } else if (numero) {
      numero = numero.padStart(9, "0").slice(0, 9);
      form.value.numero_documento = numero;
    }

    await banco.execute(
      `UPDATE comprovante
         SET nome_curto = $1, descricao = $2, numero_documento = $3,
             data_documento = $4
       WHERE id = $5`,
      [
        form.value.nome_curto || null,
        form.value.descricao || null,
        numero || null,
        brParaIso(form.value.data_documento) || null,
        linhaSelecionadaId.value,
      ]
    );
    await carregarComprovantes();
  } finally {
    salvando.value = false;
  }
}

async function arquivarLinha(id, event) {
  event.stopPropagation();
  const banco = await getDb();
  await banco.execute(
    "UPDATE comprovante SET status = 'disponivel' WHERE id = $1",
    [id]
  );
  await carregarComprovantes();
}

async function excluirLinha(id, event) {
  event.stopPropagation();
  const banco = await getDb();
  await banco.execute(
    "UPDATE comprovante SET deletado_em = datetime('now') WHERE id = $1",
    [id]
  );
  if (linhaSelecionadaId.value === id) linhaSelecionadaId.value = null;
  await carregarComprovantes();
}

const pdfSrc = computed(() => {
  if (!toolbar.exibirPdf) return null;
  const c = comprovantes.value.find((x) => x.id === linhaSelecionadaId.value);
  return c?.caminho_arquivo ? convertFileSrc(c.caminho_arquivo) : null;
});

function exibirArquivo() {
  toolbar.exibirPdf = !toolbar.exibirPdf;
}

const compovantesFiltrados = computed(() => {
  const termo = termoBusca.value.trim().toLowerCase();
  if (!termo) return comprovantes.value;
  return comprovantes.value.filter(
    (c) =>
      (c.nome ?? "").toLowerCase().includes(termo) ||
      (c.descricao ?? "").toLowerCase().includes(termo) ||
      (c.numero_documento ?? "").toLowerCase().includes(termo)
  );
});

function formatarData(iso) {
  if (!iso) return "";
  const [a, m, d] = iso.split("-");
  return `${d}/${m}/${a}`;
}

function onDataInput(e) {
  let v = e.target.value.replace(/\D/g, "");
  if (v.length > 2) v = v.slice(0, 2) + "/" + v.slice(2);
  if (v.length > 5) v = v.slice(0, 5) + "/" + v.slice(5);
  if (v.length > 10) v = v.slice(0, 10);
  form.value.data_documento = v;
}

function nomeExibicao(c) {
  const ext = c.nome?.includes(".") ? c.nome.split(".").pop() : "";
  const num = c.numero_documento || "";
  const nome = (c.nome_curto || "").replace(/\s+/g, "-");
  if (num && nome) return `${num}_${nome}${ext ? "." + ext : ""}`;
  if (num) return `${num}${ext ? "." + ext : ""}`;
  if (nome) return `${nome}${ext ? "." + ext : ""}`;
  return c.nome || "";
}

function identificarCompleto(c) {
  return !!(c.nome_curto && c.numero_documento && c.descricao && c.data_documento);
}

function disponibilizarCompleto(c) {
  return identificarCompleto(c) && c.total_categorias > 0;
}

function selecionarLinha(c) {
  linhaSelecionadaId.value = c.id;
  carregarDetalhe(c.id);
  carregarCategoriasComprovante(c.id);
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
    carregarCategoriasComprovante(novo.id);
    const el = document.querySelector(`[data-id="${novo.id}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }
}

function alternarSidePanel() {
  sidePanelAberto.value = !sidePanelAberto.value;
}

watch(linhaSelecionadaId, (id) => {
  toolbar.exibirAtivo = id !== null;
  if (id === null) {
    toolbar.exibirPdf = false;
    categoriasSelecionadas.value = new Set();
  }
});

watch(() => toolbar.filtro, () => {
  linhaSelecionadaId.value = null;
  carregarComprovantes();
});

onMounted(async () => {
  toolbar.ativarComprovantes({
    importar: () => console.log("importar"),
    excluir: () => linhaSelecionadaId.value && excluirLinha(linhaSelecionadaId.value, { stopPropagation: () => {} }),
    exibir: exibirArquivo,
  });
  toolbar.exibirAtivo = false;
  await carregarComprovantes();
  await carregarCategorias();
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
              >✕</button>
            </div>

            <div
              v-if="totalInbox > 0"
              class="sininho-wrapper"
              :title="`${totalInbox} documento${totalInbox !== 1 ? 's' : ''} no Inbox`"
            >
              <Bell :size="18" class="sininho-icone" />
              <span class="sininho-badge">{{ totalInbox > 99 ? '99+' : totalInbox }}</span>
            </div>
          </div>

          <!-- Cabeçalho datagrid -->
          <div class="datagrid-header">
            <span class="col-nome">Nome e Descrição</span>
            <span class="col-cat">Cat.</span>
            <span class="col-data">Data</span>
            <span class="col-status">Status</span>
            <span class="col-acoes">Ações</span>
          </div>

          <!-- Visualizador de PDF -->
          <iframe
            v-if="toolbar.exibirPdf && pdfSrc"
            :src="pdfSrc"
            class="pdf-viewer"
          />
          <div v-else-if="toolbar.exibirPdf" class="pdf-sem-arquivo">
            Nenhum arquivo associado a este comprovante.
          </div>

          <!-- Linhas -->
          <div v-else class="datagrid-body">
            <div
              v-for="c in compovantesFiltrados"
              :key="c.id"
              :data-id="c.id"
              class="linha"
              :class="{ selecionada: c.id === linhaSelecionadaId, 'em-exibicao': c.id === linhaSelecionadaId && toolbar.exibirPdf }"
              @click="selecionarLinha(c)"
            >
              <div class="col-nome">
                <span class="nome-arquivo">{{ nomeExibicao(c) }}</span>
                <span class="descricao">{{ c.descricao }}</span>
              </div>
              <div class="col-cat">{{ c.total_categorias }}</div>
              <div class="col-data">{{ formatarData(c.data_documento) }}</div>
              <div class="col-status">
                <span class="badge" :class="c.status">
                  {{ c.status === 'inbox' ? 'Inbox' : 'Disponível' }}
                </span>
              </div>
              <div class="col-acoes">
                <button class="btn-acao" title="Identificar" @click.stop="selecionarLinha(c); sidePanelAberto = true"><Pencil :size="13" /></button>
                <button
                  class="btn-acao"
                  v-if="c.status === 'inbox'"
                  title="Disponibilizar"
                  :disabled="!disponibilizarCompleto(c)"
                  :class="{ 'btn-bloqueado': !disponibilizarCompleto(c) }"
                  @click="arquivarLinha(c.id, $event)"
                ><Archive :size="13" /></button>
                <button class="btn-acao btn-excluir" title="Excluir" @click="excluirLinha(c.id, $event)"><Trash2 :size="13" /></button>
              </div>
            </div>

            <div v-if="compovantesFiltrados.length === 0" class="estado-vazio">
              Nenhum comprovante encontrado.
            </div>
          </div>

          <!-- Painel inferior -->
          <div class="painel-inferior">
            <span class="rodape-contador">
              {{ compovantesFiltrados.length }} documento{{ compovantesFiltrados.length !== 1 ? 's' : '' }}
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
              >Identificar</button>
              <button
                class="aba"
                :class="{ ativa: abaAtiva === 'classificar' }"
                @click="abaAtiva = 'classificar'"
              >Classificar</button>
            </div>
            <button class="btn-toggle-panel" @click="alternarSidePanel">
              <PanelRightClose v-if="sidePanelAberto" :size="16" />
              <PanelRightOpen v-else :size="16" />
            </button>
          </div>

          <div v-if="sidePanelAberto" class="side-panel-corpo">

            <!-- Aba Identificar -->
            <template v-if="abaAtiva === 'identificar'">
              <div v-if="!linhaSelecionadaId" class="painel-vazio">
                Selecione um comprovante na lista.
              </div>
              <div v-else class="form-identificar">

                <!-- Nome do arquivo (read-only) -->
                <div class="campo">
                  <label>Nome do arquivo</label>
                  <div class="campo-readonly">{{ form.nome || '—' }}</div>
                </div>

                <!-- Número do documento -->
                <div class="campo">
                  <label>Número do documento</label>
                  <input
                    v-model="form.numero_documento"
                    type="text"
                    :disabled="form.sem_numero"
                    placeholder="Ex: NF-001"
                    :class="{ desabilitado: form.sem_numero }"
                  />
                  <label class="checkbox-label">
                    <input
                      type="checkbox"
                      v-model="form.sem_numero"
                      @change="onSemNumero"
                    />
                    Documento sem número
                  </label>
                </div>

                <!-- Descrição -->
                <div class="campo">
                  <label>Descrição <span class="obrigatorio">*</span></label>
                  <textarea v-model="form.descricao" rows="3" placeholder="Descrição do documento…" />
                </div>

                <!-- Data da Transação -->
                <div class="campo">
                  <label>Data da Transação <span class="obrigatorio">*</span></label>
                  <input v-model="form.data_documento" type="text" placeholder="DD/MM/AAAA" @input="onDataInput" />
                </div>

                <!-- Nome curto -->
                <div class="campo">
                  <label>Nome curto <span class="obrigatorio">*</span></label>
                  <input v-model="form.nome_curto" type="text" placeholder="Ex: Aluguel Joao" />
                </div>

                <!-- Preview do nome normalizado -->
                <div class="campo">
                  <label>Nome normalizado</label>
                  <div class="campo-preview" :title="nomeNormalizado">{{ nomeNormalizado }}</div>
                </div>

                <button class="btn-salvar" :disabled="salvando" @click="salvarIdentificacao">
                  {{ salvando ? "Salvando…" : "Salvar" }}
                </button>

              </div>
            </template>

            <!-- Aba Classificar -->
            <template v-if="abaAtiva === 'classificar'">
              <div v-if="!linhaSelecionadaId" class="painel-vazio">
                Selecione um comprovante na lista.
              </div>
              <div v-else class="form-classificar">

                <!-- Busca -->
                <input
                  v-model="termoBuscaCategoria"
                  type="text"
                  class="busca-categoria"
                  placeholder="Buscar categoria..."
                />

                <!-- Filtro por tipo -->
                <div class="filtros-tipo">
                  <button
                    v-for="opt in tipoOpcoes"
                    :key="opt.value"
                    class="btn-tipo"
                    :class="{ ativo: filtroTipo === opt.value }"
                    @click="filtroTipo = opt.value"
                  >{{ opt.label }}</button>
                </div>

                <!-- Lista de categorias -->
                <div class="lista-categorias">
                  <label
                    v-for="cat in categoriasFiltradas"
                    :key="cat.id"
                    class="categoria-item"
                    :class="{ selecionada: categoriasSelecionadas.has(cat.id) }"
                  >
                    <input
                      type="checkbox"
                      :checked="categoriasSelecionadas.has(cat.id)"
                      @change="toggleCategoria(cat.id)"
                    />
                    <span class="cat-nome">{{ cat.nome }}</span>
                    <span class="cat-tipo" :class="cat.tipo">{{ cat.tipo }}</span>
                  </label>

                  <div v-if="categoriasFiltradas.length === 0" class="painel-vazio">
                    Nenhuma categoria encontrada.
                  </div>
                </div>

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
  display: flex;
  align-items: center;
  gap: 12px;
}

.busca-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: #111111;
  border: 1px solid var(--cor-borda);
  border-radius: 20px;
  padding: 5px 16px;
  flex: 1;
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

/* Sininho */
.sininho-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  cursor: default;
  margin-left: auto;
}

.sininho-icone {
  color: var(--cor-texto-fraco);
}

.sininho-badge {
  position: absolute;
  top: -6px;
  right: -7px;
  background-color: #cc2222;
  color: #fff;
  border-radius: 50%;
  font-size: 9px;
  font-weight: bold;
  min-width: 15px;
  height: 15px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 3px;
  line-height: 1;
}

/* Datagrid */
.datagrid-header,
.linha {
  display: grid;
  grid-template-columns: 1fr 60px 100px 110px 90px;
  align-items: center;
  padding: 0 16px 0 10px;
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

.pdf-viewer {
  flex: 1;
  border: none;
  background: #111;
}

.pdf-sem-arquivo {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--cor-texto-fraco);
  font-size: 13px;
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

.linha.em-exibicao {
  background-color: #0f2a40;
  border-left: 2px solid #4a9eff;
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

.btn-bloqueado {
  opacity: 0.2 !important;
  cursor: not-allowed !important;
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

/* Abas */
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

.obrigatorio {
  color: #cc4444;
  font-size: 11px;
}

.campo input,
.campo textarea {
  background-color: #111;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-family: inherit;
  font-size: 13px;
  padding: 6px 8px;
  outline: none;
}

.campo input.desabilitado {
  opacity: 0.4;
  cursor: not-allowed;
}

.campo textarea {
  resize: vertical;
}

.campo input:focus,
.campo textarea:focus {
  border-color: #4a9eff;
}

/* Campo read-only */
.campo-readonly {
  background-color: #0e0e0e;
  border: 1px solid #333;
  border-radius: 4px;
  color: var(--cor-texto-fraco);
  font-size: 12px;
  padding: 6px 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: monospace;
}

/* Preview nome normalizado */
.campo-preview {
  background-color: #0d1a2a;
  border: 1px solid #1e3a5f;
  border-radius: 4px;
  color: #4a9eff;
  font-size: 11px;
  padding: 6px 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: monospace;
}

/* Checkbox */
.checkbox-label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px !important;
  font-weight: normal !important;
  text-transform: none !important;
  letter-spacing: 0 !important;
  color: var(--cor-texto) !important;
  cursor: pointer;
  margin-top: 4px;
}

.checkbox-label input[type="checkbox"] {
  width: auto;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
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

/* ── Aba Classificar ───────────────────────────────────────── */
.form-classificar {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.busca-categoria {
  width: 100%;
  box-sizing: border-box;
  background-color: #111;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-forte);
  font-family: inherit;
  font-size: 12px;
  padding: 6px 8px;
  outline: none;
}

.busca-categoria:focus {
  border-color: #4a9eff;
}

.busca-categoria::placeholder {
  color: var(--cor-texto-fraco);
}

.filtros-tipo {
  display: flex;
  gap: 2px;
}

.btn-tipo {
  flex: 1;
  background: none;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  padding: 4px 0;
  transition: background-color 0.1s, color 0.1s;
}

.btn-tipo:hover {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}

.btn-tipo.ativo {
  background-color: #1a3a5a;
  border-color: #4a9eff;
  color: #4a9eff;
}

.lista-categorias {
  display: flex;
  flex-direction: column;
  gap: 1px;
  margin-top: 2px;
}

.categoria-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 4px;
  cursor: pointer;
  user-select: none;
  transition: background-color 0.1s;
}

.categoria-item:hover {
  background-color: var(--cor-menu-hover);
}

.categoria-item.selecionada {
  background-color: #0d1a2a;
}

.categoria-item input[type="checkbox"] {
  width: auto;
  margin: 0;
  padding: 0;
  border: none;
  background: none;
  cursor: pointer;
  flex-shrink: 0;
  accent-color: #4a9eff;
}

.cat-nome {
  flex: 1;
  font-size: 12px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cat-tipo {
  font-size: 9px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.4px;
  padding: 2px 6px;
  border-radius: 3px;
  flex-shrink: 0;
}

.cat-tipo.entrada {
  background-color: #1a3a1a;
  color: #44bb44;
}

.cat-tipo.saida {
  background-color: #3a1a1a;
  color: #cc4444;
}

.cat-tipo.neutro {
  background-color: #2a2a2a;
  color: var(--cor-texto-fraco);
}
</style>
