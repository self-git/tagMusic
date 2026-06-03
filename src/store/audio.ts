import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AudioFileMeta } from "@/types/audio";
import type { ParseResult } from "@/types/llm";
import type { WriteOutcome, ResetOutcome } from "@/types/write";
import type { CoverCandidates, CoverMatchResult, CoverSelection } from "@/types/cover";

// 可整列填充/批量应用的文本字段
type EditableTextField = "title" | "album" | "artist";

function basename(p: string): string {
  return p.split("/").pop() ?? p;
}

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
  // 已写回（存在原始快照）的文件当前路径，决定是否可重置
  const writtenPaths = ref<Set<string>>(new Set());
  // 封面选择状态，按文件 path 索引（候选/AI 选中/手动/清除，随写回一起落盘）
  const coverByPath = ref<Record<string, CoverSelection>>({});

  const currentFile = computed<AudioFileMeta | null>(
    () => files.value[currentIndex.value] ?? null,
  );

  const total = computed(() => files.value.length);

  // 按 path 去重后追加，避免同一文件重复导入。
  // 用新数组替换引用（而非原地 push）：TanStack Table 按 data 引用 memo 行模型，
  // 原地修改不会触发表格刷新。
  function addFiles(list: AudioFileMeta[]): void {
    const seen = new Set(files.value.map((f) => f.path));
    const fresh = list.filter((f) => !seen.has(f.path));
    if (fresh.length === 0) return;
    files.value = [...files.value, ...fresh];
  }

  // 从工作区移除指定文件：同步清理置信度记录并收敛 currentIndex（同样替换数组引用以刷新表格）
  function removeByPaths(paths: string[]): void {
    const set = new Set(paths);
    if (set.size === 0) return;
    files.value = files.value.filter((f) => !set.has(f.path));
    const written = new Set(writtenPaths.value);
    for (const p of paths) {
      delete confidenceByPath.value[p];
      delete coverByPath.value[p];
      written.delete(p);
    }
    writtenPaths.value = written;
    if (currentIndex.value > files.value.length - 1) {
      currentIndex.value = Math.max(0, files.value.length - 1);
    }
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
    writtenPaths.value = new Set();
    coverByPath.value = {};
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

  function isWritten(path: string): boolean {
    return writtenPaths.value.has(path);
  }

  // 启动时载入已写回文件路径（来自 SQLite 快照），用于标记可重置
  function setWrittenPaths(paths: string[]): void {
    writtenPaths.value = new Set(paths);
  }

  // 写回成功后：重命名的文件同步更新 path/fileName 与置信度键，并标记为已写回
  function applyWriteOutcomes(outcomes: WriteOutcome[]): void {
    const written = new Set(writtenPaths.value);
    const indexByPath = new Map(files.value.map((f, i) => [f.path, i]));
    for (const o of outcomes) {
      const idx = indexByPath.get(o.oldPath);
      if (idx !== undefined && o.newPath !== o.oldPath) {
        const f = files.value[idx];
        const conf = confidenceByPath.value[o.oldPath];
        if (conf !== undefined) {
          confidenceByPath.value[o.newPath] = conf;
          delete confidenceByPath.value[o.oldPath];
        }
        const cover = coverByPath.value[o.oldPath];
        if (cover !== undefined) {
          coverByPath.value[o.newPath] = cover;
          delete coverByPath.value[o.oldPath];
        }
        f.path = o.newPath;
        f.fileName = basename(o.newPath);
      }
      written.delete(o.oldPath);
      written.add(o.newPath);
    }
    files.value = [...files.value];
    writtenPaths.value = written;
  }

  // 重置成功后：还原 path/fileName 与四字段，并清除已写回标记
  function applyResetOutcomes(outcomes: ResetOutcome[]): void {
    const written = new Set(writtenPaths.value);
    const indexByPath = new Map(files.value.map((f, i) => [f.path, i]));
    for (const o of outcomes) {
      written.delete(o.currentPath);
      written.delete(o.restoredPath);
      const idx = indexByPath.get(o.currentPath);
      if (idx === undefined) continue;
      const f = files.value[idx];
      if (o.restoredPath !== o.currentPath) {
        const conf = confidenceByPath.value[o.currentPath];
        if (conf !== undefined) {
          confidenceByPath.value[o.restoredPath] = conf;
          delete confidenceByPath.value[o.currentPath];
        }
        f.path = o.restoredPath;
        f.fileName = basename(o.restoredPath);
      }
      f.title = o.title;
      f.album = o.album;
      f.artist = o.artist;
      f.track = o.track;
    }
    files.value = [...files.value];
    writtenPaths.value = written;
  }

  // 默认封面选择态（无候选、不写、未清除）
  function emptyCover(): CoverSelection {
    return { candidates: [], chosen: null, confidence: null, cleared: false, thumb: null };
  }

  // 回填扫描到的同目录候选图片
  function setCoverCandidates(list: CoverCandidates[]): void {
    for (const c of list) {
      const prev = coverByPath.value[c.path] ?? emptyCover();
      coverByPath.value[c.path] = { ...prev, candidates: c.images };
    }
  }

  // 回填 AI 封面匹配结果（chosen 为完整图片路径或 null）
  function applyCoverMatches(results: CoverMatchResult[]): void {
    for (const r of results) {
      const prev = coverByPath.value[r.path] ?? emptyCover();
      coverByPath.value[r.path] = {
        ...prev,
        chosen: r.chosen,
        confidence: r.confidence,
        cleared: false,
      };
    }
  }

  // 手动指定封面图片（覆盖 AI 选择）
  function setCover(path: string, image: string, thumb: string | null): void {
    const prev = coverByPath.value[path] ?? emptyCover();
    coverByPath.value[path] = { ...prev, chosen: image, cleared: false, thumb };
  }

  // 设置缩略图 data URL（异步加载后回填）
  function setCoverThumb(path: string, thumb: string | null): void {
    const prev = coverByPath.value[path];
    if (prev) coverByPath.value[path] = { ...prev, thumb };
  }

  // 清除封面：写回时移除文件已有封面
  function clearCover(path: string): void {
    const prev = coverByPath.value[path] ?? emptyCover();
    coverByPath.value[path] = { ...prev, chosen: null, cleared: true, thumb: null };
  }

  function coverFor(path: string): CoverSelection | undefined {
    return coverByPath.value[path];
  }

  return {
    files,
    currentIndex,
    currentFile,
    total,
    confidenceByPath,
    writtenPaths,
    coverByPath,
    setCoverCandidates,
    applyCoverMatches,
    setCover,
    setCoverThumb,
    clearCover,
    coverFor,
    addFiles,
    removeByPaths,
    next,
    prev,
    reset,
    applyParseResults,
    fillColumn,
    applyToPaths,
    isWritten,
    setWrittenPaths,
    applyWriteOutcomes,
    applyResetOutcomes,
  };
});
