import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useProfilesStore } from "./profiles";
import type { ShowProfile } from "@/types/profile";
import type { AudioFileMeta } from "@/types/audio";

function profile(overrides: Partial<ShowProfile> = {}): ShowProfile {
  return { id: 1, album: "反派影评", artist: "波米", keywords: [], ...overrides };
}

function file(overrides: Partial<AudioFileMeta> = {}): AudioFileMeta {
  return {
    path: "/dir/x.mp3",
    fileName: "x.mp3",
    title: null,
    album: null,
    artist: null,
    track: null,
    durationSecs: null,
    ...overrides,
  };
}

describe("profiles store 节目档案匹配", () => {
  let store: ReturnType<typeof useProfilesStore>;

  beforeEach(() => {
    setActivePinia(createPinia());
    store = useProfilesStore();
  });

  // 规则一：解析出的 album 与档案 album 同名（忽略大小写/空白）即命中
  it("按 album 同名匹配", () => {
    store.profiles = [profile()];
    const m = store.match(file({ album: " 反派影评 " }));
    expect(m?.id).toBe(1);
  });

  // 规则二：任一 keyword 命中原始文件名即视为该节目
  it("按 keyword 命中文件名匹配", () => {
    store.profiles = [profile({ keywords: ["爱发电"] })];
    const m = store.match(file({ fileName: "QA009丨反派影评丨爱发电.mp3" }));
    expect(m?.id).toBe(1);
  });

  it("无 album 无 keyword 命中时返回 null", () => {
    store.profiles = [profile({ keywords: ["爱发电"] })];
    expect(store.match(file({ fileName: "无关文件.mp3", album: "别的节目" }))).toBeNull();
  });

  // autoFill：命中档案的文件补全空缺 album/artist，返回未命中的去重节目名
  it("autoFill 回填空缺字段并返回未命中节目名", () => {
    store.profiles = [profile()];
    const files = [
      file({ album: "反派影评" }), // 命中：artist 应被回填
      file({ album: "反派影评", artist: "已有作者" }), // 命中但 artist 已有，不覆盖
      file({ album: "未知节目A" }), // 未命中
      file({ album: "未知节目A" }), // 未命中（去重）
    ];

    const unmatched = store.autoFill(files);

    expect(files[0].artist).toBe("波米");
    expect(files[1].artist).toBe("已有作者");
    expect(unmatched).toEqual(["未知节目A"]);
  });
});
