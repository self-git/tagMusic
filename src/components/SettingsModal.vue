<script setup lang="ts">
import { storeToRefs } from "pinia";
import { useSettingsStore } from "@/store/settings";
import type { ProviderConfig } from "@/types/llm";

defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: [] }>();

const settings = useSettingsStore();
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
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
    @click.self="emit('close')"
  >
    <div class="max-h-[85vh] w-[480px] overflow-y-auto rounded-xl border border-neutral-800 bg-neutral-900 p-5">
      <h2 class="mb-4 text-base font-semibold">LLM Provider 设置</h2>

      <div class="mb-4 flex gap-2">
        <button
          v-for="p in (['deepseek', 'openai', 'anthropic'] as Preset[])"
          :key="p"
          class="rounded-md border border-neutral-700 px-2.5 py-1 text-xs text-neutral-300 hover:bg-neutral-800"
          @click="applyPreset(p)"
        >
          {{ p === "deepseek" ? "DeepSeek" : p === "openai" ? "OpenAI 兼容" : "Anthropic" }}
        </button>
      </div>

      <div class="grid gap-3">
        <label class="block">
          <span class="text-xs text-neutral-500">协议类型</span>
          <select
            v-model="llmProvider.providerType"
            class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          >
            <option value="openai">OpenAI 兼容（含 DeepSeek）</option>
            <option value="anthropic">Anthropic 兼容</option>
          </select>
        </label>
        <label class="block">
          <span class="text-xs text-neutral-500">Base URL</span>
          <input
            v-model="llmProvider.baseUrl"
            type="text"
            class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
        </label>
        <label class="block">
          <span class="text-xs text-neutral-500">API Key</span>
          <input
            v-model="llmProvider.apiKey"
            type="password"
            placeholder="sk-..."
            class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
        </label>
        <label class="block">
          <span class="text-xs text-neutral-500">Model</span>
          <input
            v-model="llmProvider.model"
            type="text"
            class="mt-1 w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
        </label>
      </div>

      <div class="mt-5 border-t border-neutral-800 pt-4">
        <h3 class="mb-2 text-sm font-semibold">重命名模板</h3>
        <input
          v-model="renameTemplate"
          type="text"
          class="w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
        />
        <p class="mt-1.5 text-xs text-neutral-500">
          可用变量：{track} {title} {album} {artist} {ext}（在顶部开启「重命名」后预览生效）
        </p>
      </div>

      <div class="mt-5 border-t border-neutral-800 pt-4">
        <h3 class="mb-2 text-sm font-semibold">AI 解析提示词</h3>
        <p class="mb-3 text-xs text-neutral-500">
          自定义文件名解析的提示词；留空则使用默认。返回结构（results 数组）由程序固定，不受影响。
        </p>

        <div class="mb-3">
          <div class="mb-1 flex items-center justify-between">
            <span class="text-xs text-neutral-500">System Prompt</span>
            <button
              class="text-xs text-sky-400 hover:text-sky-300"
              @click="settings.resetParseField('systemPrompt')"
            >
              恢复默认
            </button>
          </div>
          <textarea
            v-model="parseConfig.systemPrompt"
            rows="6"
            class="w-full resize-y rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-xs outline-none focus:border-sky-500"
          ></textarea>
        </div>

        <div class="mb-3">
          <div class="mb-1 flex items-center justify-between">
            <span class="text-xs text-neutral-500">Few-shot 示例</span>
            <button
              class="text-xs text-sky-400 hover:text-sky-300"
              @click="settings.resetParseField('fewShot')"
            >
              恢复默认
            </button>
          </div>
          <textarea
            v-model="parseConfig.fewShot"
            rows="4"
            class="w-full resize-y rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-xs outline-none focus:border-sky-500"
          ></textarea>
        </div>

        <label class="block">
          <div class="mb-1 flex items-center justify-between">
            <span class="text-xs text-neutral-500">Temperature（0~2）</span>
            <button
              class="text-xs text-sky-400 hover:text-sky-300"
              @click="settings.resetParseField('temperature')"
            >
              恢复默认
            </button>
          </div>
          <input
            v-model.number="parseConfig.temperature"
            type="number"
            min="0"
            max="2"
            step="0.1"
            class="w-full rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
        </label>
      </div>

      <div class="mt-5 flex justify-end">
        <button
          class="rounded-lg bg-sky-600 px-4 py-2 text-sm font-medium text-white hover:bg-sky-500"
          @click="emit('close')"
        >
          完成
        </button>
      </div>
    </div>
  </div>
</template>
