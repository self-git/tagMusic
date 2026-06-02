import { defineStore } from "pinia";
import { ref, watch } from "vue";
import type { ProviderConfig } from "@/types/llm";

const ICLOUD_KEY = "tagcast.settings.icloudAutoDownload";
const LLM_KEY = "tagcast.settings.llmProvider";

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

  return { icloudAutoDownload, llmProvider };
});
