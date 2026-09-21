import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  {
    path: "/",
    name: "inicio",
    component: () => import("../views/Inicio.vue"),
  },
  {
    path: "/cadastros/categorias",
    name: "categorias",
    component: () => import("../views/CategoriaView.vue"),
  },
  {
    path: "/cadastros/pessoas",
    name: "pessoas",
    component: () => import("../views/PessoaView.vue"),
  },
  {
    path: "/cadastros/contas",
    name: "contas",
    component: () => import("../views/ContaView.vue"),
  },
  {
    path: "/cadastros/ativar-conta",
    name: "ativar-conta",
    component: () => import("../views/AtivarContaView.vue"),
  },
  {
    path: "/lancamentos/diretos",
    name: "lancamentos-diretos",
    component: () => import("../views/LancamentosDiretosView.vue"),
  },
  {
    path: "/lancamentos/indiretos",
    name: "lancamentos-indiretos",
    component: () => import("../views/LancamentosIndiretosView.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
