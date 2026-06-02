import { defineStore } from "pinia";
import { ref, watch } from "vue";

const STORAGE_KEY = "tagcast.settings.icloudAutoDownload";

function readBool(key: string, fallback: boolean): boolean {
  const raw = localStorage.getItem(key);
  if (raw === null) return fallback;
  return raw === "1";
}

/**
 * 应用设置。当前仅含 iCloud 下载策略，后续 PR 在此扩展（LLM provider 等）。
 */
export const useSettingsStore = defineStore("settings", () => {
  // iCloud 未下载文件策略：true=自动下载，false=仅提示（PRD 5.6 默认自动下载）
  const icloudAutoDownload = ref(readBool(STORAGE_KEY, true));

  watch(icloudAutoDownload, (v) => {
    localStorage.setItem(STORAGE_KEY, v ? "1" : "0");
  });

  return { icloudAutoDownload };
});
