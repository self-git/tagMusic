import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ICloudStatus } from "@/types/icloud";

const POLL_INTERVAL_MS = 1000;
// 最长等待约 2 分钟，避免长期挂起
const MAX_POLLS = 120;

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * iCloud 文件状态检查与下载编排：触发下载后轮询状态直至完成/超时（PRD 选定轮询方案，非 KVO）。
 */
export function useIcloud() {
  // 下载进度：总数 / 已完成数（供 UI 展示进度）
  const downloadTotal = ref(0);
  const downloadDone = ref(0);

  async function checkStatus(paths: string[]): Promise<ICloudStatus[]> {
    if (paths.length === 0) return [];
    return invoke<ICloudStatus[]>("check_icloud_status", { paths });
  }

  // 触发下载并轮询，返回最终仍未下载的路径（超时残留）
  async function ensureDownloaded(paths: string[]): Promise<string[]> {
    if (paths.length === 0) return [];
    downloadTotal.value = paths.length;
    downloadDone.value = 0;

    await Promise.all(paths.map((p) => invoke("start_icloud_download", { path: p })));

    let pending = [...paths];
    for (let i = 0; i < MAX_POLLS && pending.length > 0; i += 1) {
      await delay(POLL_INTERVAL_MS);
      const statuses = await checkStatus(pending);
      pending = statuses.filter((s) => !s.isDownloaded).map((s) => s.path);
      downloadDone.value = downloadTotal.value - pending.length;
    }
    return pending;
  }

  function resetProgress(): void {
    downloadTotal.value = 0;
    downloadDone.value = 0;
  }

  return { downloadTotal, downloadDone, checkStatus, ensureDownloaded, resetProgress };
}
