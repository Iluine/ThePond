import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'

// Hello World du bootstrap : une seule route. Les autres (welcome, upload,
// gallery, etc.) seront ajoutées au prompt 4 (routing & stores Pinia).
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'home',
    component: () => import('../views/HomeView.vue'),
  },
]

export const router = createRouter({
  history: createWebHistory(),
  routes,
})
