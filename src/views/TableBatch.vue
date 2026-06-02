<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { getCoreRowModel, useVueTable, type ColumnDef } from "@tanstack/vue-table";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { useProfilesStore } from "@/store/profiles";
import { useAudioImport } from "@/composables/useAudioImport";
import { useLlmParse } from "@/composables/useLlmParse";
import { useRename } from "@/composables/useRename";
import { useWriteback } from "@/composables/useWriteback";
import type { AudioFileMeta } from "@/types/audio";

const store = useAudioStore();
const { files, confidenceByPath } = storeToRefs(store);
const settings = useSettingsStore();
const profiles = useProfilesStore();
const { pickFiles, loading, isDragging, downloadTotal, downloadDone, pendingDownload } =
  useAudioImport();
const { parsing, error, parse } = useLlmParse();
// 重命名预览（开关开启且有标题时返回新名）
const { renameEnabled, preview } = useRename();
// 元数据写回与重置
const { working, error: writeError, notice, write, reset } = useWriteback();

// 解析后未匹配到档案的节目名，引导用户建档
const unmatchedAlbums = ref<string[]>([]);

// 选中行集合（按 path），用于「批量应用」
const selected = ref<Set<string>>(new Set());
function toggle(path: string): void {
  const next = new Set(selected.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  selected.value = next;
}
const allSelected = computed(
  () => files.value.length > 0 && selected.value.size === files.value.length,
);
function toggleAll(): void {
  selected.value = allSelected.value ? new Set() : new Set(files.value.map((f) => f.path));
}

// 从工作区移除文件，并同步清理选中集合
function removeByPaths(paths: string[]): void {
  store.removeByPaths(paths);
  const next = new Set(selected.value);
  for (const p of paths) next.delete(p);
  selected.value = next;
}
function removeSelected(): void {
  removeByPaths([...selected.value]);
}

// 写回全部文件元数据（开启重命名时同步改名）
async function writeAll(): Promise<void> {
  try {
    await write(files.value);
  } catch {
    // 错误已记录在 writeError，UI 顶部展示
  }
}
// 重置：恢复原 tag + 原文件名。无参数时重置所选
async function resetPaths(paths: string[]): Promise<void> {
  try {
    await reset(paths);
  } catch {
    // 错误已记录在 writeError，UI 顶部展示
  }
}

// 整列填充 / 批量应用
const fillField = ref<"title" | "album" | "artist">("album");
const fillValue = ref("");
function applyFill(scope: "all" | "selected"): void {
  if (scope === "all") store.fillColumn(fillField.value, fillValue.value);
  else store.applyToPaths(fillField.value, fillValue.value, [...selected.value]);
}

// TanStack：用列定义 + 核心行模型驱动表格（headless，单元格自渲染为可编辑输入）
const columns: ColumnDef<AudioFileMeta>[] = [
  { accessorKey: "fileName", header: "原始文件名" },
  { accessorKey: "title", header: "标题" },
  { accessorKey: "album", header: "节目" },
  { accessorKey: "artist", header: "作者" },
  { accessorKey: "track", header: "集" },
];
const table = useVueTable({
  get data() {
    return files.value;
  },
  columns,
  getCoreRowModel: getCoreRowModel(),
});

async function parseAll(): Promise<void> {
  try {
    const results = await parse(files.value, settings.llmProvider);
    store.applyParseResults(results);
    // 用节目档案库自动回填 artist/album，收集未匹配节目供引导建档
    unmatchedAlbums.value = profiles.autoFill(files.value);
  } catch {
    // 错误已记录在 error，UI 顶部展示
  }
}

function confidenceLabel(path: string): string {
  const c = confidenceByPath.value[path];
  return c === null || c === undefined ? "—" : `${Math.round(c * 100)}%`;
}
</script>

<template>
  <div class="relative flex h-full flex-col">
    <div
      v-if="isDragging"
      class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center border-2 border-dashed border-sky-400 bg-sky-500/10"
    >
      <p class="text-lg font-medium text-sky-300">松开以导入音频文件</p>
    </div>

    <!-- 工具栏 -->
    <div class="flex flex-wrap items-center gap-2 border-b border-neutral-800 px-4 py-2">
      <button
        class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 hover:bg-neutral-800 disabled:opacity-50"
        :disabled="loading"
        @click="pickFiles"
      >
        添加文件
      </button>
      <button
        class="rounded-lg bg-sky-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-sky-500 disabled:opacity-50"
        :disabled="parsing || files.length === 0"
        @click="parseAll"
      >
        {{ parsing ? "AI 解析中…" : "AI 解析" }}
      </button>
      <button
        class="rounded-lg bg-emerald-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-emerald-500 disabled:opacity-50"
        :disabled="working || files.length === 0"
        @click="writeAll"
      >
        {{ working ? "写回中…" : "写回元数据" }}
      </button>

      <div class="ml-auto flex items-center gap-2">
        <select
          v-model="fillField"
          class="rounded-lg border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-sm outline-none"
        >
          <option value="title">标题</option>
          <option value="album">节目</option>
          <option value="artist">作者</option>
        </select>
        <input
          v-model="fillValue"
          placeholder="整列填充值"
          class="w-40 rounded-lg border border-neutral-700 bg-neutral-950 px-2 py-1.5 text-sm outline-none focus:border-sky-500"
        />
        <button
          class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 hover:bg-neutral-800 disabled:opacity-50"
          :disabled="files.length === 0"
          @click="applyFill('all')"
        >
          填充全部
        </button>
        <button
          class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 hover:bg-neutral-800 disabled:opacity-50"
          :disabled="selected.size === 0"
          @click="applyFill('selected')"
        >
          应用所选({{ selected.size }})
        </button>
        <button
          class="rounded-lg border border-amber-800 px-3 py-1.5 text-sm text-amber-300 hover:bg-amber-950/50 disabled:opacity-50"
          :disabled="working || selected.size === 0"
          @click="resetPaths([...selected])"
        >
          重置所选
        </button>
        <button
          class="rounded-lg border border-red-800 px-3 py-1.5 text-sm text-red-300 hover:bg-red-950/50 disabled:opacity-50"
          :disabled="selected.size === 0"
          @click="removeSelected"
        >
          移除所选
        </button>
      </div>
    </div>

    <div
      v-if="downloadTotal > 0"
      class="border-b border-sky-900 bg-sky-950/40 px-4 py-1.5 text-sm text-sky-300"
    >
      正在从 iCloud 下载… {{ downloadDone }} / {{ downloadTotal }}
    </div>
    <div
      v-if="pendingDownload.length > 0"
      class="border-b border-amber-900 bg-amber-950/40 px-4 py-1.5 text-sm text-amber-300"
    >
      {{ pendingDownload.length }} 个文件尚未下载，已跳过。
    </div>
    <p
      v-if="error"
      class="border-b border-red-900 bg-red-950/40 px-4 py-1.5 text-sm text-red-300"
    >
      解析失败：{{ error }}
    </p>
    <p
      v-if="writeError"
      class="border-b border-red-900 bg-red-950/40 px-4 py-1.5 text-sm text-red-300"
    >
      写回/重置失败：{{ writeError }}
    </p>
    <p
      v-if="notice"
      class="border-b border-emerald-900 bg-emerald-950/40 px-4 py-1.5 text-sm text-emerald-300"
    >
      {{ notice }}
    </p>
    <div
      v-if="unmatchedAlbums.length > 0"
      class="flex flex-wrap items-center gap-2 border-b border-emerald-900 bg-emerald-950/30 px-4 py-1.5 text-sm text-emerald-300"
    >
      <span>发现 {{ unmatchedAlbums.length }} 个新节目，建立档案后可自动匹配：</span>
      <button
        v-for="name in unmatchedAlbums"
        :key="name"
        class="rounded-md border border-emerald-800 px-2 py-0.5 text-xs hover:bg-emerald-900/50"
        @click="profiles.openLibrary(name)"
      >
        + {{ name }}
      </button>
    </div>

    <!-- 表格 -->
    <div class="min-h-0 flex-1 overflow-auto">
      <table v-if="files.length > 0" class="w-full border-collapse text-sm">
        <thead class="sticky top-0 bg-neutral-900 text-left text-xs text-neutral-400">
          <tr>
            <th class="w-8 px-2 py-2">
              <input type="checkbox" :checked="allSelected" class="accent-sky-500" @change="toggleAll" />
            </th>
            <th class="px-2 py-2">原始文件名</th>
            <th class="px-2 py-2">标题</th>
            <th class="px-2 py-2">节目</th>
            <th class="px-2 py-2">作者</th>
            <th class="w-16 px-2 py-2">集</th>
            <th class="w-14 px-2 py-2">置信</th>
            <th v-if="renameEnabled" class="px-2 py-2">重命名预览</th>
            <th class="w-24 px-2 py-2"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in table.getRowModel().rows"
            :key="row.original.path"
            class="border-t border-neutral-800 hover:bg-neutral-900/50"
          >
            <td class="px-2 py-1">
              <input
                type="checkbox"
                class="accent-sky-500"
                :checked="selected.has(row.original.path)"
                @change="toggle(row.original.path)"
              />
            </td>
            <td class="max-w-[220px] truncate px-2 py-1 text-neutral-400" :title="row.original.fileName">
              {{ row.original.fileName }}
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.title"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-neutral-700 focus:border-sky-500 focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.album"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-neutral-700 focus:border-sky-500 focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.artist"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-neutral-700 focus:border-sky-500 focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model.number="row.original.track"
                type="number"
                class="w-14 rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-neutral-700 focus:border-sky-500 focus:outline-none"
              />
            </td>
            <td class="px-2 py-1 text-xs text-neutral-500">
              {{ confidenceLabel(row.original.path) }}
            </td>
            <td
              v-if="renameEnabled"
              class="max-w-[260px] truncate px-2 py-1 text-xs text-emerald-400"
              :title="preview(row.original) ?? ''"
            >
              {{ preview(row.original) ?? "—" }}
            </td>
            <td class="whitespace-nowrap px-2 py-1 text-center">
              <button
                v-if="store.isWritten(row.original.path)"
                class="mr-2 text-xs text-amber-400 hover:underline disabled:opacity-50"
                :disabled="working"
                title="恢复原文件名与原标签"
                @click="resetPaths([row.original.path])"
              >
                重置
              </button>
              <button
                class="text-neutral-600 hover:text-red-400"
                title="从工作区移除"
                @click="removeByPaths([row.original.path])"
              >
                ✕
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div v-else class="p-10 text-center text-neutral-500">
        还没有文件，点击「添加文件」或直接拖入音频文件。
      </div>
    </div>
  </div>
</template>
