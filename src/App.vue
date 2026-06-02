<script setup lang="ts">
import { useAudioStore } from "@/store/audio";
import { storeToRefs } from "pinia";

const store = useAudioStore();
const { total } = storeToRefs(store);
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
      <div class="text-sm text-neutral-400">已导入 {{ total }} 个文件</div>
    </header>

    <main class="min-h-0 flex-1 overflow-auto">
      <router-view />
    </main>
  </div>
</template>
