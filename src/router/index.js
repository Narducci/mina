import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'inicio',
    component: () => import('../views/Inicio.vue'),
  },
  {
    path: '/cadastros/categorias',
    name: 'categorias',
    component: () => import('../views/CategoriaView.vue'),
  },
  {
    path: '/cadastros/pessoas',
    name: 'pessoas',
    component: () => import('../views/PessoaView.vue'),
  },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router
