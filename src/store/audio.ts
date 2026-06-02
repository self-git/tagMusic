import { defineStore } from "pinia";
import { computed, ref } from "vue";
import type { AudioFileMeta } from "@/types/audio";

/**
 * 音频文件审核状态：保存已读取的文件列表与单文件向导的当前定位。
 * 来源：拖入文件 / 文件选择对话框（见 useAudioImport）。
 */
export const useAudioStore = defineStore("audio", () => {
  const files = ref<AudioFileMeta[]>([]);
  // 单文件向导当前处理到的索引
  const currentIndex = ref(0);

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
  }

  return { files, currentIndex, currentFile, total, addFiles, next, prev, reset };
});
