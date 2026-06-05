<script setup lang="ts">
import { computed, ref } from "vue";
import { storeToRefs } from "pinia";
import { Bot, Sparkles, PenLine, Filter, Database, X } from "lucide-vue-next";
import { useSettingsStore } from "@/store/settings";
import { useConfigIo } from "@/composables/useConfigIo";
import RuleEditor from "@/components/RuleEditor.vue";
import type { ProviderConfig } from "@/types/llm";

defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: [] }>();

const settings = useSettingsStore();
// 配置导出/导入（API Key 加密；导入整体覆盖）
const { busy: ioBusy, notice: ioNotice, error: ioError, exportConfig, importConfig } = useConfigIo();
// LLM provider 配置 + 重命名模板 + 解析提示词（来源：设置 store，localStorage 持久化）
const { llmProvider, renameTemplate, parseConfig } = storeToRefs(settings);

type Preset = "deepseek" | "openai" | "anthropic";

// 预设仅替换协议/地址/模型，保留用户已填的 apiKey
function applyPreset(preset: Preset): void {
  const key = llmProvider.value.apiKey;
  const presets: Record<Preset, ProviderConfig> = {
    deepseek: { providerType: "openai", baseUrl: "https://api.deepseek.com", apiKey: key, model: "deepseek-chat" },
    openai: { providerType: "openai", baseUrl: "https://api.openai.com/v1", apiKey: key, model: "gpt-4o-mini" },
    anthropic: { providerType: "anthropic", baseUrl: "https://api.anthropic.com", apiKey: key, model: "claude-3-5-sonnet-latest" },
  };
  llmProvider.value = presets[preset];
}

// 左侧分类导航：id 决定右侧渲染哪块内容（active 为当前选中分类）
type Tab = "provider" | "parse" | "rename" | "rules" | "data";
const tabs = [
  { id: "provider", label: "AI 服务", desc: "模型供应商与密钥", icon: Bot, tile: "bg-indigo-500" },
  { id: "parse", label: "AI 解析", desc: "文件名解析提示词", icon: Sparkles, tile: "bg-purple-500" },
  { id: "rename", label: "重命名", desc: "文件重命名模板", icon: PenLine, tile: "bg-orange-500" },
  { id: "rules", label: "匹配规则", desc: "文件名本地匹配规则", icon: Filter, tile: "bg-sky-500" },
  { id: "data", label: "数据管理", desc: "导出 / 导入配置", icon: Database, tile: "bg-emerald-500" },
] as const;
const active = ref<Tab>("provider");
const activeMeta = computed(() => tabs.find((t) => t.id === active.value));
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-6"
    @click.self="emit('close')"
    @keydown.esc.window="emit('close')"
  >
    <div
      class="flex h-[90vh] w-[92vw] max-w-[1100px] overflow-hidden rounded-2xl border border-line bg-surface shadow-2xl"
    >
      <!-- 左侧分类导航 -->
      <aside class="flex w-[228px] shrink-0 flex-col border-r border-line bg-base">
        <div class="px-5 pb-2 pt-5">
          <h2 class="text-lg font-semibold">设置</h2>
        </div>
        <nav class="flex-1 space-y-1 overflow-auto px-3 py-2">
          <button
            v-for="t in tabs"
            :key="t.id"
            class="flex w-full items-center gap-3 rounded-lg px-2.5 py-2 text-left transition"
            :class="
              active === t.id
                ? 'bg-accent text-white'
                : 'text-strong hover:bg-elevated'
            "
            @click="active = t.id"
          >
            <span
              class="flex h-7 w-7 shrink-0 items-center justify-center rounded-md"
              :class="t.tile"
            >
              <component :is="t.icon" class="h-4 w-4 text-white" />
            </span>
            <span class="min-w-0">
              <span class="block truncate text-sm font-medium">{{ t.label }}</span>
            </span>
          </button>
        </nav>
      </aside>

      <!-- 右侧内容区 -->
      <section class="flex min-w-0 flex-1 flex-col">
        <header class="flex items-center justify-between border-b border-line px-6 py-4">
          <div>
            <h3 class="text-base font-semibold">{{ activeMeta?.label }}</h3>
            <p class="text-xs text-faint">{{ activeMeta?.desc }}</p>
          </div>
          <button
            class="flex h-8 w-8 items-center justify-center rounded-lg text-faint transition hover:bg-elevated hover:text-strong"
            title="关闭（Esc）"
            @click="emit('close')"
          >
            <X class="h-5 w-5" />
          </button>
        </header>

        <div class="min-h-0 flex-1 overflow-auto px-6 py-5">
          <!-- AI 服务 -->
          <div v-show="active === 'provider'" class="max-w-2xl">
            <div class="mb-4 flex flex-wrap gap-2">
              <button
                v-for="p in (['deepseek', 'openai', 'anthropic'] as Preset[])"
                :key="p"
                class="rounded-lg border border-edge px-3 py-1.5 text-xs text-muted transition hover:bg-elevated"
                @click="applyPreset(p)"
              >
                {{ p === "deepseek" ? "DeepSeek" : p === "openai" ? "OpenAI 兼容" : "Anthropic" }}
              </button>
            </div>

            <div class="grid gap-4">
              <label class="block">
                <span class="text-xs font-medium text-muted">协议类型</span>
                <select
                  v-model="llmProvider.providerType"
                  class="mt-1.5 w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
                >
                  <option value="openai">OpenAI 兼容（含 DeepSeek）</option>
                  <option value="anthropic">Anthropic 兼容</option>
                </select>
              </label>
              <label class="block">
                <span class="text-xs font-medium text-muted">Base URL</span>
                <input
                  v-model="llmProvider.baseUrl"
                  type="text"
                  class="mt-1.5 w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
                />
              </label>
              <label class="block">
                <span class="text-xs font-medium text-muted">API Key</span>
                <input
                  v-model="llmProvider.apiKey"
                  type="password"
                  placeholder="sk-..."
                  class="mt-1.5 w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
                />
              </label>
              <label class="block">
                <span class="text-xs font-medium text-muted">Model</span>
                <input
                  v-model="llmProvider.model"
                  type="text"
                  class="mt-1.5 w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
                />
              </label>
            </div>
          </div>

          <!-- AI 解析 -->
          <div v-show="active === 'parse'" class="max-w-2xl">
            <p class="mb-4 text-xs text-faint">
              自定义文件名解析的提示词；留空则使用默认。返回结构（results 数组）由程序固定，不受影响。
            </p>

            <div class="mb-4">
              <div class="mb-1.5 flex items-center justify-between">
                <span class="text-xs font-medium text-muted">System Prompt</span>
                <button class="text-xs text-accent-fg hover:underline" @click="settings.resetParseField('systemPrompt')">
                  恢复默认
                </button>
              </div>
              <textarea
                v-model="parseConfig.systemPrompt"
                rows="8"
                class="w-full resize-y rounded-lg border border-edge bg-field px-3 py-2 text-xs outline-none focus:border-accent"
              ></textarea>
            </div>

            <div class="mb-4">
              <div class="mb-1.5 flex items-center justify-between">
                <span class="text-xs font-medium text-muted">Few-shot 示例</span>
                <button class="text-xs text-accent-fg hover:underline" @click="settings.resetParseField('fewShot')">
                  恢复默认
                </button>
              </div>
              <textarea
                v-model="parseConfig.fewShot"
                rows="5"
                class="w-full resize-y rounded-lg border border-edge bg-field px-3 py-2 text-xs outline-none focus:border-accent"
              ></textarea>
            </div>

            <label class="block max-w-[200px]">
              <div class="mb-1.5 flex items-center justify-between">
                <span class="text-xs font-medium text-muted">Temperature（0~1）</span>
                <button class="text-xs text-accent-fg hover:underline" @click="settings.resetParseField('temperature')">
                  恢复默认
                </button>
              </div>
              <input
                v-model.number="parseConfig.temperature"
                type="number"
                min="0"
                max="2"
                step="0.1"
                class="w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
              />
            </label>
          </div>

          <!-- 重命名 -->
          <div v-show="active === 'rename'" class="max-w-2xl">
            <label class="block">
              <span class="text-xs font-medium text-muted">重命名模板</span>
              <input
                v-model="renameTemplate"
                type="text"
                class="mt-1.5 w-full rounded-lg border border-edge bg-field px-3 py-2 text-sm outline-none focus:border-accent"
              />
            </label>
            <p class="mt-2 text-xs text-faint">
              可用变量：{track} {title} {album} {artist} {ext}（在顶部开启「重命名」后预览生效）
            </p>
          </div>

          <!-- 匹配规则 -->
          <div v-show="active === 'rules'">
            <RuleEditor />
          </div>

          <!-- 数据管理：导出 / 导入配置（API Key 加密，导入整体覆盖） -->
          <div v-show="active === 'data'" class="max-w-2xl">
            <p class="mb-4 text-xs text-faint">
              导出当前全部设置为 JSON 文件（API Key 会加密存储），可在其他设备通过导入整体恢复。不含节目档案库。
            </p>
            <div class="flex flex-wrap gap-3">
              <button
                class="rounded-lg border border-edge px-4 py-2 text-sm text-strong transition hover:bg-elevated disabled:opacity-50"
                :disabled="ioBusy"
                @click="exportConfig"
              >
                导出数据
              </button>
              <button
                class="rounded-lg border border-edge px-4 py-2 text-sm text-strong transition hover:bg-elevated disabled:opacity-50"
                :disabled="ioBusy"
                @click="importConfig"
              >
                导入数据
              </button>
            </div>
            <p v-if="ioNotice" class="mt-3 text-xs text-success-fg">{{ ioNotice }}</p>
            <p v-if="ioError" class="mt-3 text-xs text-danger-fg">{{ ioError }}</p>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>
