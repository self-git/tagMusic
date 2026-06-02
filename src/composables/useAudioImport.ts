import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { AudioFileMeta } from "@/types/audio";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { useIcloud } from "@/composables/useIcloud";

// v1 支持的音频扩展名（与 PRD 5.1 一致）
const AUDIO_EXTENSIONS = ["mp3", "m4a", "flac", "ogg", "opus", "wav", "aiff", "aif"];

function isAudioPath(p: string): boolean {
  const ext = p.split(".").pop()?.toLowerCase() ?? "";
  return AUDIO_EXTENSIONS.includes(ext);
}

/**
 * 文件导入逻辑：监听 Tauri 拖拽事件 + 提供文件选择对话框，
 * 统一调用后端 `read_audio_metadata` 读取标签并写入 store。
 */
export function useAudioImport() {
  const store = useAudioStore();
  const settings = useSettingsStore();
  const icloud = useIcloud();
  const isDragging = ref(false);
  const loading = ref(false);
  // 仅提示模式 / 下载超时下，仍未下载、未读取的文件，供 UI 提示用户去 Finder 处理
  const pendingDownload = ref<string[]>([]);
  let unlisten: UnlistenFn | null = null;

  async function importPaths(paths: string[]): Promise<void> {
    const audioPaths = paths.filter(isAudioPath);
    if (audioPaths.length === 0) return;
    loading.value = true;
    try {
      // 先查 iCloud 状态，挑出 iCloud 托管但尚未下载的文件
      const statuses = await icloud.checkStatus(audioPaths);
      const notDownloaded = statuses
        .filter((s) => s.isUbiquitous && !s.isDownloaded)
        .map((s) => s.path);

      let readable = audioPaths;
      if (notDownloaded.length > 0) {
        if (settings.icloudAutoDownload) {
          // 自动下载：触发并轮询，超时残留的仍记为待处理
          const stillPending = await icloud.ensureDownloaded(notDownloaded);
          readable = audioPaths.filter((p) => !stillPending.includes(p));
          pendingDownload.value = stillPending;
        } else {
          // 仅提示：跳过未下载文件，交用户去 Finder 手动下载
          readable = audioPaths.filter((p) => !notDownloaded.includes(p));
          pendingDownload.value = notDownloaded;
        }
      } else {
        pendingDownload.value = [];
      }

      if (readable.length > 0) {
        // IPC 调用 Rust 端读取元数据，返回结构对应 AudioFileMeta
        const metas = await invoke<AudioFileMeta[]>("read_audio_metadata", {
          paths: readable,
        });
        store.addFiles(metas);
      }
    } finally {
      loading.value = false;
      icloud.resetProgress();
    }
  }

  async function pickFiles(): Promise<void> {
    const selected = await open({
      multiple: true,
      filters: [{ name: "Audio", extensions: AUDIO_EXTENSIONS }],
    });
    if (selected === null) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    await importPaths(paths);
  }

  onMounted(async () => {
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const payload = event.payload;
      if (payload.type === "enter" || payload.type === "over") {
        isDragging.value = true;
      } else if (payload.type === "drop") {
        isDragging.value = false;
        void importPaths(payload.paths);
      } else {
        isDragging.value = false;
      }
    });
  });

  onUnmounted(() => {
    unlisten?.();
  });

  return {
    isDragging,
    loading,
    pendingDownload,
    downloadTotal: icloud.downloadTotal,
    downloadDone: icloud.downloadDone,
    importPaths,
    pickFiles,
  };
}
