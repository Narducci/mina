<script setup>
import { ref, onMounted } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import Database from "@tauri-apps/plugin-sql";
import AppSelect from "../components/AppSelect.vue";

const config = ref({
  pasta_raiz_comprovantes: "",
  pasta_raiz_avatares: "",
  tema: "sistema",
  versao_schema: null,
});

const opcoesThema = [
  { value: "sistema", label: "Sistema" },
  { value: "claro", label: "Claro" },
  { value: "escuro", label: "Escuro" },
];

function abreviarPath(path) {
  if (!path) return "";
  const partes = path.split("/").filter(Boolean);
  if (partes.length <= 3) return "/" + partes.join("/");
  return "/" + partes[0] + "/" + partes[1] + "/.../" + partes[partes.length - 1];
}

onMounted(async () => {
  try {
    const db = await Database.load("sqlite:mina.db");
    const rows = await db.select("SELECT * FROM configuracao WHERE id = 1");
    if (rows.length > 0) {
      config.value = { ...config.value, ...rows[0] };
    }
  } catch (e) {
    console.error("Erro ao carregar configurações:", e);
  }
});

async function salvar() {
  try {
    const db = await Database.load("sqlite:mina.db");
    await db.execute(
      `INSERT OR REPLACE INTO configuracao
         (id, pasta_raiz_comprovantes, pasta_raiz_avatares, tema, versao_schema)
       VALUES (1, $1, $2, $3, $4)`,
      [
        config.value.pasta_raiz_comprovantes || null,
        config.value.pasta_raiz_avatares || null,
        config.value.tema,
        config.value.versao_schema || null,
      ],
    );
  } catch (e) {
    console.error("Erro ao salvar configurações:", e);
  }
}

async function selecionarPasta(campo) {
  try {
    const path = await open({ directory: true, multiple: false });
    if (!path) return;
    config.value[campo] = path;
    await salvar();
    if (campo === "pasta_raiz_comprovantes") {
      await invoke("iniciar_watcher", { path });
    }
  } catch (e) {
    console.error("Erro ao selecionar pasta:", e);
  }
}

async function onTemaMudou(valor) {
  config.value.tema = valor;
  await salvar();
}
</script>

<template>
  <div class="main-panel">
    <div class="shell">
      <!-- PASTAS -->
      <section class="secao">
        <h2 class="secao-titulo">Pastas</h2>

        <div class="campo-pasta">
          <label class="campo-label">Pasta de comprovantes</label>
          <div class="pasta-row">
            <div class="pasta-path" :title="config.pasta_raiz_comprovantes">
              {{ abreviarPath(config.pasta_raiz_comprovantes) || "(não configurado)" }}
            </div>
            <button
              class="btn-selecionar"
              @click="selecionarPasta('pasta_raiz_comprovantes')"
            >
              Selecionar
            </button>
          </div>
        </div>

        <div class="campo-pasta">
          <label class="campo-label">Pasta de avatares</label>
          <div class="pasta-row">
            <div class="pasta-path" :title="config.pasta_raiz_avatares">
              {{ abreviarPath(config.pasta_raiz_avatares) || "(não configurado)" }}
            </div>
            <button
              class="btn-selecionar"
              @click="selecionarPasta('pasta_raiz_avatares')"
            >
              Selecionar
            </button>
          </div>
        </div>
      </section>

      <!-- APARÊNCIA -->
      <section class="secao">
        <h2 class="secao-titulo">Aparência</h2>
        <div class="campo-tema">
          <AppSelect
            label="Tema"
            :modelValue="config.tema"
            :options="opcoesThema"
            @update:modelValue="onTemaMudou"
          />
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.main-panel {
  padding: 12px;
  height: 100%;
  overflow-y: auto;
}

.shell {
  border: 1px solid var(--cor-borda);
  border-radius: 6px;
  background: var(--cor-menu-bg);
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 28px;
}

.secao {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.secao-titulo {
  font-size: 11px;
  font-weight: bold;
  text-transform: uppercase;
  letter-spacing: 0.8px;
  color: var(--cor-texto-fraco);
  border-bottom: 1px solid var(--cor-borda);
  padding-bottom: 6px;
}

.campo-pasta {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.campo-label {
  font-size: 11px;
  color: var(--cor-texto-fraco);
  padding-left: 2px;
}

.pasta-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.pasta-path {
  flex: 1;
  background: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  padding: 7px 10px;
  font-size: 12px;
  color: var(--cor-texto);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.btn-selecionar {
  background: var(--cor-input-bg);
  border: 1px solid var(--cor-borda);
  border-radius: 4px;
  padding: 7px 14px;
  color: var(--cor-texto);
  font-size: 12px;
  cursor: pointer;
  white-space: nowrap;
  flex-shrink: 0;
}

.btn-selecionar:hover {
  background: var(--cor-menu-hover);
}

.campo-tema {
  max-width: 200px;
}
</style>
