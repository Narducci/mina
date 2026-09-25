<script setup>
import { useToolbarStore } from "../stores/toolbar.js";
import { FolderOpen, Trash2, Eye } from "@lucide/vue";

const toolbar = useToolbarStore();

function clicarImportar() {
  if (toolbar.onImportar) toolbar.onImportar();
}

function clicarExcluir() {
  if (toolbar.onExcluir) toolbar.onExcluir();
}

function clicarExibir() {
  toolbar.exibirPdf = !toolbar.exibirPdf;
  if (toolbar.onExibir) toolbar.onExibir(toolbar.exibirPdf);
}
</script>

<template>
  <div class="app-toolbar">
    <!-- Botões de ação -->
    <div class="toolbar-grupo">
      <button
        class="toolbar-btn"
        :disabled="!toolbar.importarAtivo"
        @click="clicarImportar"
      >
        <FolderOpen :size="15" />
        <span>Importar...</span>
      </button>
      <button
        class="toolbar-btn"
        :disabled="!toolbar.excluirAtivo"
        @click="clicarExcluir"
      >
        <Trash2 :size="15" />
        <span>Excluir</span>
      </button>
      <button
        class="toolbar-btn"
        :class="{ ativo: toolbar.exibirPdf }"
        :disabled="!toolbar.exibirAtivo"
        @click="clicarExibir"
      >
        <Eye :size="15" />
        <span>Exibir</span>
      </button>
    </div>

    <!-- Separador -->
    <div class="toolbar-sep" v-if="toolbar.filtroAtivo"></div>

    <!-- Filtro de status -->
    <div class="toolbar-filtro" v-if="toolbar.filtroAtivo">
      <button
        class="filtro-btn"
        :class="{ ativo: toolbar.filtro === 'inbox' }"
        @click="toolbar.filtro = 'inbox'"
      >
        Inbox
      </button>
      <button
        class="filtro-btn"
        :class="{ ativo: toolbar.filtro === 'disponiveis' }"
        @click="toolbar.filtro = 'disponiveis'"
      >
        Disponíveis
      </button>
      <button
        class="filtro-btn"
        :class="{ ativo: toolbar.filtro === 'todos' }"
        @click="toolbar.filtro = 'todos'"
      >
        Todos
      </button>
    </div>
  </div>
</template>

<style scoped>
.app-toolbar {
  display: flex;
  align-items: center;
  gap: 6px;
  height: 48px;
  padding: 0 10px;
  background-color: var(--cor-menu-bg);
  border-bottom: 1px solid var(--cor-borda);
  flex-shrink: 0;
}

.toolbar-grupo {
  display: flex;
  align-items: center;
  gap: 4px;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  background: none;
  border: 1px solid transparent;
  color: var(--cor-texto);
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.toolbar-btn:hover:not(:disabled) {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
  border-color: var(--cor-borda);
}

.toolbar-btn:disabled {
  opacity: 0.3;
  cursor: default;
}

.toolbar-btn.ativo {
  background-color: var(--cor-selecao);
  color: var(--cor-texto-forte);
}

.toolbar-sep {
  width: 1px;
  height: 24px;
  background-color: var(--cor-borda);
  margin: 0 6px;
}

/* Filtro segmented control */
.toolbar-filtro {
  display: flex;
  align-items: center;
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  overflow: hidden;
}

.filtro-btn {
  background: none;
  border: none;
  border-right: 1px solid var(--cor-borda);
  color: var(--cor-texto);
  padding: 5px 12px;
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.filtro-btn:last-child {
  border-right: none;
}

.filtro-btn:hover {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}

.filtro-btn.ativo {
  background-color: var(--cor-selecao);
  color: var(--cor-texto-forte);
}
</style>
