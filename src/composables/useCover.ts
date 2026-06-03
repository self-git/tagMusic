import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { AudioFileMeta } from "@/types/audio";
import type { ProviderConfig } from "@/types/llm";
import type { CoverCandidates, CoverMatchResult } from "@/types/cover";
import { useAudioStore } from "@/store/audio";

// 与后端一致的可嵌入封面扩展名
const IMAGE_EXTENSIONS = ["jpg", "jpeg", "png"];

/**
 * 封面自动导入（v2 A）：扫描音频同目录候选图片 → AI 文本匹配 → 审核态写入 store。
 * 匹配为纯文本（候选文件名 + 已解析 title/album），best-effort，失败不阻断解析主流程。
 */
export function useCover() {
  const store = useAudioStore();
  const matching = ref(false);
  const error = ref<string | null>(null);

  // 读取图片为 data URL 供缩略图预览，失败返回 null（不阻断）
  async function thumb(image: string): Promise<string | null> {
    try {
      return await invoke<string>("read_image_data_url", { path: image });
    } catch {
      return null;
    }
  }

  // 扫描候选 + AI 匹配；应在 LLM 解析完成（title/album 已就绪）后调用
  async function scanAndMatch(files: AudioFileMeta[], config: ProviderConfig): Promise<void> {
    if (files.length === 0) return;
    matching.value = true;
    error.value = null;
    try {
      const audioPaths = files.map((f) => f.path);
      const candidates = await invoke<CoverCandidates[]>("scan_cover_candidates", { audioPaths });
      store.setCoverCandidates(candidates);

      const items = files.map((f) => ({
        path: f.path,
        title: f.title,
        album: f.album,
        candidates: candidates.find((c) => c.path === f.path)?.images ?? [],
      }));
      const results = await invoke<CoverMatchResult[]>("match_covers", { items, config });
      store.applyCoverMatches(results);

      // 为选中封面预取缩略图
      await Promise.all(
        results
          .filter((r): r is CoverMatchResult & { chosen: string } => r.chosen !== null)
          .map(async (r) => store.setCoverThumb(r.path, await thumb(r.chosen))),
      );
    } catch (e) {
      error.value = String(e);
    } finally {
      matching.value = false;
    }
  }

  // 手动为指定文件选择封面图片
  async function pickCover(path: string): Promise<void> {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Image", extensions: IMAGE_EXTENSIONS }],
    });
    if (selected === null) return;
    const image = Array.isArray(selected) ? selected[0] : selected;
    store.setCover(path, image, await thumb(image));
  }

  return { matching, error, scanAndMatch, pickCover };
}
