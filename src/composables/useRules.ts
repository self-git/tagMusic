import { RULE_FIELDS, type FilenameRule, type RuleField, type RuleHint, type RuleMatch } from "@/types/rule";

// 字段中文标签，仅用于拼接给 AI 的规则描述
const FIELD_LABEL: Record<RuleField, string> = {
  title: "标题",
  album: "节目",
  artist: "作者",
  track: "集",
};

/**
 * 文件名规则本地匹配引擎（问题 2）。纯函数，无副作用，供解析链路与规则编辑器预览复用。
 * 执行策略：字段级叠加——按规则数组顺序(即优先级)依次跑 enabled 规则，高优先级已填字段不被覆盖。
 */

// 去掉最后一个扩展名（如 .mp3）；无扩展名则原样返回
function stripExtension(fileName: string): string {
  const i = fileName.lastIndexOf(".");
  return i > 0 ? fileName.slice(0, i) : fileName;
}

// 从字符串提取首个整数作为集数（无数字返回 null）
function parseTrack(s: string): number | null {
  const m = s.match(/\d+/);
  return m ? Number.parseInt(m[0], 10) : null;
}

// 文本字段赋值：trim 后非空才算命中
function assignText(out: RuleMatch, field: "title" | "album" | "artist", value: string | undefined): void {
  const v = value?.trim();
  if (v) out[field] = v;
}

// 集数赋值：从原始片段提取整数，提取到才算命中
function assignTrack(out: RuleMatch, value: string | undefined): void {
  if (value === undefined) return;
  const t = parseTrack(value);
  if (t !== null) out.track = t;
}

// 固定值赋值：命中后用常量覆盖指定字段（捕获/分段做不到「赋常量」，由此补足）
function applyConstants(out: RuleMatch, constants: Partial<Record<RuleField, string>> | undefined): void {
  if (!constants) return;
  assignText(out, "title", constants.title);
  assignText(out, "album", constants.album);
  assignText(out, "artist", constants.artist);
  assignTrack(out, constants.track);
}

/** 单条规则匹配文件名 → 命中的字段（不含未命中字段的键）。未命中时固定值不应用。 */
export function matchRule(rule: FilenameRule, fileName: string): RuleMatch {
  const base = stripExtension(fileName);
  const out: RuleMatch = {};
  if (rule.type === "regex") {
    if (rule.pattern.length === 0) return out;
    let re: RegExp;
    try {
      re = new RegExp(rule.pattern);
    } catch {
      return out; // 非法正则视为不命中（编辑器会单独提示语法错误）
    }
    // 正则整体匹配作为命中条件；命中后捕获组提取字段，再用固定值覆盖
    const m = re.exec(base);
    if (m === null) return out;
    const groups = m.groups;
    if (groups) {
      assignText(out, "title", groups.title);
      assignText(out, "album", groups.album);
      assignText(out, "artist", groups.artist);
      assignTrack(out, groups.track);
    }
    applyConstants(out, rule.constants);
    return out;
  }
  // separator 规则：分隔符存在才算命中；命中后按段映射，再用固定值覆盖
  if (rule.separator.length === 0) return out;
  if (!base.includes(rule.separator)) return out;
  const segs = base.split(rule.separator);
  for (const [field, idx] of Object.entries(rule.mapping)) {
    if (idx === undefined) continue;
    const seg = segs[idx];
    if (field === "track") assignTrack(out, seg);
    else assignText(out, field as "title" | "album" | "artist", seg);
  }
  applyConstants(out, rule.constants);
  return out;
}

// 单条规则 → 单行中文描述（注入 AI 用）
function ruleDetail(rule: FilenameRule): string {
  let base: string;
  if (rule.type === "regex") {
    base = `正则 ${rule.pattern}`;
  } else {
    const parts = Object.entries(rule.mapping)
      .filter(([, idx]) => idx !== undefined)
      .map(([field, idx]) => `${FIELD_LABEL[field as RuleField]}=第${idx}段`);
    base = parts.length > 0 ? `用「${rule.separator}」切分，${parts.join("，")}` : `含「${rule.separator}」`;
  }
  // 固定值一并注入 AI 作提示
  const consts = RULE_FIELDS.filter((f) => (rule.constants?.[f] ?? "").trim()).map(
    (f) => `${FIELD_LABEL[f]}=${rule.constants?.[f]}`,
  );
  return consts.length > 0 ? `${base}；命中则固定 ${consts.join("，")}` : base;
}

/** 把 enabled 规则按当前顺序(优先级) 转成注入 AI 的提示清单 */
export function toRuleHints(rules: FilenameRule[]): RuleHint[] {
  return rules.filter((r) => r.enabled).map((r) => ({ name: r.name, detail: ruleDetail(r) }));
}

/** 全部规则按优先级字段级叠加匹配（已填字段不被低优先级覆盖） */
export function applyRules(fileName: string, rules: FilenameRule[]): RuleMatch {
  const out: RuleMatch = {};
  for (const rule of rules) {
    if (!rule.enabled) continue;
    const m = matchRule(rule, fileName);
    if (out.title === undefined && m.title !== undefined) out.title = m.title;
    if (out.album === undefined && m.album !== undefined) out.album = m.album;
    if (out.artist === undefined && m.artist !== undefined) out.artist = m.artist;
    if (out.track === undefined && m.track !== undefined) out.track = m.track;
  }
  return out;
}
