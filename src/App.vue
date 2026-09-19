<script setup>
import AppMenu from './components/AppMenu.vue'
import { exit } from '@tauri-apps/plugin-process'
import router from './router/index.js'

async function sair() {
  await exit(0)
}
</script>

<template>
  <div class="app-shell">
    <!-- Barra de título -->
    <div class="titlebar">
      <span class="app-nome" @click="router.push('/')">MINA</span>
    </div>

    <!-- Barra de menus -->
    <AppMenu
      @arquivo:importar="() => {}"
      @arquivo:backup="() => {}"
      @arquivo:restaurar="() => {}"
      @arquivo:sair="sair"
      @periodo:novo="() => {}"
      @periodo:diretos="() => {}"
      @periodo:indiretos="() => {}"
      @periodo:conciliar="() => {}"
      @comprovantes:inbox="() => {}"
      @comprovantes:disponiveis="() => {}"
      @cadastros:pessoas="() => {}"
      @cadastros:contas="() => {}"
      @cadastros:categorias="router.push('/cadastros/categorias')"
      @relatorios:prestacao="() => {}"
      @configuracoes:geral="() => {}"
      @configuracoes:pastas="() => {}"
    />

    <!-- Área de conteúdo -->
    <main class="conteudo">
      <router-view />
    </main>
  </div>
</template>

<style>
:root {
  --cor-bg:           #181818;
  --cor-menu-bg:      #222222;
  --cor-menu-hover:   #333333;
  --cor-borda:        #333333;
  --cor-texto:        #cccccc;
  --cor-texto-forte:  #ffffff;
  --cor-texto-fraco:  #888888;
  --cor-selecao:      #094771;
  --cor-input-bg:     #2a2a2a;

  font-family: "Lucida Sans", "Lucida Sans Unicode", Geneva, Verdana, sans-serif;
  font-size: 13px;
  user-select: none;
}

*, *::before, *::after {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  background-color: var(--cor-bg);
  color: var(--cor-texto);
  height: 100vh;
  overflow: hidden;
}
</style>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.titlebar {
  background-color: #121212;
  height: 32px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  border-bottom: 1px solid #282828;
  flex-shrink: 0;
}

.app-nome {
  font-weight: bold;
  font-size: 12px;
  color: #ffffff;
  letter-spacing: 0.5px;
  cursor: pointer;
}

.app-nome:hover {
  color: var(--cor-selecao);
}

.conteudo {
  flex: 1;
  overflow: hidden;
}
</style>
