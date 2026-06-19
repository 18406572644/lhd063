<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { Plus, Edit, Delete, Refresh, Camera, Picture, View, Box, Sort, ArrowUp, ArrowDown } from "@element-plus/icons-vue";
import { usePartsStore, useMasterDataStore, useAppStore, useViewsStore } from "@/stores";
import { useApiRequest } from "@/composables";
import type { Part, SavedView, PartFilter } from "@/types";
import PartDialog from "@/components/PartDialog.vue";
import PartImageDialog from "@/components/PartImageDialog.vue";
import FilterPanel from "@/components/FilterPanel.vue";
import ViewsSidebar from "@/components/ViewsSidebar.vue";

const partsStore = usePartsStore();
const masterDataStore = useMasterDataStore();
const appStore = useAppStore();
const viewsStore = useViewsStore();
const { execute } = useApiRequest();

const currentFilter = ref<PartFilter>(JSON.parse(JSON.stringify(partsStore.filter || {})));
const dialogVisible = ref(false);
const viewDialogVisible = ref(false);
const imageDialogVisible = ref(false);
const editingPart = ref<Part | null>(null);
const viewingPart = ref<Part | null>(null);
const imagePartId = ref("");
const imagePartName = ref("");

const activeViewName = computed(() => viewsStore.activeView?.name || "全部零件");

async function loadData() {
  await execute(() =>
    Promise.all([partsStore.loadParts(), masterDataStore.loadAll()]).then(
      () => ({ success: true as const, data: undefined as void })
    )
  );
}

function onFilterChange(newFilter: PartFilter) {
  currentFilter.value = JSON.parse(JSON.stringify(newFilter || {}));
  partsStore.replaceFilter(newFilter);
}

function onFilterSearch() {
  // reactive computed filters automatically; no-op to trigger anything else
}

function onFilterReset() {
  currentFilter.value = {};
  partsStore.clearFilter();
  if (viewsStore.defaultView) {
    viewsStore.setActiveView(viewsStore.defaultView.id);
  }
}

function handleApplyView(view: SavedView) {
  partsStore.applyView(view);
  currentFilter.value = JSON.parse(JSON.stringify(partsStore.filter || {}));
}

function handleSaveCurrentView() {
  // placeholder (button in header not wired, sidebar has +)
}

const sortOptions = [
  { field: "", label: "默认排序" },
  { field: "name", label: "名称" },
  { field: "partNumber", label: "零件编号" },
  { field: "quantity", label: "库存数量" },
  { field: "createdAt", label: "创建时间" },
  { field: "updatedAt", label: "更新时间" },
];

function toggleSort(field: string) {
  if (!field) {
    partsStore.setSort("", "desc");
    return;
  }
  if (partsStore.sortField === field) {
    partsStore.setSort(field, partsStore.sortOrder === "asc" ? "desc" : "asc");
  } else {
    partsStore.setSort(field, "desc");
  }
}

function handleAdd() {
  editingPart.value = null;
  dialogVisible.value = true;
}

function handleEdit(part: Part) {
  editingPart.value = { ...part };
  dialogVisible.value = true;
}

async function handleDelete(part: Part) {
  const confirmed = await appStore.showConfirm(
    `确定要删除零件「${part.name}」吗？此操作不可撤销。`,
    "删除零件"
  );
  if (confirmed) {
    await partsStore.deletePart(part.id);
    appStore.showSuccess("删除成功");
  }
}

function handleView(part: Part) {
  viewingPart.value = { ...part };
  viewDialogVisible.value = true;
}

function handleViewEdit() {
  if (viewingPart.value) {
    editingPart.value = { ...viewingPart.value };
    viewDialogVisible.value = false;
    dialogVisible.value = true;
  }
}

function handleImage(part: Part) {
  imagePartId.value = part.id;
  imagePartName.value = part.name;
  imageDialogVisible.value = true;
}

async function handleDialogSave(partData: Omit<Part, "id" | "createdAt" | "updatedAt">) {
  if (editingPart.value) {
    await partsStore.updatePart({
      ...editingPart.value,
      ...partData,
    });
    appStore.showSuccess("更新成功");
  } else {
    await partsStore.addPart(partData);
    appStore.showSuccess("添加成功");
  }
  dialogVisible.value = false;
}

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

function getTypeName(code: string) {
  return masterDataStore.getPartTypeName(code);
}

function getLocationName(code: string) {
  return masterDataStore.getLocationName(code);
}

onMounted(async () => {
  await loadData();
  if (viewsStore.activeView) {
    partsStore.applyView(viewsStore.activeView);
    currentFilter.value = JSON.parse(JSON.stringify(partsStore.filter || {}));
  }
});

watch(
  () => partsStore.filter,
  (f) => {
    currentFilter.value = JSON.parse(JSON.stringify(f || {}));
  },
  { deep: true }
);
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>
        <span class="brick-stud"></span>
        零件管理
        <el-tag v-if="activeViewName && activeViewName !== '全部零件'" type="warning" effect="dark" size="small" class="view-tag">
          {{ activeViewName }}
        </el-tag>
      </h1>
      <div class="header-actions">
        <el-dropdown trigger="click" @command="toggleSort" :visible-arrow="false">
          <button class="brick-btn brick-btn-sm brick-btn-secondary sort-btn">
            <el-icon><Sort /></el-icon>
            <span>{{ partsStore.sortField ? sortOptions.find(o => o.field === partsStore.sortField)?.label : '排序' }}</span>
            <el-icon v-if="partsStore.sortField">
              <component :is="partsStore.sortOrder === 'asc' ? ArrowUp : ArrowDown" />
            </el-icon>
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item
                v-for="opt in sortOptions"
                :key="opt.field"
                :command="opt.field"
              >
                {{ opt.label }}
                <span
                  v-if="opt.field && partsStore.sortField === opt.field"
                  class="sort-indicator"
                >
                  {{ partsStore.sortOrder === 'asc' ? '↑' : '↓' }}
                </span>
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
        <button class="brick-btn brick-btn-sm brick-btn-secondary" @click="loadData">
          <el-icon><Refresh /></el-icon>
          刷新
        </button>
        <button class="brick-btn brick-btn-sm" @click="handleAdd">
          <el-icon><Plus /></el-icon>
          新增零件
        </button>
      </div>
    </div>

    <div class="page-content-with-sidebar">
      <ViewsSidebar
        :current-filter="currentFilter"
        :current-sort-field="partsStore.sortField"
        :current-sort-order="partsStore.sortOrder"
        @apply-view="handleApplyView"
        @save-current="handleSaveCurrentView"
      />

      <div class="page-content-main">
        <FilterPanel
          :model-value="currentFilter"
          @update:modelValue="onFilterChange"
          @search="onFilterSearch"
          @reset="onFilterReset"
        />

        <div class="stats-bar">
          <div class="stat-item">
            <span class="stat-label">零件种类</span>
            <span class="stat-value">{{ partsStore.filteredParts.length }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">零件总数</span>
            <span class="stat-value">{{ partsStore.totalQuantity }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">低库存</span>
            <span class="stat-value stat-warning">
              {{ partsStore.lowStockParts.length }}
            </span>
          </div>
        </div>

        <div v-if="partsStore.filteredParts.length === 0" class="empty-state">
          <el-icon class="empty-icon"><Box /></el-icon>
          <div class="empty-text">暂无零件数据</div>
          <div class="empty-desc">点击「新增零件」开始添加你的乐高零件，或调整筛选条件</div>
        </div>

        <div v-else class="grid-container">
          <div
            v-for="part in partsStore.filteredParts"
            :key="part.id"
            class="part-card brick-card"
          >
            <div class="part-header">
              <div class="part-title">
                <span
                  class="color-indicator"
                  :style="{ backgroundColor: getColorHex(part.color) }"
                ></span>
                <h3>{{ part.name }}</h3>
              </div>
              <div class="part-actions">
                <button
                  class="icon-btn"
                  @click="handleView(part)"
                  title="查看详情"
                >
                  <el-icon><View /></el-icon>
                </button>
                <button
                  class="icon-btn"
                  @click="handleImage(part)"
                  title="管理图片"
                >
                  <el-icon><Camera /></el-icon>
                </button>
                <button
                  class="icon-btn"
                  @click="handleEdit(part)"
                  title="编辑"
                >
                  <el-icon><Edit /></el-icon>
                </button>
                <button
                  class="icon-btn icon-btn-danger"
                  @click="handleDelete(part)"
                  title="删除"
                >
                  <el-icon><Delete /></el-icon>
                </button>
              </div>
            </div>

            <div class="part-number">#{{ part.partNumber }}</div>

            <div class="part-info">
              <div class="info-row">
                <span class="info-label">类型</span>
                <span class="info-value">{{ getTypeName(part.type) }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">颜色</span>
                <span class="info-value">
                  <span
                    class="color-dot-small"
                    :style="{ backgroundColor: getColorHex(part.color) }"
                  ></span>
                  {{ part.color }}
                </span>
              </div>
              <div class="info-row">
                <span class="info-label">尺寸</span>
                <span class="info-value">{{ part.size }}</span>
              </div>
              <div class="info-row">
                <span class="info-label">位置</span>
                <span class="info-value">{{ getLocationName(part.location) }}</span>
              </div>
            </div>

            <div class="part-footer">
              <div class="quantity-section">
                <span class="quantity-label">库存</span>
                <span
                  class="quantity-value"
                  :class="{ 'low-stock': part.quantity <= 5 }"
                >
                  {{ part.quantity }}
                </span>
              </div>
              <el-tag
                v-if="part.quantity <= 5"
                type="danger"
                effect="dark"
                size="small"
              >
                库存不足
              </el-tag>
              <el-tag v-else type="success" effect="dark" size="small">
                充足
              </el-tag>
            </div>

            <div v-if="part.description" class="part-description">
              {{ part.description }}
            </div>

            <div v-if="part.imagePath" class="part-image-preview">
              <el-icon><Picture /></el-icon>
              已上传图片
            </div>
          </div>
        </div>
      </div>
    </div>

    <PartDialog
      v-model="dialogVisible"
      :part="editingPart"
      @save="handleDialogSave"
    />

    <PartDialog
      v-model="viewDialogVisible"
      :part="viewingPart"
      mode="view"
      @edit="handleViewEdit"
    />

    <PartImageDialog
      v-model="imageDialogVisible"
      :part-id="imagePartId"
      :part-name="imagePartName"
    />
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.stats-bar {
  display: flex;
  gap: $spacing-lg;
  margin-bottom: $spacing-lg;
  padding: $spacing-md $spacing-lg;
  background: $color-dark-light;
  border: $brick-border solid $color-dark-border;
  border-radius: $brick-radius;
}

.stat-item {
  display: flex;
  flex-direction: column;

  .stat-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }

  .stat-value {
    font-size: $font-size-lg;
    font-weight: 700;
    color: $color-primary;

    &.stat-warning {
      color: $color-danger;
    }
  }
}

.page-content-with-sidebar {
  display: flex;
  gap: $spacing-lg;
  align-items: flex-start;
}

.page-content-main {
  flex: 1;
  min-width: 0;
}

.view-tag {
  margin-left: $spacing-sm;
}

.sort-btn {
  gap: $spacing-xs;
}

.sort-indicator {
  margin-left: auto;
  color: $color-primary;
  font-weight: 700;
}

.part-card {
  padding: $spacing-lg;
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
}

.part-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: $spacing-xs;
}

.part-title {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  flex: 1;
  min-width: 0;

  h3 {
    font-size: $font-size-base;
    font-weight: 600;
    color: $color-white;
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.color-indicator {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  border: 2px solid $color-dark-border;
  flex-shrink: 0;
}

.color-dot-small {
  display: inline-block;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  margin-right: 4px;
  vertical-align: middle;
}

.part-actions {
  display: flex;
  gap: $spacing-xs;
  margin-left: $spacing-sm;
}

.icon-btn {
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

  &.icon-btn-danger:hover {
    background: $color-danger;
    color: $color-white;
  }
}

.part-number {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  font-family: monospace;
}

.part-info {
  display: flex;
  flex-direction: column;
  gap: $spacing-xs;
  margin: $spacing-sm 0;
}

.info-row {
  display: flex;
  justify-content: space-between;
  font-size: $font-size-sm;

  .info-label {
    color: $color-gray-dark;
  }

  .info-value {
    color: $color-gray-light;
    font-weight: 500;
  }
}

.part-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: $spacing-sm;
  border-top: 1px solid $color-dark-border;
  margin-top: auto;
}

.quantity-section {
  display: flex;
  align-items: baseline;
  gap: $spacing-sm;

  .quantity-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }

  .quantity-value {
    font-size: $font-size-lg;
    font-weight: 700;
    color: $color-success;

    &.low-stock {
      color: $color-danger;
    }
  }
}

.part-description {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  padding-top: $spacing-sm;
  border-top: 1px solid $color-dark-border;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.part-image-preview {
  display: flex;
  align-items: center;
  gap: $spacing-xs;
  font-size: $font-size-sm;
  color: $color-primary;
  padding-top: $spacing-sm;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: $spacing-md;
}

.empty-state {
  padding: $spacing-xl * 2;
  text-align: center;
  background: $color-dark-light;
  border: $brick-border dashed $color-dark-border;
  border-radius: $brick-radius;

  .empty-icon {
    font-size: 64px;
    color: $color-gray-dark;
    margin-bottom: $spacing-md;
  }

  .empty-text {
    font-size: $font-size-lg;
    font-weight: 600;
    color: $color-white;
    margin-bottom: $spacing-xs;
  }

  .empty-desc {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

@media (max-width: 1200px) {
  .page-content-with-sidebar {
    flex-direction: column;
  }
}
</style>
