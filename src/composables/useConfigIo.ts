import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useSettingsStore, type ConfigSnapshot } from "@/store/settings";

// 内置密钥种子：用于对 API Key 做对称加密，使导出文件不明文存 Key
const KEY_SEED = "tagcast.config.v1.builtin-key";
// 加密串前缀（标识 AES-GCM 密文，便于导入时判断是否需要解密）
const ENC_PREFIX = "enc:v1:";
const IV_BYTES = 12;

// 导出文件结构
interface ConfigFile {
  version: 1;
  settings: ConfigSnapshot;
}

function bytesToBase64(bytes: Uint8Array): string {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s);
}

function base64ToBytes(b64: string): Uint8Array {
  const s = atob(b64);
  const out = new Uint8Array(s.length);
  for (let i = 0; i < s.length; i++) out[i] = s.charCodeAt(i);
  return out;
}

// 由固定种子派生 AES-GCM 密钥（SHA-256 → 256bit key）
async function deriveKey(): Promise<CryptoKey> {
  const seed = new TextEncoder().encode(KEY_SEED);
  const hash = await crypto.subtle.digest("SHA-256", seed);
  return crypto.subtle.importKey("raw", hash, "AES-GCM", false, ["encrypt", "decrypt"]);
}

async function encryptKey(plain: string): Promise<string> {
  const key = await deriveKey();
  const iv = crypto.getRandomValues(new Uint8Array(IV_BYTES));
  const ct = new Uint8Array(
    await crypto.subtle.encrypt({ name: "AES-GCM", iv }, key, new TextEncoder().encode(plain)),
  );
  const merged = new Uint8Array(iv.length + ct.length);
  merged.set(iv);
  merged.set(ct, iv.length);
  return ENC_PREFIX + bytesToBase64(merged);
}

async function decryptKey(blob: string): Promise<string> {
  if (!blob.startsWith(ENC_PREFIX)) return blob;
  const merged = base64ToBytes(blob.slice(ENC_PREFIX.length));
  const iv = merged.slice(0, IV_BYTES);
  const ct = merged.slice(IV_BYTES);
  const key = await deriveKey();
  const plain = await crypto.subtle.decrypt({ name: "AES-GCM", iv }, key, ct);
  return new TextDecoder().decode(plain);
}

/**
 * 设置导出/导入：写入/读取 JSON 配置文件（经 Rust 命令落盘）。
 * API Key 用内置密钥 AES-GCM 加密，导出文件不明文存 Key；导入时自动解密、整体覆盖。
 */
export function useConfigIo() {
  const settings = useSettingsStore();
  const busy = ref(false);
  const notice = ref<string | null>(null);
  const error = ref<string | null>(null);

  async function exportConfig(): Promise<void> {
    const path = await save({
      defaultPath: "tagcast-config.json",
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (path === null) return;
    busy.value = true;
    notice.value = null;
    error.value = null;
    try {
      const snapshot = settings.exportSnapshot();
      const apiKey = await encryptKey(snapshot.llmProvider.apiKey ?? "");
      const payload: ConfigFile = {
        version: 1,
        settings: { ...snapshot, llmProvider: { ...snapshot.llmProvider, apiKey } },
      };
      await invoke("write_text_file", { path, contents: JSON.stringify(payload, null, 2) });
      notice.value = "配置已导出";
    } catch (e) {
      error.value = `导出失败：${String(e)}`;
    } finally {
      busy.value = false;
    }
  }

  async function importConfig(): Promise<void> {
    const selected = await open({
      multiple: false,
      filters: [{ name: "JSON", extensions: ["json"] }],
    });
    if (selected === null) return;
    const path = Array.isArray(selected) ? selected[0] : selected;
    busy.value = true;
    notice.value = null;
    error.value = null;
    try {
      const text = await invoke<string>("read_text_file", { path });
      const payload = JSON.parse(text) as Partial<ConfigFile>;
      const incoming: Partial<ConfigSnapshot> = payload.settings ?? {};
      if (incoming.llmProvider && typeof incoming.llmProvider.apiKey === "string") {
        incoming.llmProvider = {
          ...incoming.llmProvider,
          apiKey: await decryptKey(incoming.llmProvider.apiKey),
        };
      }
      settings.importSnapshot(incoming);
      notice.value = "配置已导入";
    } catch (e) {
      error.value = `导入失败：${String(e)}`;
    } finally {
      busy.value = false;
    }
  }

  return { busy, notice, error, exportConfig, importConfig };
}
