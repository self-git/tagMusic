import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { storeToRefs } from "pinia";
import type { AudioFileMeta } from "@/types/audio";
import type { WriteInput, WriteOutcome, ResetOutcome } from "@/types/write";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { renderName } from "@/composables/useRename";

/**
 * 元数据写回与重置：把审核后的四字段写入文件（开启重命名时同步按模板改名），
 * 并支持依据原始快照一键重置。所有磁盘改动经 Rust IPC 完成。
 */
export function useWriteback() {
  const store = useAudioStore();
  const settings = useSettingsStore();
  const { renameEnabled, renameTemplate } = storeToRefs(settings);
  const working = ref(false);
  const error = ref<string | null>(null);
  // 操作成功提示，数秒后自动消失
  const notice = ref<string | null>(null);
  let noticeTimer: ReturnType<typeof setTimeout> | null = null;
  function flash(msg: string): void {
    notice.value = msg;
    if (noticeTimer) clearTimeout(noticeTimer);
    noticeTimer = setTimeout(() => {
      notice.value = null;
    }, 4000);
  }

  // 仅在开启重命名且有标题时生成新文件名（与表格预览口径一致）
  function buildNewName(file: AudioFileMeta): string | null {
    if (!renameEnabled.value || !file.title) return null;
    const name = renderName(file, renameTemplate.value);
    return name === file.fileName ? null : name;
  }

  async function write(files: AudioFileMeta[]): Promise<void> {
    if (files.length === 0) return;
    working.value = true;
    error.value = null;
    try {
      const inputs: WriteInput[] = files.map((f) => {
        // 封面：选中则写入路径，显式清除则置 clearCover，否则保持文件原封面不动
        const cover = store.coverFor(f.path);
        return {
          path: f.path,
          title: f.title,
          album: f.album,
          artist: f.artist,
          track: f.track,
          newName: buildNewName(f),
          coverPath: cover?.chosen ?? null,
          clearCover: cover?.cleared ?? false,
        };
      });
      const outcomes = await invoke<WriteOutcome[]>("write_metadata", { files: inputs });
      store.applyWriteOutcomes(outcomes);
      flash(`已成功写回 ${outcomes.length} 个文件的元数据`);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      working.value = false;
    }
  }

  async function reset(paths: string[]): Promise<void> {
    if (paths.length === 0) return;
    working.value = true;
    error.value = null;
    try {
      const outcomes = await invoke<ResetOutcome[]>("reset_files", { paths });
      store.applyResetOutcomes(outcomes);
      flash(`已重置 ${outcomes.length} 个文件`);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      working.value = false;
    }
  }

  // 启动时同步已写回文件路径，用于标记可重置状态
  async function loadWritten(): Promise<void> {
    const paths = await invoke<string[]>("list_snapshot_paths");
    store.setWrittenPaths(paths);
  }

  return { working, error, notice, write, reset, loadWritten };
}
