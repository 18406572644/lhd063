<script setup lang="ts">
import { computed, ref } from "vue";
import {
  Plus,
  Edit,
  Delete,
  Star,
  StarFilled,
  Grid,
  Warning,
  Clock,
  Picture,
  Folder,
  CopyDocument,
  Check,
  Close,
  MoreFilled,
} from "@element-plus/icons-vue";
import { useViewsStore, useAppStore } from "@/stores";
import type { SavedView, PartFilter } from "@/types";

const props = defineProps<{
  currentFilter: PartFilter;
  currentSortField?: string;
  currentSortOrder?: "asc" | "desc";
}>();

const emit = defineEmits<{
  (e: "apply-view", view: SavedView): void;
  (e: "save-current"): void;
}>();

const viewsStore = useViewsStore();
const appStore = useAppStore();

const saveDialogVisible = ref(false);
const editDialogVisible = ref(false);
const editingView = ref<SavedView | null>(null);
const newViewName = ref("");
const newViewDescription = ref("");
const newViewIcon = ref("Folder");
const expandedMenuId = ref<string | null>(null);

const iconOptions = [
  { value: "Grid", label: "网格" },
  { value: "Warning", label: "警告" },
  { value: "Clock", label: "时钟" },
  { value: "Picture", label: "图片" },
  { value: "Folder", label: "文件夹" },
  { value: "Star", label: "星标" },
  { value: "Search", label: "搜索" },
  { value: "Filter", label: "筛选" },
  { value: "List", label: "列表" },
  { value: "Box", label: "盒子" },
];

const iconComponentMap: Record<string, any> = {
  Grid,
  Warning,
  Clock,
  Picture,
  Folder,
  Star,
  Search: Plus,
  Filter: Plus,
  List: Folder,
  Box: Folder,
};

function getIcon(name?: string) {
  return iconComponentMap[name || "Folder"] || Folder;
}

function applyView(view: SavedView) {
  viewsStore.setActiveView(view.id);
  emit("apply-view", view);
}

function openSaveDialog() {
  newViewName.value = "自定义视图 " + (viewsStore.customViews.length + 1);
  newViewDescription.value = "";
  newViewIcon.value = "Folder";
  saveDialogVisible.value = true;
}

function confirmSaveView() {
  if (!newViewName.value.trim()) return;
  const saved = viewsStore.saveView({
    name: newViewName.value.trim(),
    filter: JSON.parse(JSON.stringify(props.currentFilter || {})),
    sortField: props.currentSortField,
    sortOrder: props.currentSortOrder,
    icon: newViewIcon.value,
    description: newViewDescription.value.trim(),
  });
  viewsStore.setActiveView(saved.id);
  appStore.showSuccess("视图已保存");
  saveDialogVisible.value = false;
}

function openEditDialog(view: SavedView) {
  if (view.isPreset) return;
  editingView.value = view;
  newViewName.value = view.name;
  newViewDescription.value = view.description || "";
  newViewIcon.value = view.icon || "Folder";
  expandedMenuId.value = null;
  editDialogVisible.value = true;
}

function confirmEditView() {
  if (!editingView.value || !newViewName.value.trim()) return;
  viewsStore.updateView(editingView.value.id, {
    name: newViewName.value.trim(),
    description: newViewDescription.value.trim(),
    icon: newViewIcon.value,
  });
  appStore.showSuccess("视图已更新");
  editDialogVisible.value = false;
  editingView.value = null;
}

async function handleDelete(view: SavedView) {
  if (view.isPreset) return;
  expandedMenuId.value = null;
  const confirmed = await appStore.showConfirm(
    `确定要删除视图「${view.name}」吗？`,
    "删除视图"
  );
  if (!confirmed) return;
  viewsStore.deleteView(view.id);
  appStore.showSuccess("视图已删除");
}

function handleDuplicate(view: SavedView) {
  expandedMenuId.value = null;
  const copy = viewsStore.duplicateView(view.id);
  if (copy) {
    appStore.showSuccess("视图已复制");
    viewsStore.setActiveView(copy.id);
    emit("apply-view", copy);
  }
}

async function handleSetDefault(view: SavedView) {
  expandedMenuId.value = null;
  viewsStore.setAsDefault(view.id);
  if (view.isPreset) {
    appStore.showSuccess("已重置为默认预设视图");
  } else {
    appStore.showSuccess("已设为默认视图");
  }
}

function overwriteCurrentView() {
  if (viewsStore.activeView?.isPreset) {
    appStore.showWarning("预设视图无法覆盖，请先另存为新视图");
    return;
  }
  const ok = viewsStore.overwriteCurrentView(
    props.currentFilter,
    props.currentSortField,
    props.currentSortOrder
  );
  if (ok) {
    appStore.showSuccess("视图已更新为当前筛选条件");
  }
}

const activeId = computed(() => viewsStore.activeView?.id);
</script>

<template>
  <div class="views-sidebar">
    <div class="sidebar-header">
      <h3 class="sidebar-title">视图</h3>
      <button class="icon-btn-mini" @click="openSaveDialog" title="保存当前视图">
        <el-icon><Plus /></el-icon>
      </button>
    </div>

    <div class="sidebar-section">
      <div class="section-label">预设</div>
      <div class="view-list">
        <div
          v-for="view in viewsStore.presetViews"
          :key="view.id"
          class="view-item"
          :class="{ active: activeId === view.id }"
          @click="applyView(view)"
        >
          <el-icon class="view-icon">
            <component :is="getIcon(view.icon)" />
          </el-icon>
          <span class="view-name">{{ view.name }}</span>
          <el-icon v-if="view.isDefault" class="default-icon" title="默认视图">
            <StarFilled />
          </el-icon>
        </div>
      </div>
    </div>

    <div class="sidebar-section">
      <div class="section-label">自定义</div>
      <div v-if="viewsStore.customViews.length === 0" class="empty-views">
        暂无自定义视图
      </div>
      <div v-else class="view-list">
        <div
          v-for="view in viewsStore.customViews"
          :key="view.id"
          class="view-item"
          :class="{ active: activeId === view.id }"
          @click="applyView(view)"
        >
          <el-icon class="view-icon">
            <component :is="getIcon(view.icon)" />
          </el-icon>
          <span class="view-name">{{ view.name }}</span>
          <div class="view-actions" @click.stop>
            <el-dropdown
              trigger="click"
              :visible-arrow="false"
              @command="(cmd: any) => {
                if (cmd === 'edit') openEditDialog(view);
                else if (cmd === 'delete') handleDelete(view);
                else if (cmd === 'dup') handleDuplicate(view);
                else if (cmd === 'default') handleSetDefault(view);
              }"
            >
              <button class="icon-btn-mini">
                <el-icon><MoreFilled /></el-icon>
              </button>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="edit">
                    <el-icon><Edit /></el-icon>编辑
                  </el-dropdown-item>
                  <el-dropdown-item command="dup">
                    <el-icon><CopyDocument /></el-icon>复制
                  </el-dropdown-item>
                  <el-dropdown-item command="default">
                    <el-icon><Star /></el-icon>设为默认
                  </el-dropdown-item>
                  <el-dropdown-item command="delete" divided>
                    <el-icon><Delete /></el-icon>删除
                  </el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </div>
          <el-icon v-if="view.isDefault" class="default-icon" title="默认视图">
            <StarFilled />
          </el-icon>
        </div>
      </div>
    </div>

    <div class="sidebar-footer-actions">
      <button
        v-if="!viewsStore.activeView?.isPreset"
        class="brick-btn brick-btn-sm brick-btn-secondary w-full"
        @click="overwriteCurrentView"
      >
        <el-icon><Check /></el-icon>
        更新当前视图
      </button>
    </div>

    <el-dialog
      v-model="saveDialogVisible"
      title="保存视图"
      width="420px"
      class="brick-dialog"
    >
      <div class="form-group">
        <label>名称</label>
        <el-input v-model="newViewName" placeholder="输入视图名称" maxlength="32" />
      </div>
      <div class="form-group">
        <label>描述（可选）</label>
        <el-input
          v-model="newViewDescription"
          type="textarea"
          :rows="2"
          placeholder="描述此视图的筛选条件"
          maxlength="120"
        />
      </div>
      <div class="form-group">
        <label>图标</label>
        <div class="icon-picker">
          <button
            v-for="ic in iconOptions"
            :key="ic.value"
            class="icon-pick-btn"
            :class="{ active: newViewIcon === ic.value }"
            :title="ic.label"
            @click="newViewIcon = ic.value"
          >
            <el-icon><component :is="getIcon(ic.value)" /></el-icon>
          </button>
        </div>
      </div>
      <template #footer>
        <button class="brick-btn brick-btn-secondary" @click="saveDialogVisible = false">
          <el-icon><Close /></el-icon>取消
        </button>
        <button class="brick-btn" @click="confirmSaveView">
          <el-icon><Check /></el-icon>保存
        </button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="editDialogVisible"
      title="编辑视图"
      width="420px"
      class="brick-dialog"
    >
      <div class="form-group">
        <label>名称</label>
        <el-input v-model="newViewName" placeholder="输入视图名称" maxlength="32" />
      </div>
      <div class="form-group">
        <label>描述（可选）</label>
        <el-input
          v-model="newViewDescription"
          type="textarea"
          :rows="2"
          placeholder="描述此视图的筛选条件"
          maxlength="120"
        />
      </div>
      <div class="form-group">
        <label>图标</label>
        <div class="icon-picker">
          <button
            v-for="ic in iconOptions"
            :key="ic.value"
            class="icon-pick-btn"
            :class="{ active: newViewIcon === ic.value }"
            :title="ic.label"
            @click="newViewIcon = ic.value"
          >
            <el-icon><component :is="getIcon(ic.value)" /></el-icon>
          </button>
        </div>
      </div>
      <template #footer>
        <button class="brick-btn brick-btn-secondary" @click="editDialogVisible = false; editingView = null">
          <el-icon><Close /></el-icon>取消
        </button>
        <button class="brick-btn" @click="confirmEditView">
          <el-icon><Check /></el-icon>保存
        </button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.views-sidebar {
  width: 260px;
  flex-shrink: 0;
  background: $color-dark-light;
  border: $brick-border solid $color-dark-border;
  border-radius: $brick-radius;
  padding: $spacing-md;
  display: flex;
  flex-direction: column;
  gap: $spacing-md;
  height: fit-content;
  max-height: calc(100vh - 140px);
  overflow-y: auto;
}

.sidebar-header {
  display: flex;
  justify-content: space-between;
  align-items: center;

  .sidebar-title {
    margin: 0;
    font-size: $font-size-base;
    font-weight: 600;
    color: $color-white;
  }
}

.icon-btn-mini {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $color-dark-lighter;
  border: none;
  border-radius: $brick-radius;
  color: $color-gray-light;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    background: $color-primary;
    color: $color-dark;
  }
}

.sidebar-section {
  display: flex;
  flex-direction: column;
  gap: $spacing-xs;
}

.section-label {
  font-size: $font-size-xs;
  color: $color-gray-dark;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 0 $spacing-xs;
}

.view-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.view-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm $spacing-xs;
  border-radius: $brick-radius;
  cursor: pointer;
  color: $color-gray-light;
  transition: all $transition-fast;
  position: relative;

  &:hover {
    background: $color-dark-lighter;
    color: $color-primary;
  }

  &.active {
    background: rgba(255, 214, 0, 0.15);
    color: $color-primary;
    font-weight: 500;

    .view-icon {
      color: $color-primary;
    }
  }

  .view-icon {
    font-size: 16px;
    flex-shrink: 0;
  }

  .view-name {
    flex: 1;
    font-size: $font-size-sm;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .view-actions {
    display: none;
    flex-shrink: 0;
  }

  &:hover .view-actions {
    display: flex;
  }

  .default-icon {
    font-size: 14px;
    color: $color-primary;
    flex-shrink: 0;
  }
}

.empty-views {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  padding: $spacing-sm $spacing-xs;
  text-align: center;
  font-style: italic;
}

.sidebar-footer-actions {
  padding-top: $spacing-sm;
  border-top: 1px solid $color-dark-border;
}

.w-full {
  width: 100%;
}

.form-group {
  margin-bottom: $spacing-md;

  label {
    display: block;
    font-size: $font-size-sm;
    color: $color-gray-light;
    margin-bottom: $spacing-xs;
  }
}

.icon-picker {
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: $spacing-xs;
}

.icon-pick-btn {
  width: 100%;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $color-dark-lighter;
  border: 2px solid transparent;
  border-radius: $brick-radius;
  color: $color-gray-light;
  cursor: pointer;
  transition: all $transition-fast;
  font-size: 18px;

  &:hover {
    background: $color-dark-border;
    color: $color-primary;
  }

  &.active {
    border-color: $color-primary;
    background: rgba(255, 214, 0, 0.1);
    color: $color-primary;
  }
}

:deep(.brick-dialog) {
  .el-dialog {
    background: $color-dark-light !important;
    border: $brick-border solid $color-dark-border;
    border-radius: $brick-radius;

    .el-dialog__header {
      padding: $spacing-md $spacing-lg;
      border-bottom: 1px solid $color-dark-border;
      margin-right: 0;

      .el-dialog__title {
        color: $color-white;
        font-size: $font-size-lg;
        font-weight: 600;
      }
    }

    .el-dialog__body {
      padding: $spacing-lg;
      color: $color-gray-light;
    }

    .el-dialog__footer {
      padding: $spacing-md $spacing-lg;
      border-top: 1px solid $color-dark-border;
      display: flex;
      gap: $spacing-sm;
      justify-content: flex-end;
    }
  }
}
</style>
