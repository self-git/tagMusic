import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AudioFileMeta } from "@/types/audio";
import type { ParseResult } from "@/types/llm";

// 可整列填充/批量应用的文本字段
type EditableTextField = "title" | "album" | "artist";

/**
 * 音频文件审核状态：保存已读取的文件列表与单文件向导的当前定位。
 * 来源：拖入文件 / 文件选择对话框（见 useAudioImport）。
 */
export const useAudioStore = defineStore("audio", () => {
  const files = ref<AudioFileMeta[]>([]);
  // 单文件向导当前处理到的索引
  const currentIndex = ref(0);
  // LLM 解析置信度，按文件 path 索引（仅展示用）
  const confidenceByPath = ref<Record<string, number | null>>({});

  const currentFile = computed<AudioFileMeta | null>(
    () => files.value[currentIndex.value] ?? null,
  );

  const total = computed(() => files.value.length);

  // 按 path 去重后追加，避免同一文件重复导入
  function addFiles(list: AudioFileMeta[]): void {
    const seen = new Set(files.value.map((f) => f.path));
    const fresh = list.filter((f) => !seen.has(f.path));
    files.value.push(...fresh);
  }

  function next(): void {
    if (currentIndex.value < files.value.length - 1) currentIndex.value += 1;
  }

  function prev(): void {
    if (currentIndex.value > 0) currentIndex.value -= 1;
  }

  function reset(): void {
    files.value = [];
    currentIndex.value = 0;
    confidenceByPath.value = {};
  }

  // 回填 LLM 解析结果：按 path 匹配，更新四字段并记录置信度
  function applyParseResults(results: ParseResult[]): void {
    const indexByPath = new Map(files.value.map((f, i) => [f.path, i]));
    for (const r of results) {
      const idx = indexByPath.get(r.path);
      if (idx === undefined) continue;
      const f = files.value[idx];
      f.title = r.title;
      f.album = r.album;
      f.artist = r.artist;
      f.track = r.track;
      confidenceByPath.value[r.path] = r.confidence;
    }
  }

  // 整列填充：将某字段统一设为同一值
  function fillColumn(field: EditableTextField, value: string): void {
    for (const f of files.value) f[field] = value;
  }

  // 批量应用：仅对指定 path 的行设置字段
  function applyToPaths(field: EditableTextField, value: string, paths: string[]): void {
    const set = new Set(paths);
    for (const f of files.value) {
      if (set.has(f.path)) f[field] = value;
    }
  }

  return {
    files,
    currentIndex,
    currentFile,
    total,
    confidenceByPath,
    addFiles,
    next,
    prev,
    reset,
    applyParseResults,
    fillColumn,
    applyToPaths,
  };
});
