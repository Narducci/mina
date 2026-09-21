<script setup>
</script>

<template>
  <div class="lancamentos-shell">
    <div class="painel-superior">Painel superior — Indiretos</div>
    <div class="datagrid">Datagrid — em construção</div>
    <div class="painel-inferior">Painel inferior — Indiretos</div>
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
</style>
