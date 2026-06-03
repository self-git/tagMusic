/**
 * 文件名匹配规则模型（问题 2）。规则按数组顺序即优先级（越靠前优先级越高），
 * 本地按字段级叠加执行：高优先级已填字段不被低优先级覆盖，跑完仍空缺的字段交给 AI 补。
 */

/** 可由规则映射的元数据字段 */
export type RuleField = "title" | "album" | "artist" | "track";

/** 字段遍历顺序（本地引擎/UI 复用，避免散落硬编码） */
export const RULE_FIELDS = ["title", "album", "artist", "track"] as const;

interface BaseRule {
  /** 稳定唯一 id（拖拽/列表 key 用） */
  id: string;
  /** 规则名称，仅展示 */
  name: string;
  /** 关闭后本地匹配与 AI 注入都跳过该规则 */
  enabled: boolean;
  /**
   * 固定值（常量赋值）：规则命中后为这些字段写入常量，覆盖捕获组/分段结果。
   * 用于「条件 → 常量映射」场景（如 含 QA\d{3} → 节目=会员问答），捕获组只能提取已有文本、无法赋常量。
   * track 以字符串存储，匹配时解析为整数；留空的字段忽略。
   */
  constants?: Partial<Record<RuleField, string>>;
}

/** 分隔符规则：用 separator 切分文件名（已去扩展名）→ 第 N 段(0 起) 映射到字段 */
export interface SeparatorRule extends BaseRule {
  type: "separator";
  separator: string;
  mapping: Partial<Record<RuleField, number>>;
}

/** 正则规则：用命名捕获组 (?<title>)/(?<album>)/(?<artist>)/(?<track>) 提取字段 */
export interface RegexRule extends BaseRule {
  type: "regex";
  pattern: string;
}

/** 一条文件名规则（判别联合：按 type 收窄） */
export type FilenameRule = SeparatorRule | RegexRule;

/** 单条/全部规则匹配出的字段（字段级叠加用；未命中的字段缺省不含该键） */
export type RuleMatch = Partial<{
  title: string;
  album: string;
  artist: string;
  track: number;
}>;

/** 注入 AI 的规则提示（对应 Rust 端 RuleHint）：name + 单行描述，顺序即优先级 */
export interface RuleHint {
  name: string;
  detail: string;
}
