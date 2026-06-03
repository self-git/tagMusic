import { defineStore } from "pinia";
import { ref, watch } from "vue";
import type { ProviderConfig, ParseConfig } from "@/types/llm";
import type { FilenameRule } from "@/types/rule";

const ICLOUD_KEY = "tagcast.settings.icloudAutoDownload";
const LLM_KEY = "tagcast.settings.llmProvider";
const RENAME_ENABLED_KEY = "tagcast.settings.renameEnabled";
const RENAME_TEMPLATE_KEY = "tagcast.settings.renameTemplate";
const PARSE_CONFIG_KEY = "tagcast.settings.parseConfig";
const RULES_KEY = "tagcast.settings.filenameRules";

// 重命名模板默认值（PRD 5.2），支持 {track} {title} {album} {artist} {ext}
const DEFAULT_RENAME_TEMPLATE = "{track} - {title}.{ext}";

// 默认解析提示词（与后端 llm.rs DEFAULT_SYSTEM_PROMPT / DEFAULT_FEW_SHOT 保持一致；
// 用户可在设置中编辑，留空时后端回落同样的默认值）。
const DEFAULT_SYSTEM_PROMPT = `你是播客元数据提取助手。从用户给出的脏文件名中提取四个字段：title(标题)、album(节目名)、artist(作者)、track(集数，整数或 null)。
要点：
- 剥离平台/网站标识（如 爱发电、知乎、喜马拉雅、小宇宙、b站、公众号 等）及其分隔符（丨 | - · _ [ ] （） 等）。
- title 取核心标题，不含平台后缀。
- 无法确定的字段返回 null，不要臆造。
- track 仅在文件名含明确集数编号时填整数。
- confidence 为 0~1 的置信度。
只输出 JSON 对象，不要任何多余文字或 markdown。`;

const DEFAULT_FEW_SHOT = `示例：
输入 \`QA009：香港金像奖·国产片含男量丨反派影评丨爱发电.mp3\`
输出 {"index":0,"title":"QA009：香港金像奖·国产片含男量","album":"反派影评","artist":null,"track":9,"confidence":0.86}`;

export const DEFAULT_PARSE_CONFIG: ParseConfig = {
  systemPrompt: DEFAULT_SYSTEM_PROMPT,
  fewShot: DEFAULT_FEW_SHOT,
  temperature: 0,
};

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

function readParseConfig(): ParseConfig {
  const raw = localStorage.getItem(PARSE_CONFIG_KEY);
  if (raw === null) return { ...DEFAULT_PARSE_CONFIG };
  try {
    return { ...DEFAULT_PARSE_CONFIG, ...(JSON.parse(raw) as Partial<ParseConfig>) };
  } catch {
    return { ...DEFAULT_PARSE_CONFIG };
  }
}

// 文件名规则默认空数组（无规则时解析链路等价于纯 AI，向后兼容）
function readRules(): FilenameRule[] {
  const raw = localStorage.getItem(RULES_KEY);
  if (raw === null) return [];
  try {
    const parsed = JSON.parse(raw);
    return Array.isArray(parsed) ? (parsed as FilenameRule[]) : [];
  } catch {
    return [];
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
  // LLM 解析提示词自定义（v2 B 项），随 parse_filenames 透传后端
  const parseConfig = ref<ParseConfig>(readParseConfig());
  // 文件名匹配规则（问题 2）：数组顺序即优先级，本地匹配 + 结构化注入 AI
  const rules = ref<FilenameRule[]>(readRules());

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

  watch(
    parseConfig,
    (v) => {
      localStorage.setItem(PARSE_CONFIG_KEY, JSON.stringify(v));
    },
    { deep: true },
  );

  watch(
    rules,
    (v) => {
      localStorage.setItem(RULES_KEY, JSON.stringify(v));
    },
    { deep: true },
  );

  // 恢复某一项解析配置到默认值（设置面板"恢复默认"用）
  function resetParseField(field: keyof ParseConfig): void {
    parseConfig.value = { ...parseConfig.value, [field]: DEFAULT_PARSE_CONFIG[field] };
  }

  return {
    icloudAutoDownload,
    llmProvider,
    renameEnabled,
    renameTemplate,
    parseConfig,
    rules,
    resetParseField,
  };
});
