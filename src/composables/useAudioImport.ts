import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import type { AudioFileMeta } from "@/types/audio";
import { useAudioStore } from "@/store/audio";

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
  const isDragging = ref(false);
  const loading = ref(false);
  let unlisten: UnlistenFn | null = null;

  async function importPaths(paths: string[]): Promise<void> {
    const audioPaths = paths.filter(isAudioPath);
    if (audioPaths.length === 0) return;
    loading.value = true;
    try {
      // IPC 调用 Rust 端读取元数据，返回结构对应 AudioFileMeta
      const metas = await invoke<AudioFileMeta[]>("read_audio_metadata", {
        paths: audioPaths,
      });
      store.addFiles(metas);
    } finally {
      loading.value = false;
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

  return { isDragging, loading, importPaths, pickFiles };
}
