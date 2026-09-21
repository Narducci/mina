<script setup>
import { ref, onMounted, watch } from 'vue'
import { useRoute } from 'vue-router'
import ModalNovoPeriodo from '../components/ModalNovoPeriodo.vue'

const route = useRoute()

const periodos = ref([])
const modalNovoPeriodoAberto = ref(false)

function abrirModalNovoPeriodo() {
  modalNovoPeriodoAberto.value = true
}

function fecharModalNovoPeriodo() {
  modalNovoPeriodoAberto.value = false
}

function onPeriodoCriado(periodo) {
  // TODO: recarregar lista de períodos do banco
  console.log('Período criado:', periodo)
}

// Abre modal se vier ?acao=novo na URL
watch(
  () => route.query.acao,
  (acao) => {
    if (acao === 'novo') abrirModalNovoPeriodo()
  },
  { immediate: true }
)

onMounted(async () => {
  // TODO: carregar períodos do banco
})
</script>

<template>
  <div class="lancamentos-shell">

    <div class="painel-superior">
      Painel superior — Diretos
    </div>

    <div class="datagrid">
      <div v-if="periodos.length === 0" class="sem-periodos">
        <p>Nenhum período cadastrado.</p>
        <button class="btn-novo-periodo" @click="abrirModalNovoPeriodo">
          Criar primeiro período
        </button>
      </div>
    </div>

    <div class="painel-inferior">
      Painel inferior — Diretos
    </div>

    <ModalNovoPeriodo
      v-if="modalNovoPeriodoAberto"
      @fechar="fecharModalNovoPeriodo"
      @criado="onPeriodoCriado"
    />

  </div>
</template>

<style scoped>
.lancamentos-shell {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.painel-superior {
  flex-shrink: 0;
  padding: 8px 12px;
  border-bottom: 1px solid var(--cor-borda);
  background-color: var(--cor-menu-bg);
}

.datagrid {
  flex: 1;
  overflow-y: auto;
  padding: 8px 12px;
}

.painel-inferior {
  flex-shrink: 0;
  padding: 8px 12px;
  border-top: 1px solid var(--cor-borda);
  background-color: var(--cor-menu-bg);
}

.sem-periodos {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: 16px;
  color: var(--cor-texto-fraco);
  font-size: 13px;
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
