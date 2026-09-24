<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from "vue";
import { useRoute } from "vue-router";
import Database from "@tauri-apps/plugin-sql";
import { ChevronLeft, ChevronRight, ChevronDown, Search, Eye } from "@lucide/vue";
import { VueDraggable } from "vue-draggable-plus";
import ModalNovoPeriodo from "../components/ModalNovoPeriodo.vue";
import ModalNovoLancamento from "../components/ModalNovoLancamento.vue";
import ModalLancamento from "../components/ModalLancamento.vue";

const route = useRoute();

let db = null;
async function getDb() {
  if (!db) db = await Database.load("sqlite:mina.db");
  return db;
}

// ── Períodos ──────────────────────────────────────────────
const periodos = ref([]);
const periodoAtivoId = ref(null);
const erroCarregamento = ref("");
const menuPeriodosAberto = ref(false);
const menuAnchorRef = ref(null);

const periodoAtivo = computed(
  () => periodos.value.find((p) => p.id === periodoAtivoId.value) ?? null,
);
const indicePeriodoAtivo = computed(() =>
  periodos.value.findIndex((p) => p.id === periodoAtivoId.value),
);

function podePeriodoAnterior() {
  return indicePeriodoAtivo.value > 0;
}
function podeProximoPeriodo() {
  return indicePeriodoAtivo.value < periodos.value.length - 1;
}
function irPeriodoAnterior() {
  if (podePeriodoAnterior())
    periodoAtivoId.value = periodos.value[indicePeriodoAtivo.value - 1].id;
}
function irProximoPeriodo() {
  if (podeProximoPeriodo())
    periodoAtivoId.value = periodos.value[indicePeriodoAtivo.value + 1].id;
}

function formatarPeriodo(p) {
  const meses = [
    "Jan",
    "Fev",
    "Mar",
    "Abr",
    "Mai",
    "Jun",
    "Jul",
    "Ago",
    "Set",
    "Out",
    "Nov",
    "Dez",
  ];
  return `${meses[p.mes - 1]}/${p.ano}`;
}

function toggleMenuPeriodos() {
  menuPeriodosAberto.value = !menuPeriodosAberto.value;
}
function fecharMenuPeriodos() {
  menuPeriodosAberto.value = false;
}

function handleClickFora(e) {
  if (menuAnchorRef.value && !menuAnchorRef.value.contains(e.target))
    fecharMenuPeriodos();
}

async function carregarPeriodos(idParaAtivar = null) {
  erroCarregamento.value = "";
  try {
    const banco = await getDb();
    periodos.value = await banco.select(
      `SELECT id, mes, ano, status, saldo_inicial, comprovante_id
         FROM periodo WHERE deletado_em IS NULL ORDER BY ano, mes`,
    );
    if (periodos.value.length === 0) {
      periodoAtivoId.value = null;
      lancamentos.value = [];
      return;
    }
    const alvo = idParaAtivar ?? periodoAtivoId.value;
    const existe = periodos.value.some((p) => p.id === alvo);
    periodoAtivoId.value = existe
      ? alvo
      : periodos.value[periodos.value.length - 1].id;
  } catch (err) {
    console.error("Erro ao carregar períodos:", err);
    erroCarregamento.value = "Erro ao carregar períodos.";
  }
}

// ── Lançamentos ───────────────────────────────────────────
const lancamentos = ref([]);
const categorias = ref([]);
const termoBusca = ref("");

// Lista base filtrada — é sobre ela que o drag & drop opera
const lancamentosBase = computed(() => {
  const termo = termoBusca.value.trim().toLowerCase();
  if (!termo) return lancamentos.value;
  return lancamentos.value.filter(
    (l) =>
      (l.descricao ?? "").toLowerCase().includes(termo) ||
      (l.categoria_nome ?? "").toLowerCase().includes(termo) ||
      (l.pessoa_nome ?? "").toLowerCase().includes(termo),
  );
});

// Lista com saldo acumulado calculado linha a linha
const lancamentosComSaldo = computed(() => {
  const saldoInicial = periodoAtivo.value?.saldo_inicial ?? 0;
  let acumulado = saldoInicial;
  return lancamentosBase.value.map((l) => {
    const cat = categorias.value.find((c) => c.id === l.categoria_id);
    const tipo = cat?.tipo ?? "neutro";
    if (tipo === "entrada") acumulado += l.valor;
    else if (tipo === "saida") acumulado -= l.valor;
    return { ...l, saldo_acumulado: acumulado, tipo_categoria: tipo };
  });
});

// Lista reativa usada pelo VueDraggable (somente quando não há filtro)
const lancamentosArrastaveis = ref([]);

watch(
  lancamentos,
  (val) => {
    lancamentosArrastaveis.value = [...val];
  },
  { immediate: true },
);

function onDragEnd() {
  // Drag & drop visual — persistência on-hold
  lancamentos.value = [...lancamentosArrastaveis.value]
}

const saldoFinal = computed(() => {
  const rows = lancamentosComSaldo.value;
  if (rows.length === 0) return periodoAtivo.value?.saldo_inicial ?? 0;
  return rows[rows.length - 1].saldo_acumulado;
});

async function carregarCategorias() {
  try {
    const banco = await getDb();
    categorias.value = await banco.select(
      `SELECT id, nome, tipo FROM categoria
        WHERE deletado_em IS NULL AND ativa = 1
          AND disponivel_em IN ('direto','ambos') ORDER BY nome`,
    );
  } catch (err) {
    console.error("Erro ao carregar categorias:", err);
  }
}

async function carregarLancamentos() {
  if (!periodoAtivoId.value) {
    lancamentos.value = [];
    return;
  }
  try {
    const banco = await getDb();
    lancamentos.value = await banco.select(
      `SELECT l.id, l.data, l.descricao, l.valor, l.ordem,
              l.categoria_id, l.pessoa_id, l.comprovante_id,
              c.nome AS categoria_nome,
              p.nome AS pessoa_nome
         FROM lancamento l
         JOIN categoria c ON c.id = l.categoria_id
         LEFT JOIN pessoa p ON p.id = l.pessoa_id
        WHERE l.periodo_id = ? AND l.tipo = 'direto' AND l.deletado_em IS NULL
        ORDER BY l.ordem`,
      [periodoAtivoId.value],
    );
  } catch (err) {
    console.error("Erro ao carregar lançamentos:", err);
  }
}

watch(periodoAtivoId, async () => {
  linhaSelecionadaId.value = null;
  await carregarLancamentos();
});

// ── Formatação ────────────────────────────────────────────
function formatarData(iso) {
  if (!iso) return "";
  const [a, m, d] = iso.split("-");
  return `${d}/${m}/${a}`;
}

function formatarValor(valor, tipo) {
  const abs = Math.abs(valor).toLocaleString("pt-BR", {
    minimumFractionDigits: 2,
  });
  if (tipo === "entrada") return `+R$ ${abs}`;
  if (tipo === "saida") return `-R$ ${abs}`;
  return `R$ ${abs}`;
}

function formatarReais(valor) {
  const abs = Math.abs(valor).toLocaleString("pt-BR", {
    minimumFractionDigits: 2,
  });
  if (valor < 0) return `(R$ ${abs})`;
  return `R$ ${abs}`;
}

// ── Seleção de linha ──────────────────────────────────────
const linhaSelecionadaId = ref(null);

function selecionarLinha(l) {
  linhaSelecionadaId.value = l.id;
}

function handleTecladoDatagrid(e) {
  // Não interfere quando há modal aberto
  if (modalLancamentoAberto.value || modalNovoLancamentoAberto.value ||
      modalNovoPeriodoAberto.value) return;
  // Não interfere quando o foco está em input/textarea
  const tag = document.activeElement?.tagName;
  if (tag === "INPUT" || tag === "TEXTAREA") return;

  const lista = lancamentosComSaldo.value;
  if (!lista.length) return;

  if (e.key === "ArrowDown" || e.key === "ArrowUp") {
    e.preventDefault();
    const idx = lista.findIndex((l) => l.id === linhaSelecionadaId.value);
    let novo;
    if (e.key === "ArrowDown") {
      if (idx >= lista.length - 1) return;
      novo = lista[idx + 1];
    } else {
      if (idx <= 0) return;
      novo = lista[idx - 1];
    }
    linhaSelecionadaId.value = novo.id;
    // Scroll para manter a linha visível
    const el = document.querySelector(`[data-id="${novo.id}"]`);
    el?.scrollIntoView({ block: "nearest" });
  }

  if (e.key === "Enter" && linhaSelecionadaId.value !== null) {
    const l = lista.find((l) => l.id === linhaSelecionadaId.value);
    if (l) abrirModalLancamento(l);
  }
}

// ── Modal Lançamento (leitura / edição / exclusão) ────────
const modalLancamentoAberto = ref(false);
const lancamentoSelecionado = ref(null);

function abrirModalLancamento(l) {
  lancamentoSelecionado.value = l;
  modalLancamentoAberto.value = true;
}

function fecharModalLancamento() {
  modalLancamentoAberto.value = false;
  lancamentoSelecionado.value = null;
}

async function onLancamentoAtualizado() {
  await carregarLancamentos();
}

async function onLancamentoExcluido() {
  linhaSelecionadaId.value = null;
  await carregarLancamentos();
}

// ── Modal Novo Lançamento ─────────────────────────────────
const modalNovoLancamentoAberto = ref(false);

function abrirModalNovoLancamento() {
  modalNovoLancamentoAberto.value = true;
}
function fecharModalNovoLancamento() {
  modalNovoLancamentoAberto.value = false;
}

async function onLancamentoCriado() {
  await carregarLancamentos();
}

// ── Modal Novo / Editar Período ───────────────────────────
const modalNovoPeriodoAberto = ref(false);
const periodoParaEditar = ref(null);

function abrirModalNovoPeriodo() {
  periodoParaEditar.value = null;
  modalNovoPeriodoAberto.value = true;
  fecharMenuPeriodos();
}

function abrirModalEditarPeriodo() {
  periodoParaEditar.value = periodoAtivo.value;
  modalNovoPeriodoAberto.value = true;
  fecharMenuPeriodos();
}

function fecharModalNovoPeriodo() {
  modalNovoPeriodoAberto.value = false;
  periodoParaEditar.value = null;
}

async function onPeriodoCriado(periodo) {
  await carregarPeriodos(periodo.id);
}

async function onPeriodoAtualizado() {
  await carregarPeriodos(periodoAtivoId.value);
}

watch(
  () => route.query.acao,
  (acao) => {
    if (acao === "novo") abrirModalNovoPeriodo();
  },
  { immediate: true },
);

onMounted(async () => {
  await carregarCategorias();
  await carregarPeriodos();
  document.addEventListener("click", handleClickFora);
  document.addEventListener("keydown", handleTecladoDatagrid);
});

onUnmounted(() => {
  document.removeEventListener("click", handleClickFora);
  document.removeEventListener("keydown", handleTecladoDatagrid);
});
</script>

<template>
  <div class="main-panel">
    <div class="shell">
      <!-- ── Painel superior ── -->
      <div class="painel-superior" v-if="periodoAtivo || periodos.length === 0">
        <div class="grade-linha sup-linha">
          <div class="sup-nav-status">
            <div class="nav-periodo">
              <button
                class="btn-chevron"
                :disabled="!podePeriodoAnterior()"
                @click="irPeriodoAnterior"
              >
                <ChevronLeft :size="16" />
              </button>
              <span class="periodo-label">{{
                periodoAtivo ? formatarPeriodo(periodoAtivo) : "—"
              }}</span>
              <button
                class="btn-chevron"
                :disabled="!podeProximoPeriodo()"
                @click="irProximoPeriodo"
              >
                <ChevronRight :size="16" />
              </button>
            </div>
            <span
              v-if="periodoAtivo"
              class="badge-status"
              :class="periodoAtivo.status"
            >
              {{ periodoAtivo.status === "aberto" ? "ABERTO" : "CONCILIADO" }}
            </span>
          </div>
          <div class="sup-vazio-valor"></div>
          <div class="sup-saldo">
            <template v-if="periodoAtivo">
              <span class="rotulo-saldo">Saldo inicial:</span>
              <span class="valor-saldo-num">{{
                formatarReais(periodoAtivo.saldo_inicial)
              }}</span>
            </template>
          </div>
          <div class="sup-vazio-comp"></div>
          <div class="sup-acoes">
            <div class="menu-anchor" ref="menuAnchorRef">
              <button class="btn-periodos" @click.stop="toggleMenuPeriodos">
                <span>Períodos</span>
                <ChevronDown
                  :size="14"
                  class="chevron-periodos"
                  :class="{ rotated: menuPeriodosAberto }"
                />
              </button>
              <div v-if="menuPeriodosAberto" class="dropdown-periodos">
                <button class="dropdown-item" @click="abrirModalNovoPeriodo">
                  Novo período
                </button>
                <button
                  class="dropdown-item"
                  :disabled="!periodoAtivo"
                  @click="abrirModalEditarPeriodo"
                >
                  Editar período
                </button>
                <div class="dropdown-separator"></div>
                <button
                  class="dropdown-item"
                  :disabled="!periodoAtivo || periodoAtivo?.status !== 'aberto'"
                >
                  Conciliar
                </button>
                <button
                  class="dropdown-item item-perigo"
                  :disabled="
                    !periodoAtivo || periodoAtivo?.status !== 'conciliado'
                  "
                >
                  Desconciliar
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ── Barra de busca ── -->
      <div class="barra-busca" v-if="periodoAtivo">
        <div class="busca-wrapper">
          <Search :size="14" class="busca-icone" />
          <input
            type="text"
            class="busca-input"
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

      <!-- ── Cabeçalho ── -->
      <div class="grade-cabecalho" v-if="periodoAtivo">
        <div class="grade-linha">
          <div class="cel-drag"></div>
          <div class="cel-data col-header">DATA</div>
          <div class="cel-descricao col-header">DESCRIÇÃO</div>
          <div class="cel-categoria col-header">CATEGORIA</div>
          <div class="cel-valor col-header">VALOR</div>
          <div class="cel-saldo col-header">SALDO</div>
          <div class="cel-comprovantes col-header">COMPROVANTES</div>
          <div class="cel-acoes">
            <button class="btn-novo" @click="abrirModalNovoLancamento">
              +
            </button>
          </div>
        </div>
      </div>

      <!-- ── Datagrid ── -->
      <div class="datagrid">
        <div v-if="erroCarregamento" class="estado-vazio">
          <p class="msg-erro">{{ erroCarregamento }}</p>
        </div>
        <div v-else-if="periodos.length === 0" class="estado-vazio">
          <p>Nenhum período cadastrado.</p>
          <button class="btn-novo-periodo" @click="abrirModalNovoPeriodo">
            Criar primeiro período
          </button>
        </div>
        <template v-else>
          <VueDraggable
            v-model="lancamentosArrastaveis"
            :disabled="!!termoBusca"
            handle=".handle"
            :animation="150"
            ghost-class="linha-ghost"
            :onEnd="onDragEnd"
          >
            <div
              v-for="l in lancamentosComSaldo"
              :key="l.id"
              :data-id="l.id"
              class="grade-linha linha-lancamento"
              :class="{ 'linha-selecionada': l.id === linhaSelecionadaId }"
              @click="selecionarLinha(l)"
              @dblclick="abrirModalLancamento(l)"
            >
              <div class="cel-drag handle">⠿</div>
              <div class="cel-data">{{ formatarData(l.data) }}</div>
              <div class="cel-descricao">
                <span class="descricao-texto">{{ l.descricao }}</span>
                <span v-if="l.pessoa_nome" class="pessoa-nome">{{
                  l.pessoa_nome
                }}</span>
              </div>
              <div class="cel-categoria">{{ l.categoria_nome }}</div>
              <div class="cel-valor" :class="l.tipo_categoria">
                {{ formatarValor(l.valor, l.tipo_categoria) }}
              </div>
              <div
                class="cel-saldo"
                :class="{ negativo: l.saldo_acumulado < 0 }"
              >
                {{ formatarReais(l.saldo_acumulado) }}
              </div>
              <div class="cel-comprovantes">
                <span class="ic-comprovante">🔗</span>
                <span class="comprovante-count">0</span>
              </div>
              <div class="cel-acoes acoes-linha">
                <button
                  class="btn-acao"
                  title="Ver"
                  @click.stop="abrirModalLancamento(l)"
                >
                  <Eye :size="20" />
                </button>
              </div>
            </div>
          </VueDraggable>

          <div v-if="lancamentosComSaldo.length === 0" class="estado-vazio">
            <p>Nenhum lançamento encontrado.</p>
          </div>
        </template>
      </div>

      <!-- ── Painel inferior ── -->
      <div class="painel-inferior" v-if="periodoAtivo">
        <div class="grade-linha">
          <div class="cel-drag"></div>
          <div class="cel-data"></div>
          <div class="cel-descricao"></div>
          <div class="cel-categoria"></div>
          <div class="cel-valor"></div>
          <div
            class="cel-saldo cel-saldo-rodape"
            :class="{ negativo: saldoFinal < 0 }"
          >
            <span class="rotulo-saldo">Saldo final:</span>
            <span class="valor-saldo-num">{{ formatarReais(saldoFinal) }}</span>
          </div>
          <div class="cel-comprovantes"></div>
          <div class="cel-acoes"></div>
        </div>
      </div>
    </div>
  </div>

  <ModalLancamento
    v-if="modalLancamentoAberto && lancamentoSelecionado"
    :lancamento="lancamentoSelecionado"
    @fechar="fecharModalLancamento"
    @atualizado="onLancamentoAtualizado"
    @excluido="onLancamentoExcluido"
  />

  <ModalNovoPeriodo
    v-if="modalNovoPeriodoAberto"
    :periodo="periodoParaEditar"
    @fechar="fecharModalNovoPeriodo"
    @criado="onPeriodoCriado"
    @atualizado="onPeriodoAtualizado"
  />

  <ModalNovoLancamento
    v-if="modalNovoLancamentoAberto && periodoAtivo"
    :periodo-id="periodoAtivo.id"
    @fechar="fecharModalNovoLancamento"
    @criado="onLancamentoCriado"
  />
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
  min-width: 600px;
}

.grade-linha {
  display: grid;
  grid-template-columns:
    28px
    100px
    minmax(120px, 1fr)
    130px
    120px
    160px
    110px
    110px;
  align-items: center;
  min-height: 38px;
}

/* ── Painel superior — células com span ── */
.sup-linha {
  min-height: 44px;
}

.sup-nav-status {
  grid-column: 1 / 5;
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 0 10px;
}

.sup-vazio-valor {
  grid-column: 5;
}

.sup-saldo {
  grid-column: 6;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
  padding: 0 10px;
}

.sup-vazio-comp {
  grid-column: 7;
}

.sup-acoes {
  grid-column: 8;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 10px;
}

/* ── Células do datagrid ── */
.cel-drag {
  padding: 0 4px;
}
.cel-data {
  padding: 0 10px;
  font-size: 13px;
}
.cel-descricao {
  padding: 0 10px;
  min-width: 0;
}
.cel-categoria {
  padding: 0 10px;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cel-valor {
  padding: 0 10px;
  text-align: right;
  font-size: 13px;
}
.cel-saldo {
  padding: 0 10px;
  text-align: right;
  font-size: 13px;
  color: var(--cor-texto-forte);
}
.cel-comprovantes {
  padding: 0 10px;
}
.cel-acoes {
  padding: 0 10px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 4px;
}

/* ── Painel superior ── */
.painel-superior {
  flex-shrink: 0;
  border-bottom: 1px solid var(--cor-borda);
  padding: 4px 0;
}

.nav-periodo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-chevron {
  background: none;
  border: 1px solid var(--cor-borda);
  color: var(--cor-texto);
  width: 28px;
  height: 28px;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  flex-shrink: 0;
  transition:
    border-color 0.15s,
    color 0.15s;
}

.btn-chevron:disabled {
  opacity: 0.3;
  cursor: default;
}
.btn-chevron:not(:disabled):hover {
  border-color: #4a9eff;
  color: #4a9eff;
}

.periodo-label {
  font-size: 14px;
  font-weight: bold;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  min-width: 76px;
  text-align: center;
}

.badge-status {
  font-size: 10px;
  font-weight: bold;
  padding: 3px 10px;
  border-radius: 10px;
  letter-spacing: 0.5px;
  white-space: nowrap;
  flex-shrink: 0;
}

.badge-status.aberto {
  background-color: #1a4a1a;
  color: #66cc66;
}
.badge-status.conciliado {
  background-color: #1a3a5a;
  color: #4a9eff;
}

.rotulo-saldo {
  font-size: 12px;
  color: var(--cor-texto-fraco);
  white-space: nowrap;
  flex-shrink: 0;
}

.valor-saldo-num {
  font-size: 13px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  min-width: 105px;
  text-align: right;
  display: inline-block;
  flex-shrink: 0;
}

/* ── Botão Períodos ── */
.menu-anchor {
  position: relative;
}

.btn-periodos {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: 1px solid var(--cor-borda);
  color: var(--cor-texto);
  padding: 6px 14px;
  border-radius: 4px;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
  transition:
    border-color 0.15s,
    color 0.15s;
}

.btn-periodos:hover {
  border-color: #4a9eff;
  color: var(--cor-texto-forte);
}
.chevron-periodos {
  transition: transform 0.2s;
  flex-shrink: 0;
}
.chevron-periodos.rotated {
  transform: rotate(180deg);
}

.dropdown-periodos {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  min-width: 165px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  z-index: 9999;
  padding: 4px 0;
}

.dropdown-item {
  display: block;
  width: 100%;
  text-align: left;
  background: none;
  border: none;
  color: var(--cor-texto);
  padding: 7px 14px;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
}

.dropdown-item:hover:not(:disabled) {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}
.dropdown-item:disabled {
  opacity: 0.35;
  cursor: default;
}
.dropdown-item.item-perigo {
  color: #cc6666;
}
.dropdown-item.item-perigo:hover:not(:disabled) {
  background-color: #3a1a1a;
  color: #ff6666;
}
.dropdown-separator {
  height: 1px;
  background-color: var(--cor-borda);
  margin: 4px 0;
}

/* ── Barra de busca ── */
.barra-busca {
  flex-shrink: 0;
  padding: 7px 12px;
  border-bottom: 1px solid var(--cor-borda);
}

.busca-wrapper {
  display: flex;
  align-items: center;
  gap: 8px;
  background-color: #111111;
  border: 1px solid var(--cor-borda);
  border-radius: 20px;
  padding: 6px 16px;
  max-width: 520px;
}

.busca-icone {
  color: var(--cor-texto-fraco);
  flex-shrink: 0;
}

.busca-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--cor-texto-forte);
  font-size: 13px;
  font-family: inherit;
  min-width: 0;
}

.busca-input::placeholder {
  color: var(--cor-texto-fraco);
}

.busca-limpar {
  background: none;
  border: none;
  color: var(--cor-texto-fraco);
  cursor: pointer;
  font-size: 12px;
  padding: 0;
  flex-shrink: 0;
}

.busca-limpar:hover {
  color: #cc4444;
}

/* ── Cabeçalho ── */
.grade-cabecalho {
  flex-shrink: 0;
  border-bottom: 1px solid var(--cor-borda);
  padding: 2px 0;
}

.col-header {
  font-size: 10px;
  font-weight: bold;
  color: var(--cor-texto-fraco);
  letter-spacing: 0.5px;
  text-transform: uppercase;
}

.btn-novo {
  background-color: #4a9eff;
  color: #ffffff;
  border: none;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
  font-family: inherit;
  padding: 0;
}

.btn-novo:hover {
  background-color: #3a8eef;
}

/* ── Datagrid ── */
.datagrid {
  flex: 1;
  overflow-y: auto;
}

.linha-lancamento {
  border-bottom: 1px solid #2a2a2a;
  padding: 4px 0;
  transition: background-color 0.1s;
  cursor: pointer;
}

.linha-lancamento:hover {
  background-color: var(--cor-menu-hover);
}

.linha-selecionada {
  background-color: #1a3a5a;
}

.linha-selecionada:hover {
  background-color: #1e4268;
}

.linha-ghost {
  opacity: 0.4;
  background-color: var(--cor-menu-hover);
}

.handle {
  color: var(--cor-texto-fraco);
  cursor: grab;
  font-size: 14px;
  text-align: center;
}

.descricao-texto {
  display: block;
  font-size: 13px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.pessoa-nome {
  display: block;
  font-size: 11px;
  color: var(--cor-texto-fraco);
  margin-top: 1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.cel-valor.entrada {
  color: #66cc66;
}
.cel-valor.saida {
  color: #cc4444;
}
.cel-valor.neutro {
  color: var(--cor-texto);
}

.cel-saldo.negativo {
  color: #cc4444;
}

.ic-comprovante {
  font-size: 12px;
  opacity: 0.5;
  margin-right: 4px;
}
.comprovante-count {
  font-size: 12px;
  color: var(--cor-texto-fraco);
}

.acoes-linha {
  justify-content: flex-end;
}

.btn-acao {
  background: none;
  border: none;
  cursor: pointer;
  color: var(--cor-texto-fraco);
  padding: 4px;
  opacity: 0.6;
  display: flex;
  align-items: center;
}

.btn-acao:hover {
  opacity: 1;
}
.btn-excluir:hover {
  color: #cc4444;
}

/* ── Painel inferior ── */
.painel-inferior {
  flex-shrink: 0;
  border-top: 1px solid var(--cor-borda);
  padding: 5px 0;
}

.cel-saldo-rodape {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 6px;
}

/* ── Estado vazio ── */
.estado-vazio {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--cor-texto-fraco);
  font-size: 13px;
  padding: 40px 0;
}

.msg-erro {
  color: #cc4444;
}

.btn-novo-periodo {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 8px 20px;
  border-radius: 4px;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
}

.btn-novo-periodo:hover {
  opacity: 0.85;
}
</style>
