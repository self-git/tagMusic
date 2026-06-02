import { defineStore } from "pinia";
import { ref, watch } from "vue";
import type { ProviderConfig } from "@/types/llm";

const ICLOUD_KEY = "tagcast.settings.icloudAutoDownload";
const LLM_KEY = "tagcast.settings.llmProvider";
const RENAME_ENABLED_KEY = "tagcast.settings.renameEnabled";
const RENAME_TEMPLATE_KEY = "tagcast.settings.renameTemplate";

// 重命名模板默认值（PRD 5.2），支持 {track} {title} {album} {artist} {ext}
const DEFAULT_RENAME_TEMPLATE = "{track} - {title}.{ext}";

// 默认 provider：DeepSeek（OpenAI 兼容协议），用户填入自己的 API key
const DEFAULT_PROVIDER: ProviderConfig = {
  providerType: "openai",
  baseUrl: "https://api.deepseek.com",
  apiKey: "",
  model: "deepseek-chat",
};

function readBool(key: string, fallback: boolean): boolean {
  const raw = localStorage.getItem(key);
  if (raw === null) return fallback;
  return raw === "1";
}

function readString(key: string, fallback: string): string {
  return localStorage.getItem(key) ?? fallback;
}

function readProvider(): ProviderConfig {
  const raw = localStorage.getItem(LLM_KEY);
  if (raw === null) return { ...DEFAULT_PROVIDER };
  try {
    return { ...DEFAULT_PROVIDER, ...(JSON.parse(raw) as Partial<ProviderConfig>) };
  } catch {
    return { ...DEFAULT_PROVIDER };
  }
}

/**
 * 应用设置：iCloud 下载策略 + LLM provider 配置，均持久化到 localStorage。
 */
export const useSettingsStore = defineStore("settings", () => {
  // iCloud 未下载文件策略：true=自动下载，false=仅提示（PRD 5.6 默认自动下载）
  const icloudAutoDownload = ref(readBool(ICLOUD_KEY, true));
  // LLM provider 配置（含 API key），用于文件名解析
  const llmProvider = ref<ProviderConfig>(readProvider());
  // 重命名开关（PRD 5.2 默认关：只改元数据）+ 重命名模板
  const renameEnabled = ref(readBool(RENAME_ENABLED_KEY, false));
  const renameTemplate = ref(readString(RENAME_TEMPLATE_KEY, DEFAULT_RENAME_TEMPLATE));

  watch(icloudAutoDownload, (v) => {
    localStorage.setItem(ICLOUD_KEY, v ? "1" : "0");
  });

  watch(
    llmProvider,
    (v) => {
      localStorage.setItem(LLM_KEY, JSON.stringify(v));
    },
    { deep: true },
  );

  watch(renameEnabled, (v) => {
    localStorage.setItem(RENAME_ENABLED_KEY, v ? "1" : "0");
  });

  watch(renameTemplate, (v) => {
    localStorage.setItem(RENAME_TEMPLATE_KEY, v);
  });

  return { icloudAutoDownload, llmProvider, renameEnabled, renameTemplate };
});
