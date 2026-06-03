import { describe, it, expect } from "vitest";
import { renderName } from "./useRename";
import type { AudioFileMeta } from "@/types/audio";

function file(overrides: Partial<AudioFileMeta> = {}): AudioFileMeta {
  return {
    path: "/dir/QA009 原始.mp3",
    fileName: "QA009 原始.mp3",
    title: "香港金像奖",
    album: "反派影评",
    artist: "波米",
    track: 9,
    durationSecs: 1200,
    embeddedCover: null,
    ...overrides,
  };
}

describe("renderName 重命名模板渲染", () => {
  // 覆盖 PRD 5.2 / 验收：至少支持 {track}{title}{album}{artist}{ext} 5 个变量
  it("替换全部 5 个占位符", () => {
    const out = renderName(file(), "{track} - {title} - {album} - {artist}.{ext}");
    expect(out).toBe("9 - 香港金像奖 - 反派影评 - 波米.mp3");
  });

  it("默认模板 {track} - {title}.{ext}", () => {
    expect(renderName(file(), "{track} - {title}.{ext}")).toBe("9 - 香港金像奖.mp3");
  });

  // 文件名含 / 与 : 会破坏 macOS 文件名，应被替换为 -
  it("清洗非法字符 / 与 :", () => {
    const out = renderName(file({ title: "上/下:集" }), "{title}.{ext}");
    expect(out).toBe("上-下-集.mp3");
  });

  // 缺失字段渲染为空串，不抛错
  it("track 为 null 渲染为空", () => {
    const out = renderName(file({ track: null }), "{track}-{title}.{ext}");
    expect(out).toBe("-香港金像奖.mp3");
  });

  it("title/album/artist 为 null 渲染为空", () => {
    const out = renderName(
      file({ title: null, album: null, artist: null }),
      "{title}{album}{artist}.{ext}",
    );
    expect(out).toBe(".mp3");
  });

  // 无扩展名文件 ext 为空
  it("无扩展名时 {ext} 为空", () => {
    const out = renderName(file({ fileName: "noext" }), "{title}.{ext}");
    expect(out).toBe("香港金像奖.");
  });
});
