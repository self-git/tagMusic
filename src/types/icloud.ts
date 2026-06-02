/**
 * 文件的 iCloud 状态，对应 Rust 端 `check_icloud_status` 命令返回值（camelCase 对齐）。
 */
export interface ICloudStatus {
  path: string;
  /** 是否 iCloud 托管文件 */
  isUbiquitous: boolean;
  /** 是否已下载到本地（非 iCloud 文件恒为 true） */
  isDownloaded: boolean;
}
