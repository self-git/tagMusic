/** 单个音频的候选封面，对应 Rust 端 CoverCandidates */
export interface CoverCandidates {
  path: string;
  images: string[];
  /** 同目录唯一 cover.* 且 <1MB 时的首选封面路径，前端直选、跳过 AI；否则 null */
  preferred: string | null;
}

/** 封面 AI 匹配结果，对应 Rust 端 CoverMatchResult。chosen 为选中的图片完整路径或 null */
export interface CoverMatchResult {
  path: string;
  chosen: string | null;
  confidence: number | null;
}

/**
 * 单个文件的封面选择状态（前端审核态，不直接对应后端）。
 * - chosen：将写入的封面图片路径；null 表示不写
 * - cleared：用户显式清除（即使文件原有封面也清掉），与 chosen=null 的"无匹配"区分
 * - thumb：chosen 的 data URL，用于缩略图预览
 */
export interface CoverSelection {
  candidates: string[];
  chosen: string | null;
  confidence: number | null;
  cleared: boolean;
  thumb: string | null;
}
