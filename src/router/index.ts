import { createRouter, createWebHashHistory } from "vue-router";
import MainLayout from "@/layouts/MainLayout.vue";

const routes = [
  {
    path: "/",
    component: MainLayout,
    children: [
      {
        path: "",
        name: "Dashboard",
        component: () => import("@/views/Dashboard.vue"),
      },
      {
        path: "parts",
        name: "Parts",
        component: () => import("@/views/Parts.vue"),
      },
      {
        path: "moc",
        name: "Moc",
        component: () => import("@/views/Moc.vue"),
      },
      {
        path: "master",
        name: "Master",
        component: () => import("@/views/MasterData.vue"),
      },
      {
        path: "import-export",
        name: "ImportExport",
        component: () => import("@/views/ImportExport.vue"),
      },
      {
        path: "settings",
        name: "Settings",
        component: () => import("@/views/Settings.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
