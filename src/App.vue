<script setup lang="ts">
import { onMounted, ref } from "vue";
import { storeToRefs } from "pinia";
import { useAudioStore } from "@/store/audio";
import { useSettingsStore } from "@/store/settings";
import { useProfilesStore } from "@/store/profiles";
import { useWriteback } from "@/composables/useWriteback";
import SettingsModal from "@/components/SettingsModal.vue";
import ProfileLibraryModal from "@/components/ProfileLibraryModal.vue";

const store = useAudioStore();
const { total } = storeToRefs(store);

const settings = useSettingsStore();
// iCloud 未下载文件策略开关（PRD 5.6）+ 重命名开关（PRD 5.2）
const { icloudAutoDownload, renameEnabled } = storeToRefs(settings);

// 节目档案库：启动时加载，弹窗开关由 store 跨视图共享
const profiles = useProfilesStore();
const { libraryOpen } = storeToRefs(profiles);
// 启动加载档案库 + 已写回文件快照（标记可重置）
const { loadWritten } = useWriteback();
onMounted(() => {
  void profiles.load();
  void loadWritten();
});

// 设置弹窗开关
const settingsOpen = ref(false);
</script>

<template>
  <div class="flex h-screen flex-col bg-base text-strong">
    <header
      class="flex items-center justify-between border-b border-line px-5 py-3"
    >
      <div class="flex items-center gap-3">
        <span class="text-lg font-semibold tracking-tight">TagCast</span>
        <span class="rounded bg-elevated px-1.5 py-0.5 text-xs text-muted"
          >v1 · 播客</span
        >
        <!-- 表格批量 / 单文件向导 模式切换 -->
        <nav class="ml-2 flex items-center gap-1 rounded-lg bg-surface p-0.5 text-sm">
          <router-link
            to="/table"
            class="rounded-md px-2.5 py-1 text-muted"
            active-class="bg-edge text-strong"
          >
            表格批量
          </router-link>
          <router-link
            to="/wizard"
            class="rounded-md px-2.5 py-1 text-muted"
            active-class="bg-edge text-strong"
          >
            单文件向导
          </router-link>
        </nav>
      </div>
      <div class="flex items-center gap-4">
        <label
          class="flex cursor-pointer items-center gap-2 text-sm text-muted"
          title="开启后按模板重命名文件，关闭则只改元数据"
        >
          <input v-model="renameEnabled" type="checkbox" class="accent-accent" />
          重命名
        </label>
        <label
          class="flex cursor-pointer items-center gap-2 text-sm text-muted"
          title="iCloud 中未下载的文件：开启则自动下载，关闭则仅提示"
        >
          <input v-model="icloudAutoDownload" type="checkbox" class="accent-accent" />
          iCloud 自动下载
        </label>
        <span class="text-sm text-muted">已导入 {{ total }} 个文件</span>
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated"
          @click="profiles.openLibrary()"
        >
          节目档案库
        </button>
        <button
          class="rounded-lg border border-edge px-3 py-1.5 text-sm text-muted hover:bg-elevated"
          @click="settingsOpen = true"
        >
          设置
        </button>
      </div>
    </header>

    <SettingsModal :open="settingsOpen" @close="settingsOpen = false" />
    <ProfileLibraryModal :open="libraryOpen" @close="profiles.closeLibrary()" />

    <main class="min-h-0 flex-1 overflow-auto">
      <router-view />
    </main>
  </div>
</template>
