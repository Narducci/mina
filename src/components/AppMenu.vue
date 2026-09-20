<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import {
  FolderOpen, Save, RotateCcw, LogOut,
  CalendarPlus, ArrowDownUp, ArrowLeftRight, CheckSquare,
  Inbox, CheckCircle,
  Users, Landmark, Tag,
  FileText,
  Settings, FolderCog
} from '@lucide/vue'

const aberto = ref(null)

function alternar(menu) {
  aberto.value = aberto.value === menu ? null : menu
}

function hover(menu) {
  if (aberto.value !== null) aberto.value = menu
}

function fechar() {
  aberto.value = null
}

function handleClickFora(e) {
  if (!e.target.closest('.menu-item')) fechar()
}

onMounted(() => {
  document.addEventListener('click', handleClickFora)
  document.addEventListener('keydown', handleHotkey)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickFora)
  document.removeEventListener('keydown', handleHotkey)
})

const emit = defineEmits([
  'arquivo:importar',
  'arquivo:backup',
  'arquivo:restaurar',
  'arquivo:sair',
  'periodo:novo',
  'periodo:diretos',
  'periodo:indiretos',
  'periodo:conciliar',
  'comprovantes:inbox',
  'comprovantes:disponiveis',
  'cadastros:pessoas',
  'cadastros:contas',
  'cadastros:categorias',
  'cadastros:ativar-conta',
  'relatorios:prestacao',
  'configuracoes:geral',
  'configuracoes:pastas',
])

function handleHotkey(e) {
  const ctrl = e.ctrlKey
  const alt  = e.altKey
  const key  = e.key.toLowerCase()

  if (ctrl && key === 'o')  { e.preventDefault(); emit('arquivo:importar') }
  if (ctrl && key === 'b')  { e.preventDefault(); emit('arquivo:backup') }
  if (ctrl && key === 'r')  { e.preventDefault(); emit('arquivo:restaurar') }
  if (alt  && key === 'f4') { e.preventDefault(); emit('arquivo:sair') }

  if (ctrl && key === 'n')  { e.preventDefault(); emit('periodo:novo') }
  if (ctrl && key === 'd')  { e.preventDefault(); emit('periodo:diretos') }
  if (ctrl && key === 'i')  { e.preventDefault(); emit('periodo:indiretos') }
  if (ctrl && key === 'k')  { e.preventDefault(); emit('periodo:conciliar') }

  if (ctrl && key === '1')  { e.preventDefault(); emit('comprovantes:inbox') }
  if (ctrl && key === '2')  { e.preventDefault(); emit('comprovantes:disponiveis') }

  if (ctrl && key === 'p')  { e.preventDefault(); emit('relatorios:prestacao') }
  if (ctrl && key === ',')  { e.preventDefault(); emit('configuracoes:geral') }
}
</script>

<template>
  <nav class="top-menu">

    <!-- Arquivo -->
    <div class="menu-item" :class="{ open: aberto === 'arquivo' }">
      <button class="menu-label" @click="alternar('arquivo')" @mouseenter="hover('arquivo')">Arquivo</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('arquivo:importar'); fechar()">
          <FolderOpen class="icone" /><span>Importar Comprovante</span><span class="hotkey">Ctrl+O</span>
        </a>
        <hr />
        <a href="#" @click.prevent="emit('arquivo:backup'); fechar()">
          <Save class="icone" /><span>Backup</span><span class="hotkey">Ctrl+B</span>
        </a>
        <a href="#" @click.prevent="emit('arquivo:restaurar'); fechar()">
          <RotateCcw class="icone" /><span>Restaurar</span><span class="hotkey">Ctrl+R</span>
        </a>
        <hr />
        <a href="#" @click.prevent="emit('arquivo:sair'); fechar()">
          <LogOut class="icone" /><span>Sair</span><span class="hotkey">Alt+F4</span>
        </a>
      </div>
    </div>

    <!-- Período -->
    <div class="menu-item" :class="{ open: aberto === 'periodo' }">
      <button class="menu-label" @click="alternar('periodo')" @mouseenter="hover('periodo')">Período</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('periodo:novo'); fechar()">
          <CalendarPlus class="icone" /><span>Novo Período</span><span class="hotkey">Ctrl+N</span>
        </a>
        <hr />
        <a href="#" @click.prevent="emit('periodo:diretos'); fechar()">
          <ArrowDownUp class="icone" /><span>Lançamentos Diretos</span><span class="hotkey">Ctrl+D</span>
        </a>
        <a href="#" @click.prevent="emit('periodo:indiretos'); fechar()">
          <ArrowLeftRight class="icone" /><span>Lançamentos Indiretos</span><span class="hotkey">Ctrl+I</span>
        </a>
        <hr />
        <a href="#" @click.prevent="emit('periodo:conciliar'); fechar()">
          <CheckSquare class="icone" /><span>Conciliar</span><span class="hotkey">Ctrl+K</span>
        </a>
      </div>
    </div>

    <!-- Comprovantes -->
    <div class="menu-item" :class="{ open: aberto === 'comprovantes' }">
      <button class="menu-label" @click="alternar('comprovantes')" @mouseenter="hover('comprovantes')">Comprovantes</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('comprovantes:inbox'); fechar()">
          <Inbox class="icone" /><span>Inbox</span><span class="hotkey">Ctrl+1</span>
        </a>
        <a href="#" @click.prevent="emit('comprovantes:disponiveis'); fechar()">
          <CheckCircle class="icone" /><span>Disponíveis</span><span class="hotkey">Ctrl+2</span>
        </a>
      </div>
    </div>

    <!-- Cadastros -->
    <div class="menu-item" :class="{ open: aberto === 'cadastros' }">
      <button class="menu-label" @click="alternar('cadastros')" @mouseenter="hover('cadastros')">Cadastros</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('cadastros:pessoas'); fechar()">
          <Users class="icone" /><span>Pessoas</span>
        </a>
        <a href="#" @click.prevent="emit('cadastros:contas'); fechar()">
          <Landmark class="icone" /><span>Contas</span>
        </a>
        <a href="#" @click.prevent="emit('cadastros:categorias'); fechar()">
          <Tag class="icone" /><span>Categorias</span>
        </a>
        <hr />
        <a href="#" @click.prevent="emit('cadastros:ativar-conta'); fechar()">
          <CheckCircle class="icone" /><span>Ativar Conta</span>
        </a>
      </div>
    </div>

    <!-- Relatórios -->
    <div class="menu-item" :class="{ open: aberto === 'relatorios' }">
      <button class="menu-label" @click="alternar('relatorios')" @mouseenter="hover('relatorios')">Relatórios</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('relatorios:prestacao'); fechar()">
          <FileText class="icone" /><span>Prestação de Contas</span><span class="hotkey">Ctrl+P</span>
        </a>
      </div>
    </div>

    <!-- Configurações -->
    <div class="menu-item" :class="{ open: aberto === 'configuracoes' }">
      <button class="menu-label" @click="alternar('configuracoes')" @mouseenter="hover('configuracoes')">Configurações</button>
      <div class="dropdown">
        <a href="#" @click.prevent="emit('configuracoes:geral'); fechar()">
          <Settings class="icone" /><span>Geral</span><span class="hotkey">Ctrl+,</span>
        </a>
        <a href="#" @click.prevent="emit('configuracoes:pastas'); fechar()">
          <FolderCog class="icone" /><span>Pastas</span>
        </a>
      </div>
    </div>

  </nav>
</template>

<style scoped>
.top-menu {
  background-color: var(--cor-menu-bg);
  display: flex;
  height: 32px;
  border-bottom: 1px solid var(--cor-borda);
  font-size: 12px;
  padding-left: 8px;
  position: relative;
  z-index: 1000;
  flex-shrink: 0;
}

.menu-item {
  position: relative;
  display: flex;
  align-items: center;
}

.menu-label {
  padding: 6px 14px;
  cursor: pointer;
  border-radius: 3px;
  background: none;
  border: none;
  color: var(--cor-texto);
  font-size: inherit;
  font-family: inherit;
}

.menu-item:hover .menu-label,
.menu-item.open .menu-label {
  background-color: var(--cor-menu-hover);
  color: var(--cor-texto-forte);
}

.dropdown {
  display: none;
  position: absolute;
  top: 100%;
  left: 0;
  background-color: var(--cor-menu-bg);
  min-width: 240px;
  box-shadow: 0px 8px 16px rgba(0, 0, 0, 0.4);
  border: 1px solid var(--cor-borda);
  border-radius: 0 0 4px 4px;
  z-index: 2000;
  padding: 4px 0;
}

.menu-item.open .dropdown {
  display: block;
}

.dropdown a {
  color: var(--cor-texto);
  padding: 7px 16px;
  text-decoration: none;
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 12px;
}

.dropdown a:hover {
  background-color: var(--cor-selecao);
  color: var(--cor-texto-forte);
}

.dropdown hr {
  border: 0;
  border-top: 1px solid var(--cor-borda);
  margin: 4px 0;
}

.icone {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.hotkey {
  margin-left: auto;
  color: var(--cor-texto-fraco);
  font-size: 11px;
}
</style>
