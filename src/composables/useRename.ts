import { storeToRefs } from "pinia";
import { useSettingsStore } from "@/store/settings";
import type { AudioFileMeta } from "@/types/audio";

type RenameVar = "track" | "title" | "album" | "artist" | "ext";

// macOS Finder 不允许文件名含 / 与 :，替换为 -
function sanitize(s: string): string {
  return s.replace(/[/:]/g, "-");
}

/** 按模板渲染目标文件名：替换 {track}/{title}/{album}/{artist}/{ext} 占位符 */
export function renderName(file: AudioFileMeta, template: string): string {
  const ext = file.fileName.includes(".") ? (file.fileName.split(".").pop() ?? "") : "";
  const map: Record<RenameVar, string> = {
    track: file.track === null ? "" : String(file.track),
    title: file.title ?? "",
    album: file.album ?? "",
    artist: file.artist ?? "",
    ext,
  };
  return template.replace(/\{(track|title|album|artist|ext)\}/g, (_, key: RenameVar) =>
    sanitize(map[key]),
  );
}

/**
 * 重命名预览：基于设置中的开关与模板，给出文件的目标新名。
 * 开关关闭或缺少标题时返回 null（不展示预览）。
 */
export function useRename() {
  const settings = useSettingsStore();
  const { renameEnabled, renameTemplate } = storeToRefs(settings);

  function preview(file: AudioFileMeta): string | null {
    if (!renameEnabled.value) return null;
    if (!file.title) return null;
    return renderName(file, renameTemplate.value);
  }

  return { renameEnabled, renameTemplate, preview };
}
