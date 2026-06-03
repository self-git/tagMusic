/**
 * 单个音频文件的元数据，结构与 Rust 端 `read_audio_metadata` 命令返回值一一对应。
 * 字段命名与 Rust 的 `#[serde(rename_all = "camelCase")]` 保持一致。
 */
export interface AudioFileMeta {
  /** 文件绝对路径（同时作为去重主键） */
  path: string;
  /** 原始文件名（含扩展名），AI 解析的输入来源 */
  fileName: string;
  title: string | null;
  album: string | null;
  artist: string | null;
  track: number | null;
  /** 时长（秒），读取标签时一并取出，仅展示用 */
  durationSecs: number | null;
  /** 文件内嵌封面的缩略图 data URL（无则 null）：封面列展示的基准层，AI/手动匹配叠加其上 */
  embeddedCover: string | null;
}
