<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Plus, Edit, Delete, Refresh, Search, Check, Close, Box } from "@element-plus/icons-vue";
import { useMocStore, usePartsStore, useMasterDataStore, useAppStore } from "@/stores";
import type { MocList } from "@/types";
import MocDialog from "@/components/MocDialog.vue";

const mocStore = useMocStore();
const partsStore = usePartsStore();
const masterDataStore = useMasterDataStore();
const appStore = useAppStore();

const searchKeyword = ref("");
const dialogVisible = ref(false);
const editingMoc = ref<MocList | null>(null);
const detailVisible = ref(false);
const currentMocDetail = ref<MocList | null>(null);

const filteredMocs = computed(() => {
  if (!searchKeyword.value) return mocStore.mocLists;
  const keyword = searchKeyword.value.toLowerCase();
  return mocStore.mocLists.filter(
    (m) =>
      m.name.toLowerCase().includes(keyword) ||
      m.description?.toLowerCase().includes(keyword)
  );
});

async function loadData() {
  await Promise.all([
    mocStore.loadMocLists(),
    partsStore.loadParts(),
    masterDataStore.loadAll(),
  ]);
}

function handleAdd() {
  editingMoc.value = null;
  dialogVisible.value = true;
}

function handleEdit(moc: MocList) {
  editingMoc.value = { ...moc };
  dialogVisible.value = true;
}

async function handleDelete(moc: MocList) {
  const confirmed = await appStore.showConfirm(
    `确定要删除 MOC 清单「${moc.name}」吗？此操作不可撤销。`,
    "删除 MOC 清单"
  );
  if (confirmed) {
    await mocStore.deleteMocList(moc.id);
    appStore.showSuccess("删除成功");
  }
}

async function handleViewDetail(moc: MocList) {
  appStore.startLoading("正在比对库存...");
  try {
    currentMocDetail.value = await mocStore.compareInventory(moc.id);
    detailVisible.value = true;
  } finally {
    appStore.stopLoading();
  }
}

async function handleDialogSave(mocData: Omit<MocList, "id" | "createdAt" | "updatedAt">) {
  if (editingMoc.value) {
    await mocStore.updateMocList({
      ...editingMoc.value,
      ...mocData,
    });
    appStore.showSuccess("更新成功");
  } else {
    await mocStore.addMocList(mocData);
    appStore.showSuccess("创建成功");
  }
  dialogVisible.value = false;
}

function getMissingCount(moc: MocList) {
  return moc.parts.filter((p) => p.isMissing).length;
}

function getStatusText(moc: MocList) {
  const missing = getMissingCount(moc);
  if (missing === 0) return "零件齐全";
  return `缺少 ${missing} 种`;
}

function getStatusType(moc: MocList) {
  const missing = getMissingCount(moc);
  if (missing === 0) return "success";
  if (missing <= 3) return "warning";
  return "danger";
}

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

onMounted(() => {
  loadData();
});
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>
        <span class="brick-stud"></span>
        MOC 零件清单
      </h1>
      <div class="header-actions">
        <button class="brick-btn brick-btn-sm" @click="loadData">
          <el-icon><Refresh /></el-icon>
          刷新
        </button>
        <button class="brick-btn brick-btn-sm" @click="handleAdd">
          <el-icon><Plus /></el-icon>
          新建清单
        </button>
      </div>
    </div>

    <div class="page-content">
      <div class="filter-section brick-card">
        <div class="filter-row">
          <div class="filter-item search-item">
            <el-input
              v-model="searchKeyword"
              placeholder="搜索清单名称或描述..."
              class="brick-input"
              clearable
            >
              <template #prefix>
                <el-icon><Search /></el-icon>
              </template>
            </el-input>
          </div>
          <div class="filter-stats">
            <span class="stat">共 {{ filteredMocs.length }} 个清单</span>
            <span class="stat stat-warning">
              缺少零件: {{ mocStore.totalMissingParts }} 种
            </span>
          </div>
        </div>
      </div>

      <div v-if="filteredMocs.length === 0" class="empty-state">
        <el-icon class="empty-icon"><Box /></el-icon>
        <div class="empty-text">暂无 MOC 清单</div>
        <div class="empty-desc">
          点击「新建清单」创建你的第一个 MOC 零件清单
        </div>
      </div>

      <div v-else class="grid-container">
        <div
          v-for="moc in filteredMocs"
          :key="moc.id"
          class="moc-card brick-card"
        >
          <div class="moc-header">
            <h3>{{ moc.name }}</h3>
            <el-tag :type="getStatusType(moc)" effect="dark" size="small">
              {{ getStatusText(moc) }}
            </el-tag>
          </div>

          <div v-if="moc.description" class="moc-desc">
            {{ moc.description }}
          </div>

          <div class="moc-stats">
            <div class="stat-item">
              <span class="stat-label">零件种类</span>
              <span class="stat-value">{{ moc.parts.length }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">零件总数</span>
              <span class="stat-value">
                {{ moc.parts.reduce((sum, p) => sum + p.quantity, 0) }}
              </span>
            </div>
            <div class="stat-item">
              <span class="stat-label">缺少种类</span>
              <span
                class="stat-value"
                :class="{ 'text-danger': getMissingCount(moc) > 0 }"
              >
                {{ getMissingCount(moc) }}
              </span>
            </div>
          </div>

          <div class="moc-parts-preview">
            <div
              v-for="part in moc.parts.slice(0, 4)"
              :key="part.partId"
              class="part-mini"
            >
              <span
                class="color-dot"
                :style="{ backgroundColor: getColorHex(part.color) }"
              ></span>
              <span class="part-name">{{ part.partName }}</span>
              <span class="part-qty">x{{ part.quantity }}</span>
            </div>
            <div v-if="moc.parts.length > 4" class="more-parts">
              +{{ moc.parts.length - 4 }} 更多
            </div>
          </div>

          <div class="moc-footer">
            <span class="update-time">
              更新于 {{ new Date(moc.updatedAt).toLocaleDateString() }}
            </span>
            <div class="moc-actions">
              <button
                class="brick-btn brick-btn-sm brick-btn-secondary"
                @click="handleViewDetail(moc)"
              >
                <el-icon><Check /></el-icon>
                比对库存
              </button>
              <button
                class="brick-btn brick-btn-sm"
                @click="handleEdit(moc)"
              >
                <el-icon><Edit /></el-icon>
                编辑
              </button>
              <button
                class="brick-btn brick-btn-sm brick-btn-danger"
                @click="handleDelete(moc)"
              >
                <el-icon><Delete /></el-icon>
                删除
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <MocDialog
      v-model="dialogVisible"
      :moc="editingMoc"
      @save="handleDialogSave"
    />

    <el-dialog
      v-model="detailVisible"
      title="库存比对详情"
      width="800px"
      :close-on-click-modal="false"
    >
      <template #title>
        <span class="brick-stud"></span>
        {{ currentMocDetail?.name }} - 库存比对
      </template>

      <div v-if="currentMocDetail" class="moc-detail">
        <div class="detail-summary">
          <div class="summary-item">
            <span class="summary-label">零件种类</span>
            <span class="summary-value">
              {{ currentMocDetail.parts.length }}
            </span>
          </div>
          <div class="summary-item">
            <span class="summary-label">零件总数</span>
            <span class="summary-value">
              {{
                currentMocDetail.parts.reduce(
                  (sum, p) => sum + p.quantity,
                  0
                )
              }}
            </span>
          </div>
          <div class="summary-item">
            <span class="summary-label">齐全种类</span>
            <span class="summary-value text-success">
              {{
                currentMocDetail.parts.filter((p) => !p.isMissing).length
              }}
            </span>
          </div>
          <div class="summary-item">
            <span class="summary-label">缺少种类</span>
            <span class="summary-value text-danger">
              {{
                currentMocDetail.parts.filter((p) => p.isMissing).length
              }}
            </span>
          </div>
        </div>

        <div class="parts-table-container">
          <el-table
            :data="currentMocDetail.parts"
            stripe
            style="width: 100%"
          >
            <el-table-column label="零件" min-width="180">
              <template #default="{ row }">
                <div class="part-cell">
                  <span
                    class="color-dot"
                    :style="{ backgroundColor: getColorHex(row.color) }"
                  ></span>
                  <div>
                    <div class="part-name">{{ row.partName }}</div>
                    <div class="part-number">#{{ row.partNumber }}</div>
                  </div>
                </div>
              </template>
            </el-table-column>
            <el-table-column label="颜色" width="100">
              <template #default="{ row }">
                {{ row.color }}
              </template>
            </el-table-column>
            <el-table-column label="需要数量" width="100" align="center">
              <template #default="{ row }">
                {{ row.quantity }}
              </template>
            </el-table-column>
            <el-table-column label="库存数量" width="100" align="center">
              <template #default="{ row }">
                <span :class="{ 'text-danger': row.inStock < row.quantity }">
                  {{ row.inStock }}
                </span>
              </template>
            </el-table-column>
            <el-table-column label="差额" width="100" align="center">
              <template #default="{ row }">
                <span :class="row.inStock >= row.quantity ? 'text-success' : 'text-danger'">
                  {{ row.inStock - row.quantity >= 0 ? '+' : '' }}{{ row.inStock - row.quantity }}
                </span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100" align="center">
              <template #default="{ row }">
                <el-tag
                  :type="row.isMissing ? 'danger' : 'success'"
                  effect="dark"
                  size="small"
                >
                  <el-icon v-if="row.isMissing"><Close /></el-icon>
                  <el-icon v-else><Check /></el-icon>
                  {{ row.isMissing ? "缺少" : "充足" }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <div v-if="currentMocDetail.parts.some((p) => p.isMissing)" class="missing-list">
          <h4>缺少零件清单</h4>
          <div class="missing-parts">
            <div
              v-for="part in currentMocDetail.parts.filter((p) => p.isMissing)"
              :key="part.partId"
              class="missing-item"
            >
              <span
                class="color-dot"
                :style="{ backgroundColor: getColorHex(part.color) }"
              ></span>
              <span class="missing-name">{{ part.partName }}</span>
              <span class="missing-qty">
                需要 {{ part.quantity }}，现有 {{ part.inStock }}，
                <span class="text-danger">缺 {{ part.quantity - part.inStock }}</span>
              </span>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <button class="brick-btn" @click="detailVisible = false">
          关闭
        </button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.filter-section {
  padding: $spacing-lg;
  margin-bottom: $spacing-lg;
}

.filter-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: $spacing-lg;
}

.search-item {
  flex: 1;
  max-width: 400px;
}

.filter-stats {
  display: flex;
  gap: $spacing-lg;

  .stat {
    font-size: $font-size-sm;
    color: $color-gray-light;

    &.stat-warning {
      color: $color-danger;
    }
  }
}

.moc-card {
  padding: $spacing-lg;
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
}

.moc-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;

  h3 {
    font-size: $font-size-lg;
    font-weight: 600;
    color: $color-white;
    margin: 0;
  }
}

.moc-desc {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.moc-stats {
  display: flex;
  justify-content: space-around;
  padding: $spacing-md 0;
  border-top: 1px solid $color-dark-border;
  border-bottom: 1px solid $color-dark-border;
  margin: $spacing-sm 0;

  .stat-item {
    text-align: center;

    .stat-label {
      display: block;
      font-size: $font-size-sm;
      color: $color-gray-dark;
      margin-bottom: $spacing-xs;
    }

    .stat-value {
      font-size: $font-size-lg;
      font-weight: 700;
      color: $color-primary;

      &.text-danger {
        color: $color-danger;
      }
    }
  }
}

.moc-parts-preview {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: $spacing-xs;
  max-height: 120px;
  overflow: hidden;
}

.part-mini {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  font-size: $font-size-sm;

  .part-name {
    flex: 1;
    color: $color-gray-light;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .part-qty {
    color: $color-primary;
    font-weight: 600;
  }
}

.color-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid $color-dark-border;
  flex-shrink: 0;
}

.more-parts {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  text-align: center;
}

.moc-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: $spacing-sm;
  border-top: 1px solid $color-dark-border;

  .update-time {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }

  .moc-actions {
    display: flex;
    gap: $spacing-xs;
  }
}

.moc-detail {
  .detail-summary {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: $spacing-md;
    margin-bottom: $spacing-lg;
    padding: $spacing-lg;
    background: $color-dark;
    border-radius: $brick-radius;

    .summary-item {
      text-align: center;

      .summary-label {
        display: block;
        font-size: $font-size-sm;
        color: $color-gray-dark;
        margin-bottom: $spacing-xs;
      }

      .summary-value {
        font-size: 24px;
        font-weight: 700;
        color: $color-white;

        &.text-success {
          color: $color-success;
        }

        &.text-danger {
          color: $color-danger;
        }
      }
    }
  }

  .parts-table-container {
    margin-bottom: $spacing-lg;
  }

  .part-cell {
    display: flex;
    align-items: center;
    gap: $spacing-sm;

    .part-name {
      color: $color-white;
      font-weight: 500;
    }

    .part-number {
      font-size: $font-size-sm;
      color: $color-gray-dark;
      font-family: monospace;
    }
  }

  .text-success {
    color: $color-success;
  }

  .text-danger {
    color: $color-danger;
  }

  .missing-list {
    padding: $spacing-lg;
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid $color-danger;
    border-radius: $brick-radius;

    h4 {
      color: $color-danger;
      margin: 0 0 $spacing-md 0;
      font-size: $font-size-base;
    }

    .missing-parts {
      display: flex;
      flex-direction: column;
      gap: $spacing-sm;
    }

    .missing-item {
      display: flex;
      align-items: center;
      gap: $spacing-sm;
      font-size: $font-size-sm;

      .missing-name {
        min-width: 150px;
        color: $color-gray-light;
      }

      .missing-qty {
        color: $color-gray-dark;
      }
    }
  }
}
</style>
