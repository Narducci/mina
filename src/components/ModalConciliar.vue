<script setup>
import { ref, onMounted } from 'vue'
import Database from '@tauri-apps/plugin-sql'
import AppModal from './AppModal.vue'

const props = defineProps({
  periodo: { type: Object, required: true },
  modo:    { type: String, required: true }, // 'conciliar' | 'desconciliar'
})

const emit = defineEmits(['fechar', 'conciliado', 'desconciliado'])

let db = null
async function getDb() {
  if (!db) db = await Database.load('sqlite:mina.db')
  return db
}

const carregando = ref(true)
const erro       = ref('')   // mensagem de pré-condição não atendida
const salvando   = ref(false)

const meses = [
  'Jan','Fev','Mar','Abr','Mai','Jun',
  'Jul','Ago','Set','Out','Nov','Dez',
]

function formatarPeriodo(p) {
  return `${meses[p.mes - 1]}/${p.ano}`
}

const titulo = props.modo === 'conciliar' ? 'Conciliar Período' : 'Desconciliar Período'

onMounted(async () => {
  try {
    const banco = await getDb()
    const p = props.periodo

    if (props.modo === 'conciliar') {
      // Pré-condição 1: extrato obrigatório
      if (!p.comprovante_id) {
        erro.value = `O período ${formatarPeriodo(p)} não possui extrato anexado. Anexe o extrato antes de conciliar.`
        return
      }
      // Pré-condição 2: período anterior deve estar conciliado (se existir)
      const anterior = await banco.select(
        `SELECT status FROM periodo
         WHERE deletado_em IS NULL AND (ano < ? OR (ano = ? AND mes < ?))
         ORDER BY ano DESC, mes DESC LIMIT 1`,
        [p.ano, p.ano, p.mes]
      )
      if (anterior.length > 0 && anterior[0].status !== 'conciliado') {
        erro.value = `Existe um período anterior ainda em aberto. Concilie-o antes de prosseguir.`
        return
      }
    } else {
      // Desconciliar — pré-condição: sem período posterior conciliado
      const posterior = await banco.select(
        `SELECT id FROM periodo
         WHERE deletado_em IS NULL AND status = 'conciliado'
           AND (ano > ? OR (ano = ? AND mes > ?))
         LIMIT 1`,
        [p.ano, p.ano, p.mes]
      )
      if (posterior.length > 0) {
        erro.value = `Existe um período posterior já conciliado. Desconcilie-o primeiro.`
        return
      }
    }
  } catch (err) {
    console.error('Erro ao verificar pré-condições:', err)
    erro.value = 'Erro ao verificar condições. Tente novamente.'
  } finally {
    carregando.value = false
  }
})

async function confirmar() {
  salvando.value = true
  try {
    const banco = await getDb()
    const novoStatus = props.modo === 'conciliar' ? 'conciliado' : 'aberto'
    await banco.execute(
      'UPDATE periodo SET status = ? WHERE id = ?',
      [novoStatus, props.periodo.id]
    )
    if (props.modo === 'conciliar') emit('conciliado')
    else emit('desconciliado')
    emit('fechar')
  } catch (err) {
    console.error('Erro ao atualizar período:', err)
  } finally {
    salvando.value = false
  }
}
</script>

<template>
  <AppModal
    :titulo="titulo"
    @fechar="emit('fechar')"
    @confirmar="confirmar"
  >
    <!-- Corpo -->
    <div v-if="carregando" class="msg-info">Verificando...</div>

    <div v-else-if="erro" class="msg-erro">{{ erro }}</div>

    <div v-else class="msg-info">
      <template v-if="modo === 'conciliar'">
        Deseja conciliar o período <strong>{{ formatarPeriodo(periodo) }}</strong>?
        Após conciliado, os lançamentos só poderão ser visualizados.
      </template>
      <template v-else>
        Deseja desconciliar o período <strong>{{ formatarPeriodo(periodo) }}</strong>?
        O período voltará para o status <strong>Aberto</strong>.
      </template>
    </div>

    <!-- Footer: só Fechar quando há erro ou ainda carregando; padrão quando pode confirmar -->
    <template #footer>
      <template v-if="carregando || erro">
        <button class="btn-fechar-acao" @click="emit('fechar')">Fechar</button>
      </template>
      <template v-else>
        <button class="btn-cancelar" @click="emit('fechar')">Cancelar</button>
        <button class="btn-confirmar" :disabled="salvando" @click="confirmar">
          {{ modo === 'conciliar' ? 'Conciliar' : 'Desconciliar' }}
        </button>
      </template>
    </template>

  </AppModal>
</template>

<style scoped>
.msg-info {
  font-size: 13px;
  color: var(--cor-texto);
  line-height: 1.5;
}

.msg-erro {
  font-size: 13px;
  color: #cc4444;
  line-height: 1.5;
}

.btn-fechar-acao {
  background-color: transparent;
  color: var(--cor-texto);
  border: 1px solid var(--cor-borda);
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

.btn-confirmar {
  background-color: var(--cor-selecao);
  color: #ffffff;
  border: none;
  padding: 6px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.btn-confirmar:disabled {
  opacity: 0.5;
  cursor: default;
}
</style>
