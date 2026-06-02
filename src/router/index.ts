import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import SingleFileWizard from "@/views/SingleFileWizard.vue";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/wizard" },
  // v1 首发的单文件向导审核界面（表格批量界面在后续 PR 接入）
  { path: "/wizard", name: "wizard", component: SingleFileWizard },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
