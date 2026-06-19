<script setup lang="ts">
import { ref, computed, onMounted, watch } from "vue";
import { Plus, Edit, Delete, Search, Refresh, Camera, Picture } from "@element-plus/icons-vue";
import { usePartsStore, useMasterDataStore, useAppStore } from "@/stores";
import type { Part } from "@/types";
import PartDialog from "@/components/PartDialog.vue";
import PartImageDialog from "@/components/PartImageDialog.vue";

const partsStore = usePartsStore();
const masterDataStore = useMasterDataStore();
const appStore = useAppStore();

const searchKeyword = ref("");
const filterType = ref("");
const filterColor = ref("");
const filterSize = ref("");
const filterLocation = ref("");
const dialogVisible = ref(false);
const imageDialogVisible = ref(false);
const editingPart = ref<Part | null>(null);
const imagePartId = ref("");
const imagePartName = ref("");

const typeOptions = computed(() =>
  masterDataStore.partTypes.map((t) => ({ label: t.name, value: t.code }))
);

const colorOptions = computed(() =>
  masterDataStore.partColors.map((c) => ({ label: c.name, value: c.name }))
);

const sizeOptions = computed(() =>
  masterDataStore.partSizes.map((s) => ({ label: s.name, value: s.name }))
);

const locationOptions = computed(() =>
  masterDataStore.locations.map((l) => ({ label: l.name, value: l.code }))
);

async function loadData() {
  await Promise.all([partsStore.loadParts(), masterDataStore.loadAll()]);
}

function handleSearch() {
  partsStore.setFilter({
    keyword: searchKeyword.value || undefined,
    type: filterType.value || undefined,
    color: filterColor.value || undefined,
    size: filterSize.value || undefined,
    location: filterLocation.value || undefined,
  });
}

function handleReset() {
  searchKeyword.value = "";
  filterType.value = "";
  filterColor.value = "";
  filterSize.value = "";
  filterLocation.value = "";
  partsStore.clearFilter();
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

watch(
  () => [filterType.value, filterColor.value, filterSize.value, filterLocation.value],
  () => {
    handleSearch();
  }
);

onMounted(() => {
  loadData();
});
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>
        <span class="brick-stud"></span>
        零件管理
      </h1>
      <div class="header-actions">
        <button class="brick-btn brick-btn-sm" @click="loadData">
          <el-icon><Refresh /></el-icon>
          刷新
        </button>
        <button class="brick-btn brick-btn-sm" @click="handleAdd">
          <el-icon><Plus /></el-icon>
          新增零件
        </button>
      </div>
    </div>

    <div class="page-content">
      <div class="filter-section brick-card">
        <div class="filter-row">
          <div class="filter-item">
            <el-input
              v-model="searchKeyword"
              placeholder="搜索零件名称/编号..."
              class="brick-input"
              clearable
              @keyup.enter="handleSearch"
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </div>
          <div class="filter-item">
            <el-select
              v-model="filterType"
              placeholder="零件类型"
              clearable
              class="w-full"
            >
              <el-option
                v-for="opt in typeOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>
          <div class="filter-item">
            <el-select
              v-model="filterColor"
              placeholder="颜色"
              clearable
              class="w-full"
            >
              <el-option
                v-for="opt in colorOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              >
                <span class="color-option">
                  <span
                    class="color-dot"
                    :style="{ backgroundColor: getColorHex(opt.value) }"
                  ></span>
                  {{ opt.label }}
                </span>
              </el-option>
            </el-select>
          </div>
          <div class="filter-item">
            <el-select
              v-model="filterSize"
              placeholder="尺寸"
              clearable
              class="w-full"
            >
              <el-option
                v-for="opt in sizeOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>
          <div class="filter-item">
            <el-select
              v-model="filterLocation"
              placeholder="存放位置"
              clearable
              class="w-full"
            >
              <el-option
                v-for="opt in locationOptions"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>
          <div class="filter-item filter-actions">
            <button class="brick-btn brick-btn-sm" @click="handleSearch">
              搜索
            </button>
            <button
              class="brick-btn brick-btn-sm brick-btn-secondary"
              @click="handleReset"
            >
              重置
            </button>
          </div>
        </div>
      </div>

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
        <div class="empty-desc">点击「新增零件」开始添加你的乐高零件</div>
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

    <PartDialog
      v-model="dialogVisible"
      :part="editingPart"
      @save="handleDialogSave"
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

.filter-section {
  padding: $spacing-lg;
  margin-bottom: $spacing-lg;
}

.filter-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr auto;
  gap: $spacing-md;
  align-items: end;
}

.filter-item {
  min-width: 0;
}

.filter-actions {
  display: flex;
  gap: $spacing-sm;
}

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

.color-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  margin-right: $spacing-xs;
  vertical-align: middle;
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

@media (max-width: 1200px) {
  .filter-row {
    grid-template-columns: 1fr 1fr;
  }

  .filter-actions {
    grid-column: span 2;
  }
}
</style>
