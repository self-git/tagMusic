<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { getCoreRowModel, useVueTable, type ColumnDef } from "@tanstack/vue-table";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { useProfilesStore } from "@/store/profiles";
import { useAudioImport } from "@/composables/useAudioImport";
import { useLlmParse } from "@/composables/useLlmParse";
import { useCover } from "@/composables/useCover";
import { useRename } from "@/composables/useRename";
import { useWriteback } from "@/composables/useWriteback";
import { applyRules, toRuleHints } from "@/composables/useRules";
import type { AudioFileMeta } from "@/types/audio";
import type { ParseResult } from "@/types/llm";

const store = useAudioStore();
const { files, confidenceByPath } = storeToRefs(store);
const settings = useSettingsStore();
const profiles = useProfilesStore();
const { pickFiles, loading, isDragging, downloadTotal, downloadDone, pendingDownload } =
  useAudioImport();
const { parsing, error, parse } = useLlmParse();
// 封面自动导入：扫描同目录候选 + AI 匹配 + 手动选图
const { matching, error: coverError, scanAndMatch, pickCover } = useCover();
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
    // 问题 2：本地规则按优先级先跑（字段级叠加），仅把仍有空缺字段的文件交给 AI 兜底
    const rules = settings.rules;
    const localByPath = new Map(files.value.map((f) => [f.path, applyRules(f.fileName, rules)]));
    const gapFiles = files.value.filter((f) => {
      const m = localByPath.get(f.path);
      return !m || m.title === undefined || m.album === undefined || m.artist === undefined || m.track === undefined;
    });
    const ai =
      gapFiles.length > 0
        ? await parse(gapFiles, settings.llmProvider, settings.parseConfig, toRuleHints(rules))
        : [];
    const aiByPath = new Map(ai.map((r) => [r.path, r]));
    // 合并：本地命中字段优先，空缺字段用 AI 结果补
    const merged: ParseResult[] = files.value.map((f) => {
      const m = localByPath.get(f.path) ?? {};
      const a = aiByPath.get(f.path);
      return {
        path: f.path,
        title: m.title ?? a?.title ?? null,
        album: m.album ?? a?.album ?? null,
        artist: m.artist ?? a?.artist ?? null,
        track: m.track ?? a?.track ?? null,
        confidence: a?.confidence ?? null,
      };
    });
    store.applyParseResults(merged);
    // 用节目档案库自动回填 artist/album，收集未匹配节目供引导建档
    unmatchedAlbums.value = profiles.autoFill(files.value);
    // 解析完成后基于 title/album 扫描并 AI 匹配同目录封面（best-effort）
    await scanAndMatch(files.value, settings.llmProvider);
  } catch {
    // 错误已记录在 error，UI 顶部展示
  }
}

// 封面展示缩略图（data URL）：叠加层(AI/手动) 优先，回落文件内嵌封面(基准)；无则 null
function coverThumb(file: AudioFileMeta): string | null {
  return store.displayThumb(file.path, file.embeddedCover);
}
// 封面大图预览（灯箱）：保存当前预览的 data URL，null 表示关闭
const previewImage = ref<string | null>(null);
function coverConfidence(path: string): string {
  const c = store.coverFor(path)?.confidence;
  return c === null || c === undefined ? "" : `${Math.round(c * 100)}%`;
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
      class="pointer-events-none absolute inset-0 z-10 flex items-center justify-center border-2 border-dashed border-accent bg-accent/10"
    >
      <p class="text-lg font-medium text-accent-fg">松开以导入音频文件</p>
    </div>

    <!-- 工具栏 -->
    <div class="flex flex-wrap items-center gap-2 border-b border-line px-4 py-2">
      <button
        class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated disabled:opacity-50"
        :disabled="loading"
        @click="pickFiles"
      >
        添加文件
      </button>
      <button
        class="rounded-lg bg-accent px-3 py-1.5 text-sm font-medium text-white hover:bg-accent-hover disabled:opacity-50"
        :disabled="parsing || matching || files.length === 0"
        @click="parseAll"
      >
        {{ parsing ? "AI 解析中…" : matching ? "匹配封面中…" : "AI 解析" }}
      </button>
      <button
        class="rounded-lg bg-success px-3 py-1.5 text-sm font-medium text-white hover:bg-success-hover disabled:opacity-50"
        :disabled="working || files.length === 0"
        @click="writeAll"
      >
        {{ working ? "写回中…" : "写回元数据" }}
      </button>

      <div class="ml-auto flex items-center gap-2">
        <select
          v-model="fillField"
          class="rounded-lg border border-edge bg-field px-2 py-1.5 text-sm outline-none"
        >
          <option value="title">标题</option>
          <option value="album">节目</option>
          <option value="artist">作者</option>
        </select>
        <input
          v-model="fillValue"
          placeholder="整列填充值"
          class="w-40 rounded-lg border border-edge bg-field px-2 py-1.5 text-sm outline-none focus:border-accent"
        />
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated disabled:opacity-50"
          :disabled="files.length === 0"
          @click="applyFill('all')"
        >
          填充全部
        </button>
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated disabled:opacity-50"
          :disabled="selected.size === 0"
          @click="applyFill('selected')"
        >
          应用所选({{ selected.size }})
        </button>
        <button
          class="rounded-lg border border-warning-edge px-3 py-1.5 text-sm text-warning-fg hover:bg-warning-bg disabled:opacity-50"
          :disabled="working || selected.size === 0"
          @click="resetPaths([...selected])"
        >
          重置所选
        </button>
        <button
          class="rounded-lg border border-danger-edge px-3 py-1.5 text-sm text-danger-fg hover:bg-danger-bg disabled:opacity-50"
          :disabled="selected.size === 0"
          @click="removeSelected"
        >
          移除所选
        </button>
      </div>
    </div>

    <div
      v-if="downloadTotal > 0"
      class="border-b border-info-edge bg-info-bg px-4 py-1.5 text-sm text-info-fg"
    >
      正在从 iCloud 下载… {{ downloadDone }} / {{ downloadTotal }}
    </div>
    <div
      v-if="pendingDownload.length > 0"
      class="border-b border-warning-edge bg-warning-bg px-4 py-1.5 text-sm text-warning-fg"
    >
      {{ pendingDownload.length }} 个文件尚未下载，已跳过。
    </div>
    <p
      v-if="error"
      class="border-b border-danger-edge bg-danger-bg px-4 py-1.5 text-sm text-danger-fg"
    >
      解析失败：{{ error }}
    </p>
    <p
      v-if="writeError"
      class="border-b border-danger-edge bg-danger-bg px-4 py-1.5 text-sm text-danger-fg"
    >
      写回/重置失败：{{ writeError }}
    </p>
    <p
      v-if="coverError"
      class="border-b border-warning-edge bg-warning-bg px-4 py-1.5 text-sm text-warning-fg"
    >
      封面匹配失败（不影响元数据写回）：{{ coverError }}
    </p>
    <p
      v-if="notice"
      class="border-b border-success-edge bg-success-bg px-4 py-1.5 text-sm text-success-fg"
    >
      {{ notice }}
    </p>
    <div
      v-if="unmatchedAlbums.length > 0"
      class="flex flex-wrap items-center gap-2 border-b border-success-edge bg-success-bg px-4 py-1.5 text-sm text-success-fg"
    >
      <span>发现 {{ unmatchedAlbums.length }} 个新节目，建立档案后可自动匹配：</span>
      <button
        v-for="name in unmatchedAlbums"
        :key="name"
        class="rounded-md border border-success-edge px-2 py-0.5 text-xs hover:bg-success-bg"
        @click="profiles.openLibrary(name)"
      >
        + {{ name }}
      </button>
    </div>

    <!-- 表格 -->
    <div class="min-h-0 flex-1 overflow-auto">
      <table v-if="files.length > 0" class="w-full border-collapse text-sm">
        <thead class="sticky top-0 bg-surface text-left text-xs text-muted">
          <tr>
            <th class="w-8 px-2 py-2">
              <input type="checkbox" :checked="allSelected" class="accent-accent" @change="toggleAll" />
            </th>
            <th class="px-2 py-2">原始文件名</th>
            <th class="px-2 py-2">标题</th>
            <th class="px-2 py-2">节目</th>
            <th class="px-2 py-2">作者</th>
            <th class="w-16 px-2 py-2">集</th>
            <th class="w-28 px-2 py-2">封面</th>
            <th class="w-14 px-2 py-2">置信</th>
            <th v-if="renameEnabled" class="px-2 py-2">重命名预览</th>
            <th class="w-24 px-2 py-2"></th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in table.getRowModel().rows"
            :key="row.original.path"
            class="border-t border-line hover:bg-surface/50"
          >
            <td class="px-2 py-1">
              <input
                type="checkbox"
                class="accent-accent"
                :checked="selected.has(row.original.path)"
                @change="toggle(row.original.path)"
              />
            </td>
            <td class="max-w-[220px] truncate px-2 py-1 text-muted" :title="row.original.fileName">
              {{ row.original.fileName }}
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.title"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-edge focus:border-accent focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.album"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-edge focus:border-accent focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model="row.original.artist"
                class="w-full rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-edge focus:border-accent focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <input
                v-model.number="row.original.track"
                type="number"
                class="w-14 rounded border border-transparent bg-transparent px-1 py-0.5 hover:border-edge focus:border-accent focus:outline-none"
              />
            </td>
            <td class="px-2 py-1">
              <div class="flex items-center gap-1.5">
                <img
                  v-if="coverThumb(row.original)"
                  :src="coverThumb(row.original) as string"
                  class="h-8 w-8 cursor-zoom-in rounded object-cover"
                  :title="`封面匹配 ${coverConfidence(row.original.path)}（单击查看大图）`"
                  @click="previewImage = coverThumb(row.original)"
                />
                <span v-else class="text-xs text-dim">无</span>
                <button
                  class="text-xs text-accent-fg hover:underline"
                  title="手动选择封面图片"
                  @click="pickCover(row.original.path)"
                >
                  选图
                </button>
                <button
                  v-if="coverThumb(row.original)"
                  class="text-xs text-faint hover:text-danger-fg"
                  title="清除封面（写回时移除）"
                  @click="store.clearCover(row.original.path)"
                >
                  清除
                </button>
              </div>
            </td>
            <td class="px-2 py-1 text-xs text-faint">
              {{ confidenceLabel(row.original.path) }}
            </td>
            <td
              v-if="renameEnabled"
              class="max-w-[260px] truncate px-2 py-1 text-xs text-success-fg"
              :title="preview(row.original) ?? ''"
            >
              {{ preview(row.original) ?? "—" }}
            </td>
            <td class="whitespace-nowrap px-2 py-1 text-center">
              <button
                v-if="store.isWritten(row.original.path)"
                class="mr-2 text-xs text-warning-fg hover:underline disabled:opacity-50"
                :disabled="working"
                title="恢复原文件名与原标签"
                @click="resetPaths([row.original.path])"
              >
                重置
              </button>
              <button
                class="text-dim hover:text-danger-fg"
                title="从工作区移除"
                @click="removeByPaths([row.original.path])"
              >
                ✕
              </button>
            </td>
          </tr>
        </tbody>
      </table>

      <div v-else class="p-10 text-center text-faint">
        还没有文件，点击「添加文件」或直接拖入音频文件。
      </div>
    </div>

    <!-- 封面大图预览灯箱：点击遮罩关闭 -->
    <div
      v-if="previewImage"
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/80 p-8"
      @click="previewImage = null"
    >
      <img
        :src="previewImage"
        class="max-h-full max-w-full rounded-lg object-contain shadow-2xl"
        alt="封面预览"
      />
    </div>
  </div>
</template>
