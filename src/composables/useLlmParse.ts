import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ParseResult, ProviderConfig, ParseConfig } from "@/types/llm";
import type { AudioFileMeta } from "@/types/audio";
import type { RuleHint } from "@/types/rule";

/**
 * 调用后端 LLM 批量解析脏文件名 → 元数据。所有文件名一次性提交（满足 30 文件/60 秒目标）。
 * rules：用户规则提示，按优先级结构化注入 prompt（问题 2，best-effort 仅供 AI 参考）。
 */
export function useLlmParse() {
  const parsing = ref(false);
  const error = ref<string | null>(null);

  async function parse(
    files: AudioFileMeta[],
    config: ProviderConfig,
    parseConfig?: ParseConfig,
    rules?: RuleHint[],
  ): Promise<ParseResult[]> {
    if (files.length === 0) return [];
    parsing.value = true;
    error.value = null;
    try {
      const inputs = files.map((f) => ({ path: f.path, fileName: f.fileName }));
      return await invoke<ParseResult[]>("parse_filenames", {
        files: inputs,
        config,
        parseConfig,
        rules,
      });
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      parsing.value = false;
    }
  }

  return { parsing, error, parse };
}
