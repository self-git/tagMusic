<script setup lang="ts">
import { ref } from "vue";
import { storeToRefs } from "pinia";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import SettingsModal from "@/components/SettingsModal.vue";

const store = useAudioStore();
const { total } = storeToRefs(store);

const settings = useSettingsStore();
// iCloud 未下载文件策略开关（PRD 5.6）
const { icloudAutoDownload } = storeToRefs(settings);

// 设置弹窗开关
const settingsOpen = ref(false);
</script>

<template>
  <div class="flex h-screen flex-col bg-neutral-950 text-neutral-100">
    <header
      class="flex items-center justify-between border-b border-neutral-800 px-5 py-3"
    >
      <div class="flex items-center gap-3">
        <span class="text-lg font-semibold tracking-tight">TagCast</span>
        <span class="rounded bg-neutral-800 px-1.5 py-0.5 text-xs text-neutral-400"
          >v1 · 播客</span
        >
        <!-- 表格批量 / 单文件向导 模式切换 -->
        <nav class="ml-2 flex items-center gap-1 rounded-lg bg-neutral-900 p-0.5 text-sm">
          <router-link
            to="/table"
            class="rounded-md px-2.5 py-1 text-neutral-400"
            active-class="bg-neutral-700 text-neutral-100"
          >
            表格批量
          </router-link>
          <router-link
            to="/wizard"
            class="rounded-md px-2.5 py-1 text-neutral-400"
            active-class="bg-neutral-700 text-neutral-100"
          >
            单文件向导
          </router-link>
        </nav>
      </div>
      <div class="flex items-center gap-4">
        <label
          class="flex cursor-pointer items-center gap-2 text-sm text-neutral-400"
          title="iCloud 中未下载的文件：开启则自动下载，关闭则仅提示"
        >
          <input v-model="icloudAutoDownload" type="checkbox" class="accent-sky-500" />
          iCloud 自动下载
        </label>
        <span class="text-sm text-neutral-400">已导入 {{ total }} 个文件</span>
        <button
          class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 hover:bg-neutral-800"
          @click="settingsOpen = true"
        >
          设置
        </button>
      </div>
    </header>

    <SettingsModal :open="settingsOpen" @close="settingsOpen = false" />

    <main class="min-h-0 flex-1 overflow-auto">
      <router-view />
    </main>
  </div>
</template>
