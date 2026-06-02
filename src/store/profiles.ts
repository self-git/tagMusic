import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { ShowProfile, ShowProfileInput } from "@/types/profile";
import type { AudioFileMeta } from "@/types/audio";

function norm(s: string): string {
  return s.trim().toLowerCase();
}

/**
 * 节目档案库：持久化于 SQLite（经 Rust IPC），提供 CRUD + AI 解析后的自动匹配回填。
 * libraryOpen / draftAlbum 为档案库弹窗的跨视图状态（表格/向导均可触发新建）。
 */
export const useProfilesStore = defineStore("profiles", () => {
  const profiles = ref<ShowProfile[]>([]);
  // 档案库管理弹窗开关 + 引导新建时预填的节目名
  const libraryOpen = ref(false);
  const draftAlbum = ref<string | null>(null);

  async function load(): Promise<void> {
    profiles.value = await invoke<ShowProfile[]>("list_show_profiles");
  }

  async function save(input: ShowProfileInput): Promise<void> {
    await invoke<number>("save_show_profile", { profile: input });
    await load();
  }

  async function remove(id: number): Promise<void> {
    await invoke("delete_show_profile", { id });
    await load();
  }

  function openLibrary(album?: string): void {
    draftAlbum.value = album ?? null;
    libraryOpen.value = true;
  }

  function closeLibrary(): void {
    libraryOpen.value = false;
    draftAlbum.value = null;
  }

  // 匹配规则：解析出的 album 同名，或任一 keyword 命中原始文件名
  function match(file: AudioFileMeta): ShowProfile | null {
    const album = file.album ? norm(file.album) : "";
    const name = norm(file.fileName);
    for (const p of profiles.value) {
      if (album && norm(p.album) === album) return p;
      if (p.keywords.some((k) => k && name.includes(norm(k)))) return p;
    }
    return null;
  }

  // 解析后自动回填：命中档案的文件补全空缺的 album/artist；返回未命中的去重节目名（供引导新建）
  function autoFill(files: AudioFileMeta[]): string[] {
    const unmatched = new Set<string>();
    for (const f of files) {
      const p = match(f);
      if (p) {
        if (!f.album) f.album = p.album;
        if (!f.artist) f.artist = p.artist;
      } else if (f.album) {
        unmatched.add(f.album);
      }
    }
    return [...unmatched];
  }

  return {
    profiles,
    libraryOpen,
    draftAlbum,
    load,
    save,
    remove,
    openLibrary,
    closeLibrary,
    match,
    autoFill,
  };
});
