<script setup lang="ts">
import { ref, watch } from "vue";
import { storeToRefs } from "pinia";
import { useProfilesStore } from "@/store/profiles";
import type { ShowProfile } from "@/types/profile";

defineProps<{ open: boolean }>();
const emit = defineEmits<{ close: [] }>();

const store = useProfilesStore();
// 档案列表 + 引导新建时预填的节目名（来源：表格未匹配的 album）
const { profiles, draftAlbum } = storeToRefs(store);

// 编辑表单：editingId 为 null 表示新增
const editingId = ref<number | null>(null);
const album = ref("");
const artist = ref("");
const keywords = ref("");

function resetForm(): void {
  editingId.value = null;
  album.value = "";
  artist.value = "";
  keywords.value = "";
}

function startEdit(p: ShowProfile): void {
  editingId.value = p.id;
  album.value = p.album;
  artist.value = p.artist ?? "";
  keywords.value = p.keywords.join(", ");
}

async function submit(): Promise<void> {
  if (!album.value.trim()) return;
  await store.save({
    id: editingId.value,
    album: album.value.trim(),
    artist: artist.value.trim() || null,
    keywords: keywords.value.split(/[,，]/).map((k) => k.trim()).filter(Boolean),
  });
  resetForm();
}

async function onDelete(id: number): Promise<void> {
  await store.remove(id);
  if (editingId.value === id) resetForm();
}

// 弹窗打开时加载档案；若带着待建节目名（引导新建）则预填到表单
watch(
  () => draftAlbum.value,
  (v) => {
    if (v) {
      resetForm();
      album.value = v;
    }
  },
);
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
    @click.self="emit('close')"
  >
    <div class="flex max-h-[80vh] w-[560px] flex-col rounded-xl border border-neutral-800 bg-neutral-900 p-5">
      <h2 class="mb-4 text-base font-semibold">节目档案库</h2>

      <!-- 已有档案列表 -->
      <div class="mb-4 min-h-0 flex-1 overflow-auto rounded-lg border border-neutral-800">
        <table v-if="profiles.length > 0" class="w-full text-sm">
          <thead class="sticky top-0 bg-neutral-950 text-left text-xs text-neutral-500">
            <tr>
              <th class="px-3 py-2">节目</th>
              <th class="px-3 py-2">作者</th>
              <th class="px-3 py-2">关键词</th>
              <th class="w-20 px-3 py-2"></th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="p in profiles" :key="p.id" class="border-t border-neutral-800">
              <td class="px-3 py-1.5">{{ p.album }}</td>
              <td class="px-3 py-1.5 text-neutral-400">{{ p.artist ?? "—" }}</td>
              <td class="px-3 py-1.5 text-xs text-neutral-500">{{ p.keywords.join("、") || "—" }}</td>
              <td class="px-3 py-1.5 text-right">
                <button class="mr-2 text-xs text-sky-400 hover:underline" @click="startEdit(p)">编辑</button>
                <button class="text-xs text-red-400 hover:underline" @click="onDelete(p.id)">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
        <p v-else class="p-6 text-center text-sm text-neutral-500">还没有节目档案，下方新建一个。</p>
      </div>

      <!-- 新增 / 编辑表单 -->
      <div class="grid gap-2 rounded-lg border border-neutral-800 bg-neutral-950/60 p-3">
        <p class="text-xs text-neutral-500">{{ editingId === null ? "新建档案" : "编辑档案" }}</p>
        <div class="grid grid-cols-2 gap-2">
          <input
            v-model="album"
            placeholder="节目名（必填）"
            class="rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
          <input
            v-model="artist"
            placeholder="作者"
            class="rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
          />
        </div>
        <input
          v-model="keywords"
          placeholder="匹配关键词（逗号分隔，命中文件名即归入该节目）"
          class="rounded-lg border border-neutral-700 bg-neutral-950 px-3 py-2 text-sm outline-none focus:border-sky-500"
        />
        <div class="flex items-center gap-2">
          <button
            class="rounded-lg bg-sky-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-sky-500 disabled:opacity-50"
            :disabled="!album.trim()"
            @click="submit"
          >
            {{ editingId === null ? "新增" : "保存" }}
          </button>
          <button
            v-if="editingId !== null"
            class="rounded-lg border border-neutral-700 px-3 py-1.5 text-sm text-neutral-300 hover:bg-neutral-800"
            @click="resetForm"
          >
            取消编辑
          </button>
        </div>
      </div>

      <div class="mt-4 flex justify-end">
        <button
          class="rounded-lg border border-neutral-700 px-4 py-2 text-sm text-neutral-300 hover:bg-neutral-800"
          @click="emit('close')"
        >
          关闭
        </button>
      </div>
    </div>
  </div>
</template>
