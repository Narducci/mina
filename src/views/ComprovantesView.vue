<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openPath } from "@tauri-apps/plugin-opener";
import { useToolbarStore } from "../stores/toolbar.js";
import { PanelRightClose, PanelRightOpen, Search, Pencil, Archive, Trash2, Bell, Merge, X } from "@lucide/vue";
import Database from "@tauri-apps/plugin-sql";
import { convertFileSrc } from "@tauri-apps/api/core";
import AppModal from "../components/AppModal.vue";

const toolbar = useToolbarStore();
const sidePanelAberto = ref(true);
const comprovantes = ref([]);
const termoBusca = ref("");
const linhaSelecionadaId = ref(null);
const abaAtiva = ref("identificar");
const salvando = ref(false);

// ── Modais ────────────────────────────────────────────────────────────
const modalErroTexto          = ref(null);   // texto → modal de erro visível
const modalConfirmarDesfazer  = ref(false);  // modal confirm desfazer tardio
const modalCancelar           = ref(false);  // modal confirm cancelar edição
let   _idDesfazerPendente     = null;

// ── Originais mesclados ───────────────────────────────────────────────
const originaisMesclados = ref([]);

// ── Formulário (Identificar) ───────────────────────────────────────────
const form = ref({
  nome: "",
  nome_curto: "",
  descricao: "",
  numero_documento: "",
  sem_numero: false,
  data_documento: "",
});
const formOriginal = ref({});

// ── Classificar ────────────────────────────────────────────────────────
const categorias = ref([]);
const categoriasSelecionadas = ref(new Set());
const categoriasSelecionadasOriginal = ref(new Set());
const termoBuscaCategoria = ref("");
const filtroTipo = ref("todos");

const tipoOpcoes = [
  { value: "todos",   label: "Todos"   },
  { value: "entrada", label: "Entrada" },
  { value: "saida",   label: "Saída"   },
  { value: "neutro",  label: "Neutro"  },
];

const categoriasFiltradas = computed(() => {
  let lista = categorias.value;
  if (filtroTipo.value !== "todos") lista = lista.filter(c => c.tipo === filtroTipo.value);
  if (termoBuscaCategoria.value.trim()) {
    const q = termoBuscaCategoria.value.trim().toLowerCase();
    lista = lista.filter(c => c.nome.toLowerCase().includes(q));
  }
  return lista;
});

// ── Dirty (computed) ───────────────────────────────────────────────────
const dirty = computed(() => {
  if (!linhaSelecionadaId.value) return false;
  const f = form.value, fo = formOriginal.value;
  const formDirty =
    (f.nome_curto       ?? "") !== (fo.nome_curto       ?? "") ||
    (f.descricao        ?? "") !== (fo.descricao        ?? "") ||
    (f.numero_documento ?? "") !== (fo.numero_documento ?? "") ||
    !!f.sem_numero              !== !!fo.sem_numero              ||
    (f.data_documento   ?? "") !== (fo.data_documento   ?? "");
  const cur = categoriasSelecionadas.value, ori = categoriasSelecionadasOriginal.value;
  const catDirty = cur.size !== ori.size || [...cur].some(id => !ori.has(id));
  return formDirty || catDirty;
});

// ── Mesclar ────────────────────────────────────────────────────────────
const modoMesclar         = ref(false);
const selecionadosMesclar = ref([]);
const confirmandoMesclar  = ref(false);
const nomeMesclado        = ref("");
const mesclando           = ref(false);
const toastMesclar        = ref(null);
let   toastTimer          = null;

// ── Computed gerais ────────────────────────────────────────────────────
const totalInbox = computed(() =>
  comprovantes.value.filter((c) => c.status === "inbox").length
);

const linhaAtual = computed(() =>
  comprovantes.value.find(c => c.id === linhaSelecionadaId.value) ?? null
);

// ── DB ─────────────────────────────────────────────────────────────────
let db = null;
async function getDb() {
  if (!db) db = await Database.load("sqlite:mina.db");
  return db;
}

// ── Helpers ────────────────────────────────────────────────────────────
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

function onSemNumero() {
  if (form.value.sem_numero) form.value.numero_documento = "";
}

const nomeNormalizado = computed(() => {
  const ext = (form.value.nome ?? "").includes(".")
    ? form.value.nome.split(".").pop() : "";
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

// ── Carregar ────────────────────────────────────────────────────────────
async function carregarComprovantes() {
  try {
    const banco = await getDb();
    const filtroSql = toolbar.filtro === "todos"
      ? ""
      : `AND c.status = '${toolbar.filtro === "inbox" ? "inbox" : "disponivel"}'`;
    comprovantes.value = await banco.select(`
      SELECT c.id, c.nome, c.nome_curto, c.descricao, c.numero_documento,
             c.data_documento, c.status, c.caminho_arquivo,
             COUNT(DISTINCT cc.categoria_id) AS total_categorias,
             COUNT(DISTINCT o.id) AS total_mesclados
      FROM comprovante c
      LEFT JOIN comprovante_categoria cc ON cc.comprovante_id = c.id
      LEFT JOIN comprovante o ON o.mesclado_em_id = c.id AND o.deletado_em IS NULL
      WHERE c.deletado_em IS NULL
        AND c.mesclado_em IS NULL
        ${filtroSql}
      GROUP BY c.id
      ORDER BY c.data_documento DESC NULLS LAST, c.id DESC
    `);
  } catch (err) {
    console.error("Erro ao carregar comprovantes:", err);
  }
}

async function carregarDetalhe(id) {
  if (!id) return;
  const banco = await getDb();
  const rows = await banco.select(
    `SELECT nome, nome_curto, descricao, numero_documento, data_documento FROM comprovante WHERE id = $1`,
    [id]
  );
  if (rows[0]) {
    const r = rows[0];
    const snap = {
      nome:             r.nome             ?? "",
      nome_curto:       r.nome_curto       ?? "",
      descricao:        r.descricao        ?? "",
      numero_documento: r.numero_documento ?? "",
      sem_numero:       false,
      data_documento:   isoParaBr(r.data_documento),
    };
    form.value         = { ...snap };
    formOriginal.value = { ...snap };
  }
}

async function carregarCategoriasComprovante(id) {
  const banco = await getDb();
  const rows  = await banco.select(
    "SELECT categoria_id FROM comprovante_categoria WHERE comprovante_id = $1",
    [id]
  );
  const s = new Set(rows.map(r => r.categoria_id));
  categoriasSelecionadas.value         = s;
  categoriasSelecionadasOriginal.value = new Set(s);
}

async function carregarCategorias() {
  const banco = await getDb();
  const rows  = await banco.select(`
    SELECT id, nome, tipo FROM categoria
    WHERE ativa = 1 AND deletado_em IS NULL
      AND nome NOT IN ('Aporte Período', 'Aporte Realizado')
    ORDER BY tipo, nome
  `);
  categorias.value = rows;
}

async function carregarOriginaisMesclados(id) {
  if (!id) { originaisMesclados.value = []; return; }
  const banco = await getDb();
  const rows  = await banco.select(
    `SELECT id, nome, nome_curto, numero_documento, caminho_arquivo
     FROM comprovante WHERE mesclado_em_id = $1`,
    [id]
  );
  originaisMesclados.value = rows;
}

async function ocultarOriginal(id) {
  const banco = await getDb();
  await banco.execute("UPDATE comprovante SET deletado_em = datetime('now') WHERE id = $1", [id]);
  await carregarOriginaisMesclados(linhaSelecionadaId.value);
}

// ── Selecionar linha ────────────────────────────────────────────────────
function selecionarLinha(c) {
  if (modoMesclar.value) { toggleMesclar(c); return; }
  if (dirty.value) {
    const ok = confirm("Há alterações não salvas. Descartar e continuar?");
    if (!ok) return;
  }
  linhaSelecionadaId.value = c.id;
  Promise.all([carregarDetalhe(c.id), carregarCategoriasComprovante(c.id)]);
}

// ── Toggle categoria ────────────────────────────────────────────────────
function toggleCategoria(catId) {
  const s = new Set(categoriasSelecionadas.value);
  if (s.has(catId)) s.delete(catId); else s.add(catId);
  categoriasSelecionadas.value = s;
}

// ── Número sequencial ────────────────────────────────────────────────────
async function gerarNumeroSequencial(banco) {
  const rows   = await banco.select("SELECT proximo FROM seq_documento WHERE id = 1");
  const proximo = rows[0]?.proximo ?? 1;
  await banco.execute("UPDATE seq_documento SET proximo = proximo + 1 WHERE id = 1");
  return "S" + String(proximo).padStart(8, "0");
}

// ── Cancelar edição ─────────────────────────────────────────────────────
function cancelarEdicao() {
  modalCancelar.value = true;
}

function confirmarCancelar() {
  modalCancelar.value = false;
  form.value = { ...formOriginal.value };
  categoriasSelecionadas.value = new Set(categoriasSelecionadasOriginal.value);
}

// ── Salvar unificado ────────────────────────────────────────────────────
async function salvar() {
  if (!linhaSelecionadaId.value || salvando.value) return;
  salvando.value = true;
  try {
    const banco = await getDb();
    const id    = linhaSelecionadaId.value;

    let numero = form.value.numero_documento.trim();
    if (form.value.sem_numero) {
      numero = await gerarNumeroSequencial(banco);
      form.value.numero_documento = numero;
      form.value.sem_numero       = false;
    } else if (numero) {
      numero = numero.padStart(9, "0").slice(0, 9);
      form.value.numero_documento = numero;
    }

    await banco.execute(
      `UPDATE comprovante SET nome_curto = $1, descricao = $2, numero_documento = $3, data_documento = $4 WHERE id = $5`,
      [form.value.nome_curto || null, form.value.descricao || null, numero || null, brParaIso(form.value.data_documento) || null, id]
    );

    await banco.execute("DELETE FROM comprovante_categoria WHERE comprovante_id = $1", [id]);
    for (const catId of categoriasSelecionadas.value) {
      await banco.execute(
        "INSERT INTO comprovante_categoria (comprovante_id, categoria_id) VALUES ($1, $2)",
        [id, catId]
      );
    }

    formOriginal.value               = { ...form.value };
    categoriasSelecionadasOriginal.value = new Set(categoriasSelecionadas.value);
    await carregarComprovantes();
  } catch (err) {
    modalErroTexto.value = "Erro ao salvar:\n" + err;
  } finally {
    salvando.value = false;
  }
}

// ── Outras ações ────────────────────────────────────────────────────────
async function arquivarLinha(id, event) {
  event.stopPropagation();
  const banco = await getDb();
  await banco.execute("UPDATE comprovante SET status = 'disponivel' WHERE id = $1", [id]);
  await carregarComprovantes();
}

async function excluirLinha(id, event) {
  event.stopPropagation();
  const banco = await getDb();
  await banco.execute("UPDATE comprovante SET deletado_em = datetime('now') WHERE id = $1", [id]);
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

// ── Teclado ─────────────────────────────────────────────────────────────
function handleTeclado(e) {
  const tag = document.activeElement?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA") return;
  const lista = compovantesFiltrados.value;
  if (!lista.length) return;
  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    const idx = lista.findIndex((c) => c.id === linhaSelecionadaId.value);
    let novo;
    if (e.key === "ArrowDown") { if (idx >= lista.length - 1) return; novo = lista[idx + 1]; }
    else { if (idx <= 0) return; novo = lista[idx - 1]; }
    linhaSelecionadaId.value = novo.id;
    Promise.all([carregarDetalhe(novo.id), carregarCategoriasComprovante(novo.id)]);
    const el = document.querySelector(`[data-id="${novo.id}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }
}

function alternarSidePanel() {
  sidePanelAberto.value = !sidePanelAberto.value;
}

// ── Mesclar ─────────────────────────────────────────────────────────────
function iniciarMesclar() {
  modoMesclar.value         = true;
  selecionadosMesclar.value = [];
  confirmandoMesclar.value  = false;
  nomeMesclado.value        = "";
}

function cancelarMesclar() {
  modoMesclar.value         = false;
  selecionadosMesclar.value = [];
  confirmandoMesclar.value  = false;
  nomeMesclado.value        = "";
}

function toggleMesclar(c) {
  if (!c.caminho_arquivo) return;
  const idx = selecionadosMesclar.value.findIndex(s => s.id === c.id);
  if (idx >= 0) selecionadosMesclar.value.splice(idx, 1);
  else selecionadosMesclar.value.push({ id: c.id, caminho_arquivo: c.caminho_arquivo, nomeDisplay: nomeExibicao(c) });
}

function ordemMesclar(c) {
  const idx = selecionadosMesclar.value.findIndex(s => s.id === c.id);
  return idx >= 0 ? idx + 1 : 0;
}

function moverMesclar(idx, dir) {
  const arr = selecionadosMesclar.value;
  const novo = idx + dir;
  if (novo < 0 || novo >= arr.length) return;
  const tmp = arr[idx]; arr[idx] = arr[novo]; arr[novo] = tmp;
}

function removerMesclar(idx) {
  selecionadosMesclar.value.splice(idx, 1);
}

async function executarMesclar() {
  if (selecionadosMesclar.value.length < 2 || mesclando.value) return;
  mesclando.value = true;
  let destino = null;
  try {
    const banco    = await getDb();
    const itens    = selecionadosMesclar.value;
    const caminhos = itens.map(i => i.caminho_arquivo);
    const primeiro = caminhos[0];
    const pasta    = primeiro.substring(0, primeiro.lastIndexOf("/") + 1);
    const nomeBase = (nomeMesclado.value.trim() || "mesclado").replace(/\.pdf$/i, "");
    destino        = pasta + nomeBase + ".pdf";

    // ── 1. Verificar hash duplicado antes de gravar no banco ──────────
    // (o arquivo físico já foi criado pelo invoke abaixo; verificamos
    //  o hash devolvido antes de qualquer escrita no banco)
    const { hash } = await invoke("mesclar_pdfs", { caminhos, destino });

    const jaExiste = await banco.select(
      "SELECT id FROM comprovante WHERE hash_arquivo = $1", [hash]
    );
    if (jaExiste.length > 0) {
      await invoke("excluir_arquivo", { caminho: destino }).catch(() => {});
      modalErroTexto.value = "Já existe um arquivo idêntico no acervo (mesmo conteúdo).";
      return;
    }

    const cfg = await banco.select("SELECT pasta_raiz_comprovantes FROM configuracao WHERE id = 1");
    const raiz = cfg[0]?.pasta_raiz_comprovantes || "";
    const caminhoRelMesclar = raiz && destino.startsWith(raiz)
      ? destino.slice(raiz.length).replace(/^\//, "")
      : nomeBase + ".pdf";

    // ── 2. INSERT + UPDATEs (rollback manual em caso de erro) ────────
    const idsOriginais = itens.map(i => i.id);
    const insResult = await banco.execute(
      "INSERT INTO comprovante (nome, caminho_relativo, caminho_arquivo, hash_arquivo, status) VALUES ($1, $2, $3, $4, 'inbox')",
      [nomeBase + ".pdf", caminhoRelMesclar, destino, hash]
    );
    const novoId = insResult.lastInsertId;

    try {
      for (const id of idsOriginais) {
        await banco.execute(
          "UPDATE comprovante SET mesclado_em = datetime('now'), mesclado_em_id = $1, deletado_em = datetime('now') WHERE id = $2",
          [novoId, id]
        );
      }
    } catch (errUpd) {
      // Desfaz o INSERT manualmente se os UPDATEs falharem
      await banco.execute("DELETE FROM comprovante WHERE id = $1", [novoId]).catch(() => {});
      throw errUpd;
    }

    destino = null; // arquivo está consistente — não deletar no catch externo
    cancelarMesclar();
    toolbar.filtro = 'inbox';
    await carregarComprovantes();
    linhaSelecionadaId.value = novoId;
    await Promise.all([carregarDetalhe(novoId), carregarCategoriasComprovante(novoId)]);
    mostrarToastMesclar(idsOriginais, novoId);
  } catch (err) {
    if (destino) await invoke("excluir_arquivo", { caminho: destino }).catch(() => {});
    modalErroTexto.value = "Erro ao mesclar:\n" + err;
  } finally {
    mesclando.value = false;
  }
}

function mostrarToastMesclar(idsOriginais, idResultado) {
  if (toastTimer) clearTimeout(toastTimer);
  toastMesclar.value = { idsOriginais, idResultado };
  toastTimer = setTimeout(() => { toastMesclar.value = null; }, 8000);
}

async function desfazerMesclar(idsOriginais, idResultado) {
  if (toastTimer) clearTimeout(toastTimer);
  toastMesclar.value = null;
  try {
    const banco = await getDb();
    // recupera caminho do arquivo mesclado antes de remover do banco
    const rows = await banco.select("SELECT caminho_arquivo FROM comprovante WHERE id = $1", [idResultado]);
    const caminhoMesclado = rows[0]?.caminho_arquivo;

    for (const id of idsOriginais) {
      await banco.execute(
        "UPDATE comprovante SET mesclado_em = NULL, mesclado_em_id = NULL, deletado_em = NULL WHERE id = $1",
        [id]
      );
    }
    await banco.execute("DELETE FROM comprovante WHERE id = $1", [idResultado]);

    if (caminhoMesclado) {
      await invoke("excluir_arquivo", { caminho: caminhoMesclado });
    }
    if (linhaSelecionadaId.value === idResultado) linhaSelecionadaId.value = null;
    await carregarComprovantes();
  } catch (err) {
    modalErroTexto.value = "Erro ao desfazer:\n" + err;
  }
}

function desfazerMesclarTardio(idResultado) {
  _idDesfazerPendente = idResultado;
  modalConfirmarDesfazer.value = true;
}

async function confirmarDesfazerTardio() {
  modalConfirmarDesfazer.value = false;
  if (!_idDesfazerPendente) return;
  const id = _idDesfazerPendente;
  _idDesfazerPendente = null;
  const banco = await getDb();
  const rows  = await banco.select("SELECT id FROM comprovante WHERE mesclado_em_id = $1 AND deletado_em IS NULL", [id]);
  await desfazerMesclar(rows.map(r => r.id), id);
}

// ── Watch / ciclo ────────────────────────────────────────────────────────
watch(linhaSelecionadaId, (id) => {
  toolbar.exibirAtivo = id !== null;
  if (id === null) { toolbar.exibirPdf = false; originaisMesclados.value = []; }
  else carregarOriginaisMesclados(id);
});

watch(() => toolbar.filtro, async () => {
  linhaSelecionadaId.value = null;
  await carregarComprovantes();
  if (comprovantes.value.length > 0) {
    const primeiro = comprovantes.value[0];
    linhaSelecionadaId.value = primeiro.id;
    await Promise.all([carregarDetalhe(primeiro.id), carregarCategoriasComprovante(primeiro.id)]);
  }
});

watch(modoMesclar, (v) => { toolbar.mesclarAtivo = v; });

let unlistenNovo = null;

onMounted(async () => {
  toolbar.ativarComprovantes({
    importar: () => console.log("importar"),
    excluir: () => linhaSelecionadaId.value && excluirLinha(linhaSelecionadaId.value, { stopPropagation: () => {} }),
    exibir: exibirArquivo,
    mesclar: iniciarMesclar,
  });
  toolbar.exibirAtivo = false;
  await Promise.all([carregarComprovantes(), carregarCategorias()]);
  if (comprovantes.value.length > 0) {
    const primeiro = comprovantes.value[0];
    linhaSelecionadaId.value = primeiro.id;
    await Promise.all([carregarDetalhe(primeiro.id), carregarCategoriasComprovante(primeiro.id)]);
  }
  document.addEventListener("keydown", handleTeclado);

  unlistenNovo = await listen("comprovante:novo", async (event) => {
    const caminho = event.payload;
    const banco   = await getDb();
    // Ignora se o caminho já está registrado (ex: arquivo criado pela mesclagem)
    const exist   = await banco.select("SELECT id FROM comprovante WHERE caminho_arquivo = $1", [caminho]);
    if (exist.length === 0) {
      const nome = caminho.split("/").pop() || caminho;
      const hash = await invoke("calcular_hash", { caminho });
      // Ignora se o hash já está registrado (dupla garantia contra corrida com mesclagem)
      const existHash = await banco.select("SELECT id FROM comprovante WHERE hash_arquivo = $1", [hash]);
      if (existHash.length === 0) {
        const cfgW = await banco.select("SELECT pasta_raiz_comprovantes FROM configuracao WHERE id = 1");
        const raizW = cfgW[0]?.pasta_raiz_comprovantes || "";
        const caminhoRelW = raizW && caminho.startsWith(raizW)
          ? caminho.slice(raizW.length).replace(/^\//, "")
          : nome;
        await banco.execute(
          "INSERT INTO comprovante (nome, caminho_relativo, caminho_arquivo, hash_arquivo, status) VALUES ($1, $2, $3, $4, 'inbox')",
          [nome, caminhoRelW, caminho, hash]
        );
        await carregarComprovantes();
      }
    }
  });
});

onUnmounted(() => {
  toolbar.desativar();
  document.removeEventListener("keydown", handleTeclado);
  unlistenNovo?.();
  if (toastTimer) clearTimeout(toastTimer);
});
</script>

<template>
  <div class="main-panel">
    <div class="shell">
      <div class="area-central">
        <div class="conteudo-principal">

          <!-- Painel superior -->
          <div class="painel-superior">
            <div class="busca-wrapper">
              <Search :size="14" class="busca-icone" />
              <input class="busca" type="text" placeholder="Buscar..." v-model="termoBusca" />
              <button v-if="termoBusca" class="busca-limpar" @click="termoBusca = ''">✕</button>
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
          <div class="datagrid-header" :class="{ 'modo-mesclar': modoMesclar }">
            <span class="col-nome">Nome e Descrição</span>
            <span class="col-cat">Categorias</span>
            <span class="col-data">Data</span>
            <span class="col-status">Status</span>
            <span v-if="!modoMesclar" class="col-acoes">Ações</span>
          </div>

          <!-- Visualizador de PDF -->
          <iframe v-if="toolbar.exibirPdf && pdfSrc" :src="pdfSrc" class="pdf-viewer" />
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
              :class="{
                selecionada:        !modoMesclar && c.id === linhaSelecionadaId,
                'em-exibicao':      c.id === linhaSelecionadaId && toolbar.exibirPdf,
                'mesclar-selecionada': modoMesclar && ordemMesclar(c) > 0,
                'mesclar-sem-pdf':  modoMesclar && !c.caminho_arquivo,
              }"
              @click="selecionarLinha(c)"
            >
              <div class="col-nome">
                <div class="col-nome-top">
                  <span v-if="modoMesclar && ordemMesclar(c) > 0" class="ordem-badge">{{ ordemMesclar(c) }}</span>
                  <span class="nome-arquivo">{{ nomeExibicao(c) }}</span>
                  <span v-if="c.total_mesclados > 0" class="badge-mesclado" title="Resultado de mesclagem">M</span>
                </div>
                <span v-if="c.descricao" class="descricao">{{ c.descricao }}</span>
              </div>
              <div class="col-cat">{{ c.total_categorias }}</div>
              <div class="col-data">{{ formatarData(c.data_documento) }}</div>
              <div class="col-status">
                <span class="badge" :class="c.status">
                  {{ c.status === 'inbox' ? 'Inbox' : 'Disponível' }}
                </span>
              </div>
              <div v-if="!modoMesclar" class="col-acoes">
                <button class="btn-acao" title="Identificar" @click.stop="selecionarLinha(c); sidePanelAberto = true"><Pencil :size="13" /></button>
                <button class="btn-acao" v-if="c.status === 'inbox'" title="Disponibilizar" :disabled="!identificarCompleto(c)" :class="{ 'btn-bloqueado': !identificarCompleto(c) }" @click.stop="arquivarLinha(c.id, $event)"><Archive :size="13" /></button>
                <button class="btn-acao btn-excluir" title="Excluir" @click.stop="excluirLinha(c.id, $event)"><Trash2 :size="13" /></button>
              </div>
            </div>

            <div v-if="compovantesFiltrados.length === 0" class="estado-vazio">
              Nenhum comprovante encontrado.
            </div>
          </div>

          <!-- Painel inferior -->
          <div class="painel-inferior" :class="{ 'painel-mesclar-ativo': modoMesclar }">
            <template v-if="!modoMesclar">
              <span class="rodape-contador">
                {{ compovantesFiltrados.length }} documento{{ compovantesFiltrados.length !== 1 ? 's' : '' }}
              </span>
            </template>

            <template v-else-if="!confirmandoMesclar">
              <div class="mesclar-chips-area">
                <span v-if="selecionadosMesclar.length === 0" class="mesclar-hint">
                  Clique nos arquivos PDF para selecionar a ordem
                </span>
                <div v-for="(item, idx) in selecionadosMesclar" :key="item.id" class="chip-mesclar">
                  <span class="chip-ordem">{{ idx + 1 }}</span>
                  <span class="chip-nome" :title="item.nomeDisplay">{{ item.nomeDisplay }}</span>
                  <button class="chip-btn" :disabled="idx === 0" @click.stop="moverMesclar(idx, -1)">↑</button>
                  <button class="chip-btn" :disabled="idx === selecionadosMesclar.length - 1" @click.stop="moverMesclar(idx, 1)">↓</button>
                  <button class="chip-btn chip-remove" @click.stop="removerMesclar(idx)">×</button>
                </div>
              </div>
              <div class="mesclar-rodape-acoes">
                <span class="mesclar-count">{{ selecionadosMesclar.length }} selecionado{{ selecionadosMesclar.length !== 1 ? 's' : '' }}</span>
                <button class="btn-mesclar-acao btn-secundario" @click="cancelarMesclar">Cancelar</button>
                <button class="btn-mesclar-acao btn-primario" :disabled="selecionadosMesclar.length < 2" @click="confirmandoMesclar = true">Confirmar →</button>
              </div>
            </template>

            <template v-else>
              <div class="mesclar-nome-area">
                <label class="mesclar-nome-label">Nome do arquivo:</label>
                <input v-model="nomeMesclado" type="text" class="mesclar-nome-input" placeholder="mesclado" @keyup.enter="executarMesclar" />
                <span class="mesclar-nome-dica">.pdf</span>
              </div>
              <div class="mesclar-rodape-acoes">
                <button class="btn-mesclar-acao btn-secundario" @click="confirmandoMesclar = false">← Voltar</button>
                <button class="btn-mesclar-acao btn-primario" :disabled="mesclando" @click="executarMesclar">{{ mesclando ? 'Mesclando…' : 'Mesclar' }}</button>
              </div>
            </template>
          </div>

        </div>

        <!-- Side panel -->
        <div class="side-panel" :class="{ aberto: sidePanelAberto }">
          <div class="side-panel-cabecalho">
            <div v-if="sidePanelAberto" class="abas">
              <button class="aba" :class="{ ativa: abaAtiva === 'identificar' }" @click="abaAtiva = 'identificar'">Identificar</button>
              <button class="aba" :class="{ ativa: abaAtiva === 'classificar' }" @click="abaAtiva = 'classificar'">Classificar</button>
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

                <div v-if="linhaAtual?.total_mesclados > 0" class="aviso-mesclado">
                  <Merge :size="12" />
                  Resultado de mesclagem ({{ linhaAtual.total_mesclados }} originais)
                  <button class="btn-link-danger" @click="desfazerMesclarTardio(linhaSelecionadaId)">Desfazer</button>
                </div>

                <div v-if="originaisMesclados.length > 0" class="originais-mesclados">
                  <div class="originais-titulo">Arquivos mesclados</div>
                  <div v-for="o in originaisMesclados" :key="o.id" class="original-item">
                    <span class="original-nome" :title="o.nome">
                      {{ o.numero_documento ? o.numero_documento + ' · ' : '' }}{{ o.nome_curto || o.nome }}
                    </span>
                  </div>
                </div>

                <div class="campo">
                  <label>Nome do arquivo</label>
                  <div class="campo-readonly">{{ form.nome || '—' }}</div>
                </div>

                <div class="campo">
                  <label>Número do documento</label>
                  <input v-model="form.numero_documento" type="text" maxlength="9" :disabled="form.sem_numero" placeholder="Ex: NF-001" :class="{ desabilitado: form.sem_numero }" />
                  <label class="checkbox-label" :class="{ desabilitado: form.numero_documento.trim() !== '' }">
                    <input type="checkbox" v-model="form.sem_numero" @change="onSemNumero"
                      :disabled="form.numero_documento.trim() !== ''" />
                    Documento sem número
                  </label>
                </div>

                <div class="campo">
                  <label>Descrição <span class="obrigatorio">*</span></label>
                  <textarea v-model="form.descricao" rows="3" placeholder="Descrição do documento…" />
                </div>

                <div class="campo">
                  <label>Data da Transação <span class="obrigatorio">*</span></label>
                  <input v-model="form.data_documento" type="text" placeholder="DD/MM/AAAA" @input="onDataInput" />
                </div>

                <div class="campo">
                  <label>Nome curto <span class="obrigatorio">*</span></label>
                  <input v-model="form.nome_curto" type="text" placeholder="Ex: Aluguel Joao" />
                </div>

                <div class="campo">
                  <label>Nome normalizado</label>
                  <div class="campo-preview" :title="nomeNormalizado">{{ nomeNormalizado }}</div>
                </div>

                <!-- Classificações selecionadas -->
                <div v-if="categoriasSelecionadas.size > 0" class="classificacoes-resumo">
                  <div class="classificacoes-titulo">Classificações</div>
                  <div class="classificacoes-lista">
                    <span
                      v-for="cat in categorias.filter(c => categoriasSelecionadas.has(c.id))"
                      :key="cat.id"
                      class="classificacao-badge"
                      :class="cat.tipo"
                    >{{ cat.nome }}</span>
                  </div>
                </div>

              </div>
            </template>

            <!-- Aba Classificar -->
            <template v-if="abaAtiva === 'classificar'">
              <div v-if="!linhaSelecionadaId" class="painel-vazio">
                Selecione um comprovante na lista.
              </div>
              <div v-else class="form-classificar">
                <div class="classificar-toolbar">
                  <input v-model="termoBuscaCategoria" type="text" placeholder="Filtrar categorias…" class="input-cat" />
                  <div class="filtro-tipo">
                    <button v-for="o in tipoOpcoes" :key="o.value" class="btn-tipo" :class="{ ativo: filtroTipo === o.value }" @click="filtroTipo = o.value">{{ o.label }}</button>
                  </div>
                </div>
                <div class="cat-lista">
                  <div v-if="categoriasFiltradas.length === 0" class="cat-vazio">Nenhuma categoria.</div>
                  <label v-for="cat in categoriasFiltradas" :key="cat.id" class="cat-item">
                    <input type="checkbox" :checked="categoriasSelecionadas.has(cat.id)" @change="toggleCategoria(cat.id)" />
                    <span class="cat-tipo-badge" :class="'cat-' + cat.tipo">{{ cat.tipo[0].toUpperCase() }}</span>
                    <span class="cat-nome">{{ cat.nome }}</span>
                  </label>
                </div>
              </div>
            </template>

          </div>

          <!-- Footer salvar -->
          <div v-if="sidePanelAberto && linhaSelecionadaId" class="side-panel-footer">
            <span v-if="dirty" class="footer-dirty">● Não salvo</span>
            <button v-if="dirty" class="btn-cancelar" :disabled="salvando" @click="cancelarEdicao">
              Cancelar
            </button>
            <button class="btn-salvar" :disabled="salvando || !dirty" @click="salvar">
              {{ salvando ? 'Salvando…' : 'Salvar' }}
            </button>
          </div>

        </div>
      </div>
    </div>
  </div>

  <!-- Toast mesclar -->
  <Teleport to="body">
    <div v-if="toastMesclar" class="toast-mesclar">
      <span>Mesclagem concluída.</span>
      <button class="toast-desfazer" @click="desfazerMesclar(toastMesclar.idsOriginais, toastMesclar.idResultado)">Desfazer</button>
    </div>
  </Teleport>

  <!-- Modal: erro genérico -->
  <Teleport to="body">
    <AppModal
      v-if="modalErroTexto"
      titulo="Erro"
      texto-confirmar="OK"
      :exibir-cancelar="false"
      @fechar="modalErroTexto = null"
      @confirmar="modalErroTexto = null"
    >
      <pre style="white-space:pre-wrap;font-size:0.82rem;margin:0">{{ modalErroTexto }}</pre>
    </AppModal>
  </Teleport>

  <!-- Modal: confirmar desfazer mesclagem tardia -->
  <Teleport to="body">
    <AppModal
      v-if="modalConfirmarDesfazer"
      titulo="Desfazer mesclagem"
      texto-cancelar="Cancelar"
      texto-confirmar="Desfazer"
      @fechar="modalConfirmarDesfazer = false"
      @confirmar="confirmarDesfazerTardio"
    >
      <p style="margin:0">Isso irá restaurar os arquivos originais e excluir o arquivo mesclado. Deseja continuar?</p>
    </AppModal>
  </Teleport>

  <!-- Modal: confirmar cancelar edição -->
  <Teleport to="body">
    <AppModal
      v-if="modalCancelar"
      titulo="Descartar alterações"
      texto-cancelar="Continuar editando"
      texto-confirmar="Descartar"
      @fechar="modalCancelar = false"
      @confirmar="confirmarCancelar"
    >
      <p style="margin:0">As alterações não salvas serão descartadas. Deseja continuar?</p>
    </AppModal>
  </Teleport>
</template>

<style scoped>
/* ── Layout original preservado ──────────────────────────────────────── */
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

.busca-icone { color: var(--cor-texto-fraco); flex-shrink: 0; }

.busca {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
}

.busca::placeholder { color: var(--cor-texto-fraco); }

.busca-limpar {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 12px;
  padding: 0;
}
.busca-limpar:hover { color: #cc4444; }

/* Botão Mesclar na toolbar */
/* Sininho */
.sininho-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex-shrink: 0;
  cursor: default;
  margin-left: auto;
}
.sininho-icone { color: var(--cor-texto-fraco); }
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
  grid-template-columns: 1fr 90px 100px 110px 90px;
  align-items: center;
  padding: 0 16px 0 10px;
}

.datagrid-header.modo-mesclar,
.modo-mesclar.linha {
  grid-template-columns: 1fr 90px 100px 110px;
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

.pdf-viewer { flex: 1; border: none; background: #111; }
.pdf-sem-arquivo {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--cor-texto-fraco);
  font-size: 13px;
}

.datagrid-body { flex: 1; overflow-y: auto; }

.linha {
  min-height: 44px;
  border-bottom: 1px solid #2a2a2a;
  cursor: pointer;
  transition: background-color 0.1s;
}
.linha:hover { background-color: var(--cor-menu-hover); }
.linha.selecionada { background-color: #1a3a5a; }
.linha.selecionada:hover { background-color: #1e4268; }
.linha.em-exibicao { background-color: #0f2a40; border-left: 2px solid #4a9eff; }
.linha.mesclar-selecionada { background-color: #1a3a1a; border-left: 2px solid #4a7a4a; }
.linha.mesclar-sem-pdf { opacity: 0.35; cursor: not-allowed; }

.col-nome {
  display: flex;
  flex-direction: column;
  justify-content: center;
  overflow: hidden;
  min-width: 0;
}

.col-nome-top {
  display: flex;
  align-items: center;
  gap: 4px;
  overflow: hidden;
  min-width: 0;
}

.nome-arquivo {
  display: block;
  font-size: 13px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  flex-shrink: 1;
  min-width: 0;
}

.descricao {
  display: block;
  font-size: 11px;
  color: var(--cor-texto-fraco);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.ordem-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 17px;
  height: 17px;
  border-radius: 50%;
  background: #4a7a4a;
  color: #fff;
  font-size: 9px;
  font-weight: 700;
  flex-shrink: 0;
}

.badge-mesclado {
  font-size: 9px;
  font-weight: 700;
  background: #1a3a5a;
  color: #4a9eff;
  border-radius: 3px;
  padding: 0 3px;
  flex-shrink: 0;
}

.col-cat, .col-data, .col-status, .col-acoes {
  font-size: 12px;
  color: var(--cor-texto);
}

.col-cat {
  text-align: center;
}

.col-acoes {
  display: flex;
  align-items: center;
  gap: 4px;
  justify-content: flex-end;
}

/* Header spans precisam herdar o alinhamento das células */
.datagrid-header .col-acoes {
  display: flex;
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
.btn-acao:hover { opacity: 1; }
.btn-excluir:hover { color: #cc4444; }
.btn-bloqueado { opacity: 0.2 !important; cursor: not-allowed !important; }

.badge {
  font-size: 10px;
  font-weight: bold;
  padding: 3px 10px;
  border-radius: 10px;
  letter-spacing: 0.5px;
  white-space: nowrap;
}
.badge.inbox     { background-color: #2a2a1a; color: #ccaa44; }
.badge.disponivel { background-color: #1a3a5a; color: #4a9eff; }

/* Painel inferior */
.painel-inferior {
  flex-shrink: 0;
  min-height: 32px;
  border-top: 1px solid var(--cor-borda);
  display: flex;
  align-items: center;
  padding: 0 12px;
  gap: 8px;
}

.rodape-contador { font-size: 12px; color: var(--cor-texto-fraco); }

/* Modo mesclar — painel inferior */
.painel-mesclar-ativo {
  background-color: #111f11;
  border-top-color: #4a7a4a;
  min-height: 44px;
  flex-wrap: nowrap;
  padding: 6px 12px;
}

.mesclar-chips-area {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  overflow-x: auto;
}

.mesclar-hint {
  color: var(--cor-texto-fraco);
  font-style: italic;
  font-size: 11px;
  white-space: nowrap;
}

.chip-mesclar {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: #1a2a1a;
  border: 1px solid #4a7a4a;
  border-radius: 12px;
  padding: 2px 6px;
  white-space: nowrap;
  flex-shrink: 0;
  font-size: 11px;
}

.chip-ordem {
  background: #4a7a4a;
  color: #fff;
  border-radius: 50%;
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 8px;
  font-weight: 700;
  flex-shrink: 0;
}

.chip-nome {
  max-width: 100px;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--cor-texto);
}

.chip-btn {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--cor-texto-fraco);
  padding: 0 1px;
  font-size: 11px;
  line-height: 1;
}
.chip-btn:disabled { opacity: 0.3; cursor: default; }
.chip-btn.chip-remove { color: #cc4444; font-size: 13px; }

.mesclar-rodape-acoes {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.mesclar-count { color: var(--cor-texto-fraco); font-size: 11px; white-space: nowrap; }

.mesclar-nome-area {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}
.mesclar-nome-label { color: var(--cor-texto-fraco); white-space: nowrap; font-size: 12px; }
.mesclar-nome-input {
  flex: 1;
  padding: 4px 6px;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  background: #111;
  color: var(--cor-texto-forte);
  font-size: 12px;
  font-family: inherit;
  outline: none;
  min-width: 80px;
}
.mesclar-nome-dica { color: var(--cor-texto-fraco); font-size: 11px; }

.btn-mesclar-acao {
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-family: inherit;
  border: 1px solid var(--cor-borda);
}
.btn-secundario { background: none; color: var(--cor-texto-fraco); }
.btn-secundario:hover { color: var(--cor-texto-forte); }
.btn-primario { background: #1a4a7a; color: #fff; border-color: #1a4a7a; font-weight: 600; }
.btn-primario:hover:not(:disabled) { background: #1e5a94; }
.btn-primario:disabled { opacity: 0.4; cursor: default; }

/* Estado vazio */
.estado-vazio {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--cor-texto-fraco);
  font-size: 13px;
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
.side-panel.aberto { width: 400px; }

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
.btn-toggle-panel:hover { color: var(--cor-texto-forte); }

.side-panel-corpo {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

/* Abas */
.abas { display: flex; gap: 2px; flex: 1; }
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
.aba.ativa { color: var(--cor-texto-forte); border-bottom-color: #4a9eff; }

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

.form-identificar { display: flex; flex-direction: column; gap: 12px; }

.aviso-mesclado {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px;
  background: #0d1a2a;
  border-radius: 4px;
  font-size: 11px;
  color: #4a9eff;
  border: 1px solid #1e3a5f;
}
.originais-mesclados {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.originais-titulo {
  font-size: 10px;
  font-weight: 600;
  color: #8899aa;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin-bottom: 2px;
}
.original-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 3px 6px;
  background: #0d1a2a;
  border-radius: 3px;
  font-size: 11px;
}
.original-nome {
  flex: 1;
  color: #aabbcc;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.original-ocultar {
  flex-shrink: 0;
  background: none;
  border: none;
  cursor: pointer;
  color: #556677;
  padding: 1px;
  display: flex;
  align-items: center;
}
.original-ocultar:hover {
  color: #cc4444;
}

.classificacoes-resumo { margin-top: 12px; }
.classificacoes-titulo { font-size: 10px; font-weight: 600; color: #8899aa; text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 5px; }
.classificacoes-lista { display: flex; flex-wrap: wrap; gap: 4px; }
.classificacao-badge {
  font-size: 11px;
  padding: 2px 7px;
  border-radius: 3px;
  font-weight: 500;
  border: 1px solid;
}
.classificacao-badge.entrada  { color: #4daa70; border-color: #2a5c3a; background: rgba(77,170,112,0.08); }
.classificacao-badge.saida    { color: #cc5555; border-color: #6a2a2a; background: rgba(204,85,85,0.08); }
.classificacao-badge.neutro   { color: #8899aa; border-color: #334455; background: rgba(136,153,170,0.08); }

.btn-link-danger {
  background: none;
  border: none;
  cursor: pointer;
  color: #cc4444;
  font-size: 11px;
  text-decoration: underline;
  padding: 0;
  margin-left: auto;
}

.campo { display: flex; flex-direction: column; gap: 4px; }
.campo label {
  font-size: 10px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--cor-texto-fraco);
}
.obrigatorio { color: #cc4444; font-size: 11px; }

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
.campo input.desabilitado { opacity: 0.4; cursor: not-allowed; }
.checkbox-label.desabilitado { opacity: 0.4; cursor: not-allowed; pointer-events: none; }
.campo textarea { resize: vertical; }
.campo input:focus, .campo textarea:focus { border-color: #4a9eff; }

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
.checkbox-label input[type="checkbox"] { width: auto; padding: 0; border: none; background: none; cursor: pointer; }

/* Formulário Classificar */
.form-classificar { display: flex; flex-direction: column; gap: 8px; }

.classificar-toolbar { display: flex; flex-direction: column; gap: 6px; }

.input-cat {
  padding: 5px 8px;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  background: #111;
  color: var(--cor-texto-forte);
  font-size: 12px;
  font-family: inherit;
  outline: none;
}
.input-cat:focus { border-color: #4a9eff; }

.filtro-tipo { display: flex; gap: 2px; }
.btn-tipo {
  flex: 1;
  padding: 3px 0;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  background: none;
  cursor: pointer;
  font-size: 11px;
  color: var(--cor-texto-fraco);
  font-family: inherit;
  transition: background-color 0.1s, color 0.1s;
}
.btn-tipo:hover { color: var(--cor-texto-forte); }
.btn-tipo.ativo { background: #1a4a7a; color: #fff; border-color: #1a4a7a; }

.cat-lista { display: flex; flex-direction: column; gap: 1px; }
.cat-vazio { font-size: 12px; color: var(--cor-texto-fraco); padding: 8px 0; }

.cat-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 4px;
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  color: var(--cor-texto);
}
.cat-item:hover { background-color: var(--cor-menu-hover); }

.cat-tipo-badge {
  font-size: 8px;
  font-weight: 700;
  padding: 1px 4px;
  border-radius: 3px;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  flex-shrink: 0;
}
.cat-entrada { background: #1a3a1a; color: #4aaa4a; }
.cat-saida   { background: #3a1a1a; color: #aa4a4a; }
.cat-neutro  { background: #2a2a2a; color: #8a8a8a; }
.cat-nome { flex: 1; }

/* Side panel footer */
.side-panel-footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  padding: 8px 12px;
  border-top: 1px solid var(--cor-borda);
  flex-shrink: 0;
  background-color: var(--cor-menu-bg);
}
.footer-dirty {
  font-size: 11px;
  color: #ccaa44;
  margin-right: auto;
}

.btn-salvar {
  margin-top: 0;
  background-color: #1a4a7a;
  border: none;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 6px 16px;
  transition: background-color 0.15s;
}
.btn-salvar:hover:not(:disabled) { background-color: #1e5a94; }
.btn-salvar:disabled { opacity: 0.4; cursor: default; }

.btn-cancelar {
  margin-top: 0;
  background-color: transparent;
  border: 1px solid #445566;
  border-radius: 4px;
  color: #aabbcc;
  cursor: pointer;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  padding: 6px 14px;
  transition: border-color 0.15s, color 0.15s;
}
.btn-cancelar:hover:not(:disabled) { border-color: #cc4444; color: #cc4444; }
.btn-cancelar:disabled { opacity: 0.4; cursor: default; }

/* Toast mesclar */
.toast-mesclar {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 18px;
  background: #111;
  color: var(--cor-texto-forte);
  border: 1px solid var(--cor-borda);
  border-radius: 8px;
  font-size: 13px;
  box-shadow: 0 4px 16px rgba(0,0,0,0.5);
  z-index: 9999;
  white-space: nowrap;
  animation: slideUp 0.2s ease;
}
.toast-desfazer {
  background: none;
  border: none;
  cursor: pointer;
  color: #4a9eff;
  font-size: 13px;
  font-weight: 600;
  text-decoration: underline;
  padding: 0;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateX(-50%) translateY(10px); }
  to   { opacity: 1; transform: translateX(-50%) translateY(0); }
}
</style>
