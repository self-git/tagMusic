<script setup lang="ts">
import { ref, computed } from "vue";
import { storeToRefs } from "pinia";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "@/store/settings";
import { useAudioStore } from "@/store/audio";
import { matchRule } from "@/composables/useRules";
import { RULE_FIELDS, type FilenameRule, type RuleField, type SeparatorRule } from "@/types/rule";

const settings = useSettingsStore();
const { rules } = storeToRefs(settings);
const audio = useAudioStore();

// 字段中文标签（UI 展示）
const FIELD_LABEL: Record<RuleField, string> = { title: "标题", album: "节目", artist: "作者", track: "集" };

// AI 生成规则的自然语言描述，按规则 id 暂存（不持久化）
const descById = ref<Record<string, string>>({});
// 每条规则 AI 生成中的状态 + 错误
const generating = ref<Record<string, boolean>>({});
const genError = ref<string | null>(null);
// 无导入文件时的手动预览测试文件名
const testName = ref("");
// 拖拽源索引
const dragIndex = ref<number | null>(null);

// 预览用样本文件名：优先已导入文件（最多 6 条），否则用手动测试输入
const sampleNames = computed<string[]>(() => {
  const loaded = audio.files.map((f) => f.fileName).slice(0, 6);
  if (loaded.length > 0) return loaded;
  return testName.value.trim() ? [testName.value.trim()] : [];
});

function newId(): string {
  return crypto.randomUUID();
}

function addRule(type: FilenameRule["type"]): void {
  const base = { id: newId(), name: type === "regex" ? "正则规则" : "分隔规则", enabled: true };
  const rule: FilenameRule =
    type === "regex"
      ? { ...base, type: "regex", pattern: "" }
      : { ...base, type: "separator", separator: "丨", mapping: {} };
  rules.value = [...rules.value, rule];
}

function removeRule(id: string): void {
  rules.value = rules.value.filter((r) => r.id !== id);
}

// 上移/下移：调整数组顺序即调整优先级
function move(index: number, dir: -1 | 1): void {
  const target = index + dir;
  if (target < 0 || target >= rules.value.length) return;
  const next = [...rules.value];
  [next[index], next[target]] = [next[target], next[index]];
  rules.value = next;
}

// WebKit(WKWebView) 下必须在 dragstart 写入 dataTransfer，否则拖拽不发起、drop 不触发
function onDragStart(event: DragEvent, index: number): void {
  dragIndex.value = index;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = "move";
    event.dataTransfer.setData("text/plain", String(index));
  }
}

function onDrop(target: number): void {
  const from = dragIndex.value;
  dragIndex.value = null;
  if (from === null || from === target) return;
  const next = [...rules.value];
  const [moved] = next.splice(from, 1);
  next.splice(target, 0, moved);
  rules.value = next;
}

// 分隔规则段映射：value 为空字符串则删除该字段映射
function setSegment(rule: SeparatorRule, field: RuleField, value: string): void {
  const next = { ...rule.mapping };
  if (value === "") delete next[field];
  else next[field] = Math.max(0, Math.floor(Number(value)));
  rule.mapping = next;
}

// 固定值：命中后写入的常量；空值删除该字段
function setConstant(rule: FilenameRule, field: RuleField, value: string): void {
  const next = { ...(rule.constants ?? {}) };
  if (value.trim() === "") delete next[field];
  else next[field] = value;
  rule.constants = next;
}

// 正则语法校验：非法返回错误信息（含重复命名组），供编辑器提示，避免静默当作未命中
function regexError(rule: FilenameRule): string | null {
  if (rule.type !== "regex" || rule.pattern.length === 0) return null;
  try {
    new RegExp(rule.pattern);
    return null;
  } catch (e) {
    return e instanceof Error ? e.message : String(e);
  }
}

// 后端 generate_filename_rule 返回结构
interface GenerateRuleResult {
  pattern?: string | null;
  separator?: string | null;
  mapping?: Record<string, number> | null;
  constants?: Record<string, string> | null;
}

// AI 生成规则：用自然语言 + 已导入文件名样本，填回当前规则配置（含固定值）
async function generate(rule: FilenameRule): Promise<void> {
  const description = (descById.value[rule.id] ?? "").trim();
  if (!description) return;
  generating.value = { ...generating.value, [rule.id]: true };
  genError.value = null;
  try {
    const result = await invoke<GenerateRuleResult>("generate_filename_rule", {
      description,
      ruleType: rule.type,
      samples: sampleNames.value,
      config: settings.llmProvider,
    });
    if (rule.type === "regex") {
      if (result.pattern) rule.pattern = result.pattern;
    } else {
      if (result.separator) rule.separator = result.separator;
      if (result.mapping) {
        const mapping: SeparatorRule["mapping"] = {};
        for (const f of RULE_FIELDS) {
          const idx = result.mapping[f];
          if (typeof idx === "number") mapping[f] = idx;
        }
        rule.mapping = mapping;
      }
    }
    // 固定值：两种规则通用，AI 给出则回填（覆盖原固定值；未给出则保留）
    if (result.constants) {
      const constants: Partial<Record<RuleField, string>> = {};
      for (const f of RULE_FIELDS) {
        const v = result.constants[f];
        if (typeof v === "string" && v.trim()) constants[f] = v;
      }
      rule.constants = constants;
    }
  } catch (e) {
    genError.value = String(e);
  } finally {
    generating.value = { ...generating.value, [rule.id]: false };
  }
}

// 预览：把规则套到样本文件名上，返回命中的字段（供编辑器即时反馈）
function previewLine(rule: FilenameRule, fileName: string): string {
  if (regexError(rule) !== null) return "⚠ 正则语法错误";
  const m = matchRule(rule, fileName);
  const parts = RULE_FIELDS.filter((f) => m[f] !== undefined).map((f) => `${FIELD_LABEL[f]}=${m[f]}`);
  return parts.length > 0 ? parts.join("  ") : "（未命中）";
}
</script>

<template>
  <div>
    <div class="mb-2 flex items-center justify-between">
      <h3 class="text-sm font-semibold">文件名匹配规则</h3>
      <div class="flex gap-2">
        <button
          class="rounded-md border border-neutral-700 px-2.5 py-1 text-xs text-neutral-300 hover:bg-neutral-800"
          @click="addRule('separator')"
        >
          + 分隔规则
        </button>
        <button
          class="rounded-md border border-neutral-700 px-2.5 py-1 text-xs text-neutral-300 hover:bg-neutral-800"
          @click="addRule('regex')"
        >
          + 正则规则
        </button>
      </div>
    </div>
    <p class="mb-3 text-xs text-neutral-500">
      解析时本地规则按优先级（从上到下）字段级叠加先跑，仍空缺的字段才交给 AI；规则也会按优先级注入 AI 提示。拖拽或用 ↑↓ 调整优先级。
    </p>

    <p v-if="genError" class="mb-2 rounded-md bg-red-950/40 px-2 py-1 text-xs text-red-300">
      AI 生成规则失败：{{ genError }}
    </p>

    <div v-if="rules.length === 0" class="rounded-lg border border-dashed border-neutral-700 p-4 text-center text-xs text-neutral-500">
      暂无规则，点击右上角新增。无规则时解析等价于纯 AI。
    </div>

    <div class="grid gap-3">
      <div
        v-for="(rule, i) in rules"
        :key="rule.id"
        class="rounded-lg border border-neutral-700 bg-neutral-950 p-3"
        @dragover.prevent
        @drop="onDrop(i)"
      >
        <div class="flex items-center gap-2">
          <span
            class="cursor-grab select-none text-neutral-600"
            draggable="true"
            title="拖拽调整优先级"
            @dragstart="onDragStart($event, i)"
            @dragend="dragIndex = null"
          >
            ⠿
          </span>
          <input v-model="rule.enabled" type="checkbox" class="accent-sky-500" title="启用/停用" />
          <input
            v-model="rule.name"
            class="min-w-0 flex-1 rounded border border-transparent bg-transparent px-1 py-0.5 text-sm hover:border-neutral-700 focus:border-sky-500 focus:outline-none"
          />
          <span class="rounded bg-neutral-800 px-1.5 py-0.5 text-[10px] text-neutral-400">
            {{ rule.type === "regex" ? "正则" : "分隔" }}
          </span>
          <button class="px-1 text-neutral-500 hover:text-neutral-200 disabled:opacity-30" :disabled="i === 0" title="上移" @click="move(i, -1)">↑</button>
          <button class="px-1 text-neutral-500 hover:text-neutral-200 disabled:opacity-30" :disabled="i === rules.length - 1" title="下移" @click="move(i, 1)">↓</button>
          <button class="px-1 text-neutral-600 hover:text-red-400" title="删除" @click="removeRule(rule.id)">✕</button>
        </div>

        <!-- 正则配置 -->
        <div v-if="rule.type === 'regex'" class="mt-2">
          <span class="text-xs text-neutral-500">正则（命名捕获组 (?&lt;title&gt;)/(?&lt;album&gt;)/(?&lt;artist&gt;)/(?&lt;track&gt;)；只提取已存在文本，赋常量请用下方「固定值」）</span>
          <textarea
            v-model="rule.pattern"
            rows="5"
            placeholder="例：^(?<album>[^丨]+)丨(?<title>.+)$  或纯条件 QA\d{3}"
            class="mt-1 w-full resize-y rounded border border-neutral-700 bg-neutral-900 px-2 py-1 text-xs outline-none focus:border-sky-500"
          ></textarea>
          <p v-if="regexError(rule)" class="mt-1 text-[11px] text-amber-400">⚠ 正则语法错误：{{ regexError(rule) }}</p>
        </div>

        <!-- 分隔配置 -->
        <div v-else class="mt-2">
          <label class="block">
            <span class="text-xs text-neutral-500">分隔符</span>
            <input
              v-model="rule.separator"
              class="mt-1 w-24 rounded border border-neutral-700 bg-neutral-900 px-2 py-1 text-xs outline-none focus:border-sky-500"
            />
          </label>
          <div class="mt-2 flex flex-wrap gap-2">
            <label v-for="f in RULE_FIELDS" :key="f" class="flex items-center gap-1 text-xs text-neutral-400">
              {{ FIELD_LABEL[f] }}
              <input
                type="number"
                min="0"
                placeholder="段"
                :value="rule.mapping[f] ?? ''"
                class="w-14 rounded border border-neutral-700 bg-neutral-900 px-1 py-0.5 text-xs outline-none focus:border-sky-500"
                @input="setSegment(rule, f, ($event.target as HTMLInputElement).value)"
              />
            </label>
          </div>
        </div>

        <!-- 固定值（常量赋值，两种规则通用）：命中后写入常量，覆盖捕获/分段 -->
        <div class="mt-2">
          <span class="text-xs text-neutral-500">固定值（命中后写入，覆盖捕获/分段；留空忽略）</span>
          <div class="mt-1 flex flex-wrap gap-2">
            <label v-for="f in RULE_FIELDS" :key="f" class="flex items-center gap-1 text-xs text-neutral-400">
              {{ FIELD_LABEL[f] }}
              <input
                :value="rule.constants?.[f] ?? ''"
                :placeholder="f === 'track' ? '数字' : '常量'"
                class="w-20 rounded border border-neutral-700 bg-neutral-900 px-1 py-0.5 text-xs outline-none focus:border-sky-500"
                @input="setConstant(rule, f, ($event.target as HTMLInputElement).value)"
              />
            </label>
          </div>
        </div>

        <!-- AI 生成 -->
        <div class="mt-2 flex items-end gap-2">
          <label class="block flex-1">
            <span class="text-[11px] text-neutral-500">用自然语言描述，让 AI 生成上面的配置（含固定值）</span>
            <input
              :value="descById[rule.id] ?? ''"
              placeholder="例：含 QA+3位数字的，节目固定为会员问答、作者波米"
              class="mt-1 w-full rounded border border-neutral-700 bg-neutral-900 px-2 py-1 text-xs outline-none focus:border-sky-500"
              @input="descById = { ...descById, [rule.id]: ($event.target as HTMLInputElement).value }"
            />
          </label>
          <button
            class="rounded-md bg-sky-600 px-2.5 py-1 text-xs text-white hover:bg-sky-500 disabled:opacity-50"
            :disabled="generating[rule.id] || !(descById[rule.id] ?? '').trim()"
            @click="generate(rule)"
          >
            {{ generating[rule.id] ? "生成中…" : "生成" }}
          </button>
        </div>

        <!-- 预览 -->
        <div v-if="sampleNames.length > 0" class="mt-2 border-t border-neutral-800 pt-2">
          <div v-for="name in sampleNames" :key="name" class="truncate text-[11px] text-neutral-500" :title="name">
            <span class="text-neutral-400">{{ name }}</span>
            →
            <span class="text-emerald-400">{{ previewLine(rule, name) }}</span>
          </div>
        </div>
      </div>
    </div>

    <label v-if="audio.files.length === 0" class="mt-3 block">
      <span class="text-[11px] text-neutral-500">预览测试文件名（未导入文件时用）</span>
      <input
        v-model="testName"
        placeholder="QA009：标题丨反派影评丨爱发电.mp3"
        class="mt-1 w-full rounded border border-neutral-700 bg-neutral-900 px-2 py-1 text-xs outline-none focus:border-sky-500"
      />
    </label>
  </div>
</template>
