/**
 * 节目档案，结构与 Rust 端 `list_show_profiles` 返回值一一对应（camelCase）。
 * 用户为每个节目设置一次元数据，AI 解析时自动匹配回填。
 */
export interface ShowProfile {
  id: number;
  /** 节目名（album），档案唯一标识 */
  album: string;
  /** 作者（artist），匹配命中时回填 */
  artist: string | null;
  /** 匹配关键词：命中原始文件名即视为该节目 */
  keywords: string[];
}

/** 保存档案的入参（对应 Rust 端 ShowProfileInput），id 为空表示新增 */
export interface ShowProfileInput {
  id: number | null;
  album: string;
  artist: string | null;
  keywords: string[];
}
