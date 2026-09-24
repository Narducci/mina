<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import Database from '@tauri-apps/plugin-sql'
import { ChevronLeft, ChevronRight, ChevronDown, Eye } from '@lucide/vue'
import ModalNovoPeriodo from '../components/ModalNovoPeriodo.vue'
import ModalNovoLancamento from '../components/ModalNovoLancamento.vue'
import ModalLancamento from '../components/ModalLancamento.vue'
import ModalEditarCota from '../components/ModalEditarCota.vue'
import ModalConciliar from '../components/ModalConciliar.vue'

const route = useRoute()

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

// ── Períodos ──────────────────────────────────────────────
const periodos        = ref([])
const periodoAtivoId  = ref(null)
const erroCarregamento = ref('')
const menuPeriodosAberto = ref(false)
const menuAnchorRef   = ref(null)

const periodoAtivo = computed(
  () => periodos.value.find(p => p.id === periodoAtivoId.value) ?? null
)
const indicePeriodoAtivo = computed(() =>
  periodos.value.findIndex(p => p.id === periodoAtivoId.value)
)

function podePeriodoAnterior() { return indicePeriodoAtivo.value > 0 }
function podeProximoPeriodo()  { return indicePeriodoAtivo.value < periodos.value.length - 1 }
function irPeriodoAnterior()   { if (podePeriodoAnterior()) periodoAtivoId.value = periodos.value[indicePeriodoAtivo.value - 1].id }
function irProximoPeriodo()    { if (podeProximoPeriodo())  periodoAtivoId.value = periodos.value[indicePeriodoAtivo.value + 1].id }

function formatarPeriodo(p) {
  const meses = ['Jan','Fev','Mar','Abr','Mai','Jun','Jul','Ago','Set','Out','Nov','Dez']
  return `${meses[p.mes - 1]}/${p.ano}`
}

function toggleMenuPeriodos() { menuPeriodosAberto.value = !menuPeriodosAberto.value }
function fecharMenuPeriodos() { menuPeriodosAberto.value = false }

function handleClickFora(e) {
  if (menuAnchorRef.value && !menuAnchorRef.value.contains(e.target))
    fecharMenuPeriodos()
}

async function carregarPeriodos(idParaAtivar = null) {
  erroCarregamento.value = ''
  try {
    const banco = await getDb()
    periodos.value = await banco.select(
      `SELECT id, mes, ano, status, saldo_inicial, comprovante_id
         FROM periodo WHERE deletado_em IS NULL ORDER BY ano, mes`
    )
    if (periodos.value.length === 0) { periodoAtivoId.value = null; return }
    const alvo = idParaAtivar ?? periodoAtivoId.value
    const existe = periodos.value.some(p => p.id === alvo)
    periodoAtivoId.value = existe ? alvo : periodos.value[periodos.value.length - 1].id
  } catch (err) {
    console.error('Erro ao carregar períodos:', err)
    erroCarregamento.value = 'Erro ao carregar períodos.'
  }
}

// ── Abas (pessoas) ────────────────────────────────────────
const pessoas        = ref([])  // pessoas com movimento no período
const abaAtivaId     = ref(null)

const abaAtiva = computed(() =>
  pessoas.value.find(p => p.id === abaAtivaId.value) ?? null
)

async function carregarPessoas() {
  if (!periodoAtivoId.value) { pessoas.value = []; abaAtivaId.value = null; return }
  try {
    const banco = await getDb()
    // Pessoas com lançamento indireto no período
    const comIndireto = await banco.select(
      `SELECT DISTINCT p.id, p.nome
         FROM pessoa p
         JOIN lancamento l ON l.pessoa_id = p.id
        WHERE l.periodo_id = ? AND l.tipo = 'indireto' AND l.deletado_em IS NULL
          AND p.deletado_em IS NULL`,
      [periodoAtivoId.value]
    )
    // Pessoas com aporte curador nos diretos
    const comAporte = await banco.select(
      `SELECT DISTINCT p.id, p.nome
         FROM pessoa p
         JOIN lancamento l ON l.pessoa_id = p.id
         JOIN categoria c ON c.id = l.categoria_id
        WHERE l.periodo_id = ? AND l.tipo = 'direto' AND l.deletado_em IS NULL
          AND p.deletado_em IS NULL
          AND c.nome = 'Aporte Curador'`,
      [periodoAtivoId.value]
    )
    // Merge sem duplicatas
    const mapa = new Map()
    for (const p of [...comIndireto, ...comAporte]) mapa.set(p.id, p)
    pessoas.value = [...mapa.values()].sort((a, b) => a.nome.localeCompare(b.nome))

    if (pessoas.value.length === 0) { abaAtivaId.value = null; return }
    const existe = pessoas.value.some(p => p.id === abaAtivaId.value)
    if (!existe) abaAtivaId.value = pessoas.value[0].id
  } catch (err) {
    console.error('Erro ao carregar pessoas:', err)
  }
}

// ── Lançamentos + linhas fixas ────────────────────────────
const lancamentosNormais = ref([])
const cotaLancamento     = ref(null)  // registro da linha de cota no banco
const depositoDirecto    = ref(0)     // calculado em memória
const categorias         = ref([])

async function carregarCategorias() {
  try {
    const banco = await getDb()
    categorias.value = await banco.select(
      `SELECT id, nome, tipo FROM categoria WHERE deletado_em IS NULL AND ativa = 1`
    )
  } catch (err) {
    console.error('Erro ao carregar categorias:', err)
  }
}

async function garantirLinhasCota() {
  if (!periodoAtivoId.value || !abaAtivaId.value) return
  try {
    const banco = await getDb()
    const contas = await banco.select(
      `SELECT id FROM conta WHERE ativa = 1 AND deletado_em IS NULL LIMIT 1`
    )
    const contaId = contas.length > 0 ? contas[0].id : null
    if (!contaId) return

    // Busca ids das categorias fixas
    const cats = await banco.select(
      `SELECT id, nome FROM categoria WHERE nome IN ('Aporte Período', 'Aporte Realizado') AND deletado_em IS NULL`
    )
    const catAportePeriodo   = cats.find(c => c.nome === 'Aporte Período')?.id
    const catAporteRealizado = cats.find(c => c.nome === 'Aporte Realizado')?.id

    if (!catAportePeriodo || !catAporteRealizado) {
      console.error('Categorias fixas do sistema não encontradas')
      return
    }

    // Busca a linha de cota (ordem 0)
    const rows = await banco.select(
      `SELECT id, valor FROM lancamento
        WHERE periodo_id = ? AND tipo = 'indireto' AND pessoa_id = ?
          AND ordem = 0 AND deletado_em IS NULL`,
      [periodoAtivoId.value, abaAtivaId.value]
    )
    if (rows.length === 0) {
      const result = await banco.execute(
        `INSERT INTO lancamento (periodo_id, categoria_id, conta_id, pessoa_id, data, descricao, tipo, valor, ordem)
         VALUES (?, ?, ?, ?, date('now'), 'Cota do período', 'indireto', 0, 0)`,
        [periodoAtivoId.value, catAportePeriodo, contaId, abaAtivaId.value]
      )
      cotaLancamento.value = { id: result.lastInsertId, valor: 0 }
    } else {
      cotaLancamento.value = rows[0]
    }
  } catch (err) {
    console.error('Erro ao garantir linha de cota:', err)
  }
}

async function carregarLancamentos() {
  if (!periodoAtivoId.value || !abaAtivaId.value) {
    lancamentosNormais.value = []
    cotaLancamento.value = null
    depositoDirecto.value = 0
    return
  }
  try {
    const banco = await getDb()

    // Garante que a linha de cota existe
    await garantirLinhasCota()

    // Lançamentos normais (ordem >= 3)
    lancamentosNormais.value = await banco.select(
      `SELECT l.id, l.data, l.descricao, l.valor, l.ordem,
              l.categoria_id, l.pessoa_id, l.comprovante_id,
              c.nome AS categoria_nome, c.tipo AS categoria_tipo
         FROM lancamento l
         JOIN categoria c ON c.id = l.categoria_id
        WHERE l.periodo_id = ? AND l.tipo = 'indireto'
          AND l.pessoa_id = ? AND l.ordem >= 3
          AND l.deletado_em IS NULL
        ORDER BY l.ordem`,
      [periodoAtivoId.value, abaAtivaId.value]
    )

    // Depósito direto: soma dos lançamentos diretos com categoria "Aporte Curador" desta pessoa
    const aportes = await banco.select(
      `SELECT COALESCE(SUM(l.valor), 0) AS total
         FROM lancamento l
         JOIN categoria c ON c.id = l.categoria_id
        WHERE l.periodo_id = ? AND l.tipo = 'direto'
          AND l.pessoa_id = ? AND c.nome = 'Aporte Curador'
          AND l.deletado_em IS NULL`,
      [periodoAtivoId.value, abaAtivaId.value]
    )
    depositoDirecto.value = aportes[0]?.total ?? 0
  } catch (err) {
    console.error('Erro ao carregar lançamentos:', err)
  }
}

// ── Linhas do datagrid com saldo acumulado ────────────────
const linhasDatagrid = computed(() => {
  const cota     = cotaLancamento.value?.valor ?? 0
  const deposito = depositoDirecto.value

  // Monta linhas fixas
  const fixas = [
    {
      id:             `cota-${abaAtivaId.value}`,
      fixa:           true,
      tipoFixa:       'cota',
      descricao:      'Cota do período',
      categoria_nome: 'Sistema',
      valor:          cota,
      tipo_categoria: 'saida',   // entra negativo no cálculo
      saldo_zero:     true,      // saldo exibido como zero nessa linha
    },
    {
      id:             `deposito-${abaAtivaId.value}`,
      fixa:           true,
      tipoFixa:       'deposito',
      descricao:      'Depósito direto',
      categoria_nome: 'Sistema',
      valor:          deposito,
      tipo_categoria: 'entrada',
    },
  ]

  // Saldo acumulado linha a linha
  let acumulado = 0
  const todas = [...fixas, ...lancamentosNormais.value.map(l => {
    return { ...l, fixa: false, tipo_categoria: 'entrada' }
  })]

  return todas.map(l => {
    if (l.tipo_categoria === 'entrada') acumulado += l.valor
    else if (l.tipo_categoria === 'saida') acumulado -= l.valor
    const saldo = l.saldo_zero ? 0 : acumulado
    return { ...l, saldo_acumulado: saldo }
  })
})

// ── Totais do rodapé ──────────────────────────────────────
const compromisso = computed(() => cotaLancamento.value?.valor ?? 0)

const realizado = computed(() =>
  depositoDirecto.value + lancamentosNormais.value.reduce((s, l) => s + l.valor, 0)
)

const carryOver = computed(() => realizado.value - compromisso.value)

// ── Formatação ────────────────────────────────────────────
function formatarData(iso) {
  if (!iso) return '—'
  const [a, m, d] = iso.split('-')
  return `${d}/${m}/${a}`
}

function formatarValor(valor, tipo) {
  const abs = Math.abs(valor).toLocaleString('pt-BR', { minimumFractionDigits: 2 })
  if (tipo === 'entrada') return `+R$ ${abs}`
  if (tipo === 'saida')   return `-R$ ${abs}`
  return `R$ ${abs}`
}

function formatarReais(valor) {
  const abs = Math.abs(valor).toLocaleString('pt-BR', { minimumFractionDigits: 2 })
  if (valor < 0) return `(R$ ${abs})`
  return `R$ ${abs}`
}

// ── Watches ───────────────────────────────────────────────
watch(periodoAtivoId, async () => {
  await carregarPessoas()
})

watch(abaAtivaId, async () => {
  await carregarLancamentos()
})

// ── Modal Lançamento ──────────────────────────────────────
const modalLancamentoAberto    = ref(false)
const lancamentoSelecionado    = ref(null)
const linhaSelecionadaId       = ref(null)

function abrirModalLancamento(l) {
  lancamentoSelecionado.value = l
  modalLancamentoAberto.value = true
}

function fecharModalLancamento() {
  modalLancamentoAberto.value = false
  lancamentoSelecionado.value = null
}

async function onLancamentoAtualizado() { await carregarLancamentos() }
async function onLancamentoExcluido()   {
  linhaSelecionadaId.value = null
  await carregarLancamentos()
  await carregarPessoas()
}

function selecionarLinha(l) {
  if (l.fixa) return
  linhaSelecionadaId.value = l.id
}

function handleTecladoDatagrid(e) {
  if (modalLancamentoAberto.value || modalNovoLancamentoAberto.value ||
      modalNovoPeriodoAberto.value || modalEditarCotaAberto.value ||
      modalConciliarAberto.value) return
  const tag = document.activeElement?.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA') return

  const lista = linhasDatagrid.value.filter(l => !l.fixa)
  if (!lista.length) return

  if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    e.preventDefault()
    const idx = lista.findIndex(l => l.id === linhaSelecionadaId.value)
    if (e.key === 'ArrowDown') {
      if (idx >= lista.length - 1) return
      linhaSelecionadaId.value = lista[idx + 1].id
    } else {
      if (idx <= 0) return
      linhaSelecionadaId.value = lista[idx - 1].id
    }
    const el = document.querySelector(`[data-id="${linhaSelecionadaId.value}"]`)
    el?.scrollIntoView({ block: 'nearest' })
  }

  if (e.key === 'Enter' && linhaSelecionadaId.value !== null) {
    const l = lista.find(l => l.id === linhaSelecionadaId.value)
    if (l) abrirModalLancamento(l)
  }
}

// ── Modal Novo Lançamento ─────────────────────────────────
const modalNovoLancamentoAberto = ref(false)

function abrirModalNovoLancamento() { modalNovoLancamentoAberto.value = true }
function fecharModalNovoLancamento() { modalNovoLancamentoAberto.value = false }

async function onLancamentoCriado() {
  await carregarLancamentos()
  await carregarPessoas()
}

// ── Modal Editar Cota ─────────────────────────────────────
const modalEditarCotaAberto = ref(false)

function abrirModalEditarCota() {
  if (!cotaLancamento.value) return
  modalEditarCotaAberto.value = true
}

function fecharModalEditarCota() { modalEditarCotaAberto.value = false }

async function onCotaAtualizada() { await carregarLancamentos() }

// ── Modal Novo / Editar Período ───────────────────────────
const modalNovoPeriodoAberto = ref(false)
const periodoParaEditar      = ref(null)

function abrirModalNovoPeriodo() {
  periodoParaEditar.value = null
  modalNovoPeriodoAberto.value = true
  fecharMenuPeriodos()
}

function abrirModalEditarPeriodo() {
  periodoParaEditar.value = periodoAtivo.value
  modalNovoPeriodoAberto.value = true
  fecharMenuPeriodos()
}

function fecharModalNovoPeriodo() {
  modalNovoPeriodoAberto.value = false
  periodoParaEditar.value = null
}

async function onPeriodoCriado(periodo) { await carregarPeriodos(periodo.id) }
async function onPeriodoAtualizado()    { await carregarPeriodos(periodoAtivoId.value) }

// ── Modal Conciliar ───────────────────────────────────────
const modalConciliarAberto = ref(false)
const modoConciliar        = ref('conciliar')

function abrirModalConciliar(modo) {
  modoConciliar.value = modo
  modalConciliarAberto.value = true
  fecharMenuPeriodos()
}

function fecharModalConciliar() { modalConciliarAberto.value = false }

async function onConciliado()    { await carregarPeriodos(periodoAtivoId.value) }
async function onDesconciliado() { await carregarPeriodos(periodoAtivoId.value) }

// ── Watch query para novo período ─────────────────────────
watch(
  () => route.query.acao,
  (acao) => { if (acao === 'novo') abrirModalNovoPeriodo() },
  { immediate: true }
)

// ── Lifecycle ─────────────────────────────────────────────
onMounted(async () => {
  await carregarCategorias()
  await carregarPeriodos()
  document.addEventListener('click', handleClickFora)
  document.addEventListener('keydown', handleTecladoDatagrid)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickFora)
  document.removeEventListener('keydown', handleTecladoDatagrid)
})
</script>

<template>
  <div class="main-panel">
    <div class="shell">

      <!-- ── Painel superior ── -->
      <div class="painel-superior" v-if="periodoAtivo || periodos.length === 0">
        <div class="sup-linha">
          <div class="sup-nav-status">
            <div class="nav-periodo">
              <button class="btn-chevron" :disabled="!podePeriodoAnterior()" @click="irPeriodoAnterior">
                <ChevronLeft :size="16" />
              </button>
              <span class="periodo-label">{{ periodoAtivo ? formatarPeriodo(periodoAtivo) : '—' }}</span>
              <button class="btn-chevron" :disabled="!podeProximoPeriodo()" @click="irProximoPeriodo">
                <ChevronRight :size="16" />
              </button>
            </div>
            <span v-if="periodoAtivo" class="badge-status" :class="periodoAtivo.status">
              {{ periodoAtivo.status === 'aberto' ? 'ABERTO' : 'CONCILIADO' }}
            </span>
          </div>
          <div class="sup-acoes">
            <div class="menu-anchor" ref="menuAnchorRef">
              <button class="btn-periodos" @click.stop="toggleMenuPeriodos">
                <span>Períodos</span>
                <ChevronDown :size="14" class="chevron-periodos" :class="{ rotated: menuPeriodosAberto }" />
              </button>
              <div v-if="menuPeriodosAberto" class="dropdown-periodos">
                <button class="dropdown-item" @click="abrirModalNovoPeriodo">Novo período</button>
                <button class="dropdown-item" :disabled="!periodoAtivo" @click="abrirModalEditarPeriodo">Editar período</button>
                <div class="dropdown-separator"></div>
                <button class="dropdown-item" :disabled="!periodoAtivo || periodoAtivo?.status !== 'aberto'" @click="abrirModalConciliar('conciliar')">Conciliar</button>
                <button class="dropdown-item item-perigo" :disabled="!periodoAtivo || periodoAtivo?.status !== 'conciliado'" @click="abrirModalConciliar('desconciliar')">Desconciliar</button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- ── Abas de pessoas ── -->
      <div class="abas" v-if="periodoAtivo && pessoas.length > 0">
        <button
          v-for="p in pessoas"
          :key="p.id"
          class="aba"
          :class="{ ativa: p.id === abaAtivaId }"
          @click="abaAtivaId = p.id"
        >
          {{ p.nome }}
        </button>
      </div>

      <!-- ── Cabeçalho ── -->
      <div class="grade-cabecalho" v-if="periodoAtivo">
        <div class="grade-linha">
          <div class="cel-data col-header">DATA</div>
          <div class="cel-descricao col-header">DESCRIÇÃO</div>
          <div class="cel-categoria col-header">CATEGORIA</div>
          <div class="cel-valor col-header">VALOR</div>
          <div class="cel-saldo col-header">SALDO</div>
          <div class="cel-comprovantes col-header">COMPROVANTES</div>
          <div class="cel-acoes">
            <button class="btn-novo" @click="abrirModalNovoLancamento">+</button>
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
          <button class="btn-novo-periodo" @click="abrirModalNovoPeriodo">Criar primeiro período</button>
        </div>
        <div v-else-if="pessoas.length === 0" class="estado-vazio">
          <p>Nenhum movimento indireto neste período.</p>
        </div>
        <template v-else>
          <div
            v-for="l in linhasDatagrid"
            :key="l.id"
            :data-id="l.fixa ? undefined : l.id"
            class="grade-linha linha-lancamento"
            :class="{
              'linha-fixa': l.fixa,
              'linha-selecionada': !l.fixa && l.id === linhaSelecionadaId,
            }"
            @click="selecionarLinha(l)"
            @dblclick="!l.fixa && abrirModalLancamento(l)"
          >
            <div class="cel-data">
              {{ l.fixa ? '—' : formatarData(l.data) }}
            </div>
            <div class="cel-descricao">
              <span class="descricao-texto" :class="{ 'texto-italico': l.fixa }">
                {{ l.descricao }}
              </span>
            </div>
            <div class="cel-categoria" :class="{ 'texto-italico': l.fixa }">
              {{ l.categoria_nome }}
            </div>
            <div class="cel-valor" :class="l.tipo_categoria">
              <span v-if="l.fixa && l.tipoFixa === 'cota'" class="link-editar" @click.stop="abrirModalEditarCota">
                {{ formatarReais(l.valor) }}
              </span>
              <span v-else>
                {{ l.fixa && l.valor === 0 ? '—' : formatarValor(l.valor, l.tipo_categoria) }}
              </span>
            </div>
            <div class="cel-saldo" :class="{ negativo: l.saldo_acumulado < 0 }">
              {{ formatarReais(l.saldo_acumulado) }}
            </div>
            <div class="cel-comprovantes">
              <template v-if="!l.fixa">
                <span class="ic-comprovante">🔗</span>
                <span class="comprovante-count">0</span>
              </template>
            </div>
            <div class="cel-acoes acoes-linha">
              <button v-if="!l.fixa" class="btn-acao" title="Ver" @click.stop="abrirModalLancamento(l)">
                <Eye :size="20" />
              </button>
            </div>
          </div>

          <div v-if="linhasDatagrid.length === 0" class="estado-vazio">
            <p>Nenhum lançamento encontrado.</p>
          </div>
        </template>
      </div>

      <!-- ── Painel inferior ── -->
      <div class="painel-inferior" v-if="periodoAtivo && abaAtiva">
        <div class="grade-linha rodape-linha">
          <div class="cel-data"></div>
          <div class="cel-descricao"></div>
          <div class="cel-categoria"></div>
          <div class="cel-valor rodape-valores">
            <div class="rodape-item">
              <span class="rodape-label">COMPROMISSO</span>
              <span class="rodape-valor">{{ formatarReais(compromisso) }}</span>
            </div>
            <div class="rodape-item">
              <span class="rodape-label">REALIZADO</span>
              <span class="rodape-valor">{{ formatarReais(realizado) }}</span>
            </div>
          </div>
          <div class="cel-saldo rodape-carryover">
            <span class="rodape-label">CARRY-OVER</span>
            <span class="rodape-valor" :class="{ positivo: carryOver > 0, negativo: carryOver < 0 }">
              {{ formatarReais(carryOver) }}
            </span>
          </div>
          <div class="cel-comprovantes"></div>
          <div class="cel-acoes"></div>
        </div>
      </div>

    </div>
  </div>

  <!-- ── Modais ── -->
  <ModalLancamento
    v-if="modalLancamentoAberto && lancamentoSelecionado"
    :lancamento="lancamentoSelecionado"
    @fechar="fecharModalLancamento"
    @atualizado="onLancamentoAtualizado"
    @excluido="onLancamentoExcluido"
  />

  <ModalNovoLancamento
    v-if="modalNovoLancamentoAberto && periodoAtivo"
    :periodo-id="periodoAtivo.id"
    tipo="indireto"
    :pessoa-id="abaAtivaId"
    @fechar="fecharModalNovoLancamento"
    @criado="onLancamentoCriado"
  />

  <ModalEditarCota
    v-if="modalEditarCotaAberto && cotaLancamento"
    :lancamento-id="cotaLancamento.id"
    :valor-atual="cotaLancamento.valor"
    @fechar="fecharModalEditarCota"
    @atualizado="onCotaAtualizada"
  />

  <ModalNovoPeriodo
    v-if="modalNovoPeriodoAberto"
    :periodo="periodoParaEditar"
    @fechar="fecharModalNovoPeriodo"
    @criado="onPeriodoCriado"
    @atualizado="onPeriodoAtualizado"
  />

  <ModalConciliar
    v-if="modalConciliarAberto && periodoAtivo"
    :periodo="periodoAtivo"
    :modo="modoConciliar"
    @fechar="fecharModalConciliar"
    @conciliado="onConciliado"
    @desconciliado="onDesconciliado"
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

/* ── Grade ── */
.grade-linha {
  display: grid;
  grid-template-columns:
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

/* ── Painel superior ── */
.painel-superior {
  flex-shrink: 0;
  border-bottom: 1px solid var(--cor-borda);
  padding: 4px 0;
}

.sup-linha {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 10px;
  min-height: 44px;
}

.sup-nav-status {
  display: flex;
  align-items: center;
  gap: 14px;
}

.sup-acoes {
  display: flex;
  align-items: center;
}

.nav-periodo {
  display: flex;
  align-items: center;
  gap: 8px;
}

.periodo-label {
  font-size: 14px;
  font-weight: bold;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  min-width: 76px;
  text-align: center;
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
  transition: border-color 0.15s, color 0.15s;
}

.btn-chevron:disabled { opacity: 0.3; cursor: default; }
.btn-chevron:not(:disabled):hover { border-color: #4a9eff; color: #4a9eff; }

.badge-status {
  font-size: 10px;
  font-weight: bold;
  padding: 3px 10px;
  border-radius: 10px;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.badge-status.aberto     { background-color: #1a4a1a; color: #66cc66; }
.badge-status.conciliado { background-color: #1a3a5a; color: #4a9eff; }

/* ── Menu Períodos ── */
.menu-anchor { position: relative; }

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
  transition: border-color 0.15s, color 0.15s;
}

.btn-periodos:hover { border-color: #4a9eff; color: var(--cor-texto-forte); }

.chevron-periodos { transition: transform 0.2s; flex-shrink: 0; }
.chevron-periodos.rotated { transform: rotate(180deg); }

.dropdown-periodos {
  position: absolute;
  top: calc(100% + 4px);
  right: 0;
  min-width: 165px;
  background-color: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.5);
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

.dropdown-item:hover:not(:disabled) { background-color: var(--cor-menu-hover); color: var(--cor-texto-forte); }
.dropdown-item:disabled { opacity: 0.35; cursor: default; }
.dropdown-item.item-perigo { color: #cc6666; }
.dropdown-item.item-perigo:hover:not(:disabled) { background-color: #3a1a1a; color: #ff6666; }

.dropdown-separator { height: 1px; background-color: var(--cor-borda); margin: 4px 0; }

/* ── Abas ── */
.abas {
  display: flex;
  flex-shrink: 0;
  border-bottom: 1px solid var(--cor-borda);
  padding: 0 12px;
  gap: 4px;
}

.aba {
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  color: var(--cor-texto-fraco);
  padding: 8px 14px;
  font-size: 13px;
  font-family: inherit;
  cursor: pointer;
  white-space: nowrap;
  transition: color 0.15s, border-color 0.15s;
}

.aba:hover { color: var(--cor-texto-forte); }
.aba.ativa { color: #4a9eff; border-bottom-color: #4a9eff; font-weight: bold; }

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

.btn-novo:hover { background-color: #3a8eef; }

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

.linha-lancamento:hover { background-color: var(--cor-menu-hover); }

.linha-fixa { cursor: default; }
.linha-fixa:hover { background-color: transparent; }

.linha-selecionada { background-color: #1a3a5a; }
.linha-selecionada:hover { background-color: #1e4268; }

/* ── Células ── */
.cel-data         { padding: 0 10px; font-size: 13px; }
.cel-descricao    { padding: 0 10px; min-width: 0; }
.cel-categoria    { padding: 0 10px; font-size: 13px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.cel-valor        { padding: 0 10px; text-align: right; font-size: 13px; }
.cel-saldo        { padding: 0 10px; text-align: right; font-size: 13px; color: var(--cor-texto-forte); }
.cel-comprovantes { padding: 0 10px; }
.cel-acoes        { padding: 0 10px; display: flex; align-items: center; justify-content: flex-end; gap: 4px; }

.descricao-texto {
  display: block;
  font-size: 13px;
  color: var(--cor-texto-forte);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.texto-italico { font-style: italic; color: var(--cor-texto-fraco) !important; }

.cel-valor.entrada { color: #66cc66; }
.cel-valor.saida   { color: #cc4444; }
.cel-valor.neutro  { color: var(--cor-texto); }
.cel-saldo.negativo { color: #cc4444; }

.link-editar {
  cursor: pointer;
  text-decoration: underline dotted;
}

.link-editar:hover { opacity: 0.8; }

.ic-comprovante    { font-size: 12px; opacity: 0.5; margin-right: 4px; }
.comprovante-count { font-size: 12px; color: var(--cor-texto-fraco); }

.acoes-linha { justify-content: flex-end; }

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

.btn-acao:hover { opacity: 1; }

/* ── Painel inferior ── */
.painel-inferior {
  flex-shrink: 0;
  border-top: 1px solid var(--cor-borda);
  padding: 4px 0;
}

.rodape-linha {
  min-height: 52px;
}

.rodape-valores {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 4px;
  padding: 6px 10px;
}

.rodape-carryover {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: flex-end;
  gap: 4px;
  padding: 6px 10px;
}

.rodape-item {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
}

.rodape-label {
  font-size: 10px;
  font-weight: bold;
  color: var(--cor-texto-fraco);
  letter-spacing: 0.5px;
}

.rodape-valor {
  font-size: 13px;
  color: var(--cor-texto-forte);
  text-align: right;
  min-width: 100px;
}

.rodape-valor.positivo { color: #66cc66; }
.rodape-valor.negativo { color: #cc4444; }

/* ── Estados ── */
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

.msg-erro { color: #cc4444; }

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

.btn-novo-periodo:hover { opacity: 0.85; }
</style>
