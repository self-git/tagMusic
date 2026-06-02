<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";

const store = useAudioStore();
const { total } = storeToRefs(store);

const settings = useSettingsStore();
// iCloud 未下载文件策略开关（PRD 5.6）
const { icloudAutoDownload } = storeToRefs(settings);
</script>

<template>
  <div class="flex h-screen flex-col bg-neutral-950 text-neutral-100">
    <header
      class="flex items-center justify-between border-b border-neutral-800 px-5 py-3"
    >
      <div class="flex items-center gap-2">
        <span class="text-lg font-semibold tracking-tight">TagCast</span>
        <span class="rounded bg-neutral-800 px-1.5 py-0.5 text-xs text-neutral-400"
          >v1 · 播客</span
        >
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
      </div>
    </header>

    <main class="min-h-0 flex-1 overflow-auto">
      <router-view />
    </main>
  </div>
</template>
