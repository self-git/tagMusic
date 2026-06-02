import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import SingleFileWizard from "@/views/SingleFileWizard.vue";
import TableBatch from "@/views/TableBatch.vue";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/table" },
  // 表格批量审核（30+ 文件）与单文件向导（零散处理），顶部切换
  { path: "/table", name: "table", component: TableBatch },
  { path: "/wizard", name: "wizard", component: SingleFileWizard },
];

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
});
