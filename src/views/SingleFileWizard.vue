<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useAudioStore } from "@/store/audio";
import { useAudioImport } from "@/composables/useAudioImport";

const store = useAudioStore();
const { files, currentIndex, currentFile } = storeToRefs(store);
const { isDragging, loading, pickFiles } = useAudioImport();

const progressLabel = computed(() =>
  files.value.length ? `${currentIndex.value + 1} / ${files.value.length}` : "0 / 0",
);

function formatDuration(secs: number | null): string {
  if (secs === null) return "--:--";
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${m}:${s.toString().padStart(2, "0")}`;
}
</script>

<template>
  <div class="relative h-full">
    <!-- 拖拽悬浮提示层 -->
    <div
      v-if="isDragging"
      class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center border-2 border-dashed border-sky-400 bg-sky-500/10"
    >
      <p class="text-lg font-medium text-sky-300">松开以导入音频文件</p>
    </div>

    <!-- 空状态：引导拖入或选择文件 -->
    <div
      v-if="files.length === 0"
      class="flex h-full flex-col items-center justify-center gap-4 px-6 text-center"
    >
      <div
        class="flex h-40 w-full max-w-md flex-col items-center justify-center gap-3 rounded-xl border-2 border-dashed border-neutral-700 text-neutral-400"
      >
        <p>拖入播客音频文件，或</p>
        <button
          class="rounded-lg bg-sky-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-sky-500 disabled:opacity-50"
          :disabled="loading"
          @click="pickFiles"
        >
          {{ loading ? "读取中…" : "选择文件" }}
        </button>
      </div>
      <p class="text-xs text-neutral-600">
        支持 MP3 / M4A / FLAC / OGG / OPUS / WAV / AIFF
      </p>
    </div>

    <!-- 单文件向导：逐个审核当前文件的元数据 -->
    <div v-else-if="currentFile" class="mx-auto max-w-2xl px-6 py-6">
      <div class="mb-4 flex items-center justify-between">
        <span class="text-sm text-neutral-400">{{ progressLabel }}</span>
        <button
          class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 transition hover:bg-neutral-800 disabled:opacity-50"
          :disabled="loading"
          @click="pickFiles"
        >
          继续添加
        </button>
      </div>

      <div class="rounded-xl border border-neutral-800 bg-neutral-900 p-5">
        <div class="mb-4">
          <p class="text-xs uppercase tracking-wide text-neutral-500">原始文件名</p>
          <p class="mt-1 break-all text-sm text-neutral-200">
            {{ currentFile.fileName }}
          </p>
          <p class="mt-1 text-xs text-neutral-600">
            时长 {{ formatDuration(currentFile.durationSecs) }}
          </p>
        </div>

        <div class="grid grid-cols-1 gap-3">
          <label class="block">
            <span class="text-xs text-neutral-500">标题 (title)</span>
            <input
              v-model="currentFile.title"
              type="text"
              class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
            />
          </label>
          <label class="block">
            <span class="text-xs text-neutral-500">节目 (album)</span>
            <input
              v-model="currentFile.album"
              type="text"
              class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
            />
          </label>
          <label class="block">
            <span class="text-xs text-neutral-500">作者 (artist)</span>
            <input
              v-model="currentFile.artist"
              type="text"
              class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
            />
          </label>
          <label class="block">
            <span class="text-xs text-neutral-500">集数 (track)</span>
            <input
              v-model.number="currentFile.track"
              type="number"
              class="mt-1 w-40 rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
            />
          </label>
        </div>
      </div>

      <div class="mt-4 flex items-center justify-between">
        <button
          class="rounded-lg border border-neutral-700 px-4 py-2 text-sm text-neutral-300 transition hover:bg-neutral-800 disabled:opacity-40"
          :disabled="currentIndex === 0"
          @click="store.prev"
        >
          上一个
        </button>
        <button
          class="rounded-lg border border-neutral-700 px-4 py-2 text-sm text-neutral-300 transition hover:bg-neutral-800 disabled:opacity-40"
          :disabled="currentIndex >= files.length - 1"
          @click="store.next"
        >
          跳过 / 下一个
        </button>
      </div>
    </div>
  </div>
</template>
