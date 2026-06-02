/** 写回入参，对应 Rust 端 WriteInput（camelCase）。newName 为空表示不重命名 */
export interface WriteInput {
  path: string;
  title: string | null;
  album: string | null;
  artist: string | null;
  track: number | null;
  newName: string | null;
}

/** 写回结果：旧路径 → 新路径（未重命名时相同），对应 Rust 端 WriteOutcome */
export interface WriteOutcome {
  oldPath: string;
  newPath: string;
}

/** 重置结果：当前路径 → 恢复后路径 + 原始四字段，对应 Rust 端 ResetOutcome */
export interface ResetOutcome {
  currentPath: string;
  restoredPath: string;
  title: string | null;
  album: string | null;
  artist: string | null;
  track: number | null;
}
