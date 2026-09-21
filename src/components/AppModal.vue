<script setup>
import { onMounted, onUnmounted } from 'vue'

const props = defineProps({
  titulo: { type: String, required: true },
  largura: { type: String, default: '380px' },
  textoCancelar: { type: String, default: 'Cancelar' },
  textoConfirmar: { type: String, default: 'Salvar' },
})

const emit = defineEmits(['fechar', 'confirmar'])

function handleKeydown(e) {
  if (e.key === 'Escape') emit('fechar')
}

onMounted(() => document.addEventListener('keydown', handleKeydown))
onUnmounted(() => document.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <div class="modal-overlay" @click.self="emit('fechar')">
    <div class="modal" role="dialog" aria-modal="true" :style="{ width: largura }">

      <div class="modal-header">
        <span>{{ titulo }}</span>
        <button class="btn-fechar" @click="emit('fechar')">✕</button>
      </div>

      <div class="modal-body">
        <slot />
      </div>

      <div class="modal-footer">
        <slot name="footer">
          <button class="btn-cancelar" @click="emit('fechar')">{{ textoCancelar }}</button>
          <button class="btn-confirmar" @click="emit('confirmar')">{{ textoConfirmar }}</button>
        </slot>
      </div>

    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9000;
}

.modal {
  background-color: #2a2a2a;
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
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
</style>
