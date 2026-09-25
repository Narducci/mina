import { defineStore } from "pinia";
import { ref } from "vue";

export const useToolbarStore = defineStore("toolbar", () => {
  // Botões ativos
  const importarAtivo = ref(false);
  const excluirAtivo = ref(false);
  const exibirAtivo = ref(false);
  const filtroAtivo = ref(false);

  // Estado do filtro de comprovantes
  const filtro = ref("inbox"); // 'inbox' | 'disponiveis' | 'todos'

  // Estado do toggle Exibir (PDF)
  const exibirPdf = ref(false);

  // Callbacks registrados pela view ativa
  const onImportar = ref(null);
  const onExcluir = ref(null);
  const onExibir = ref(null);

  function ativarComprovantes(callbacks = {}) {
    importarAtivo.value = true;
    excluirAtivo.value = true;
    exibirAtivo.value = true;
    filtroAtivo.value = true;
    onImportar.value = callbacks.importar ?? null;
    onExcluir.value = callbacks.excluir ?? null;
    onExibir.value = callbacks.exibir ?? null;
  }

  function desativar() {
    importarAtivo.value = false;
    excluirAtivo.value = false;
    exibirAtivo.value = false;
    filtroAtivo.value = false;
    exibirPdf.value = false;
    filtro.value = "inbox";
    onImportar.value = null;
    onExcluir.value = null;
    onExibir.value = null;
  }

  return {
    importarAtivo,
    excluirAtivo,
    exibirAtivo,
    filtroAtivo,
    filtro,
    exibirPdf,
    onImportar,
    onExcluir,
    onExibir,
    ativarComprovantes,
    desativar,
  };
});
