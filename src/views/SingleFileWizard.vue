<script setup lang="ts">
import { computed } from "vue";
import { storeToRefs } from "pinia";
import { useAudioStore } from "@/store/audio";
import { useProfilesStore } from "@/store/profiles";
import { useAudioImport } from "@/composables/useAudioImport";
import { useRename } from "@/composables/useRename";
import { useWriteback } from "@/composables/useWriteback";

const store = useAudioStore();
const { files, currentIndex, currentFile } = storeToRefs(store);
const profiles = useProfilesStore();
const { isDragging, loading, pickFiles, pendingDownload, downloadTotal, downloadDone } =
  useAudioImport();
// 当前文件的重命名预览（开关开启且有标题时）
const { preview } = useRename();
const currentPreview = computed(() => (currentFile.value ? preview(currentFile.value) : null));
// 元数据写回与重置
const { working, error: writeError, notice, write, reset } = useWriteback();

async function writeCurrent(): Promise<void> {
  if (!currentFile.value) return;
  try {
    await write([currentFile.value]);
  } catch {
    // 错误已记录在 writeError，UI 内展示
  }
}
async function resetCurrent(): Promise<void> {
  if (!currentFile.value) return;
  try {
    await reset([currentFile.value.path]);
  } catch {
    // 错误已记录在 writeError，UI 内展示
  }
}

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

    <!-- iCloud 下载进度 -->
    <div
      v-if="downloadTotal > 0"
      class="mx-auto mt-3 max-w-2xl rounded-lg border border-sky-800 bg-sky-950/40 px-4 py-2 text-sm text-sky-300"
    >
      正在从 iCloud 下载… {{ downloadDone }} / {{ downloadTotal }}
    </div>

    <!-- 未下载文件提示（仅提示模式 / 下载超时） -->
    <div
      v-if="pendingDownload.length > 0"
      class="mx-auto mt-3 max-w-2xl rounded-lg border border-amber-800 bg-amber-950/40 px-4 py-2 text-sm text-amber-300"
    >
      {{ pendingDownload.length }} 个文件尚未下载，已跳过。请在 Finder 中下载后重新导入。
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

        <!-- 重命名预览 + 存为节目档案 -->
        <div class="mt-4 flex items-center justify-between border-t border-neutral-800 pt-3">
          <p v-if="currentPreview" class="truncate text-xs text-emerald-400" :title="currentPreview">
            重命名预览：{{ currentPreview }}
          </p>
          <span v-else class="text-xs text-neutral-600">未开启重命名</span>
          <button
            class="rounded-lg border border-neutral-700 px-3 py-1.5 text-xs text-neutral-300 hover:bg-neutral-800 disabled:opacity-40"
            :disabled="!currentFile.album"
            @click="profiles.openLibrary(currentFile.album ?? undefined)"
          >
            存为节目档案
          </button>
        </div>
      </div>

      <p v-if="writeError" class="mt-3 text-sm text-red-400">写回/重置失败：{{ writeError }}</p>
      <p v-if="notice" class="mt-3 text-sm text-emerald-400">{{ notice }}</p>

      <div class="mt-4 flex items-center justify-between">
        <button
          class="rounded-lg border border-neutral-700 px-4 py-2 text-sm text-neutral-300 transition hover:bg-neutral-800 disabled:opacity-40"
          :disabled="currentIndex === 0"
          @click="store.prev"
        >
          上一个
        </button>
        <div class="flex items-center gap-2">
          <button
            class="rounded-lg bg-emerald-600 px-4 py-2 text-sm font-medium text-white transition hover:bg-emerald-500 disabled:opacity-50"
            :disabled="working"
            @click="writeCurrent"
          >
            {{ working ? "写回中…" : "写回元数据" }}
          </button>
          <button
            v-if="store.isWritten(currentFile.path)"
            class="rounded-lg border border-amber-800 px-4 py-2 text-sm text-amber-300 transition hover:bg-amber-950/50 disabled:opacity-50"
            :disabled="working"
            @click="resetCurrent"
          >
            重置
          </button>
        </div>
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
