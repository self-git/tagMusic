/** LLM 协议类型：openai 覆盖 DeepSeek / OpenAI 兼容；anthropic 为 Anthropic 兼容 */
export type ProviderType = "openai" | "anthropic";

/** LLM provider 配置，整体传给 Rust 端 parse_filenames（camelCase 对齐） */
export interface ProviderConfig {
  providerType: ProviderType;
  baseUrl: string;
  apiKey: string;
  model: string;
}

/** 单个文件名的 LLM 解析结果，对应 Rust 端 ParseResult */
export interface ParseResult {
  path: string;
  title: string | null;
  album: string | null;
  artist: string | null;
  track: number | null;
  confidence: number | null;
}
