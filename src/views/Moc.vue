<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import {
  Plus,
  Edit,
  Delete,
  Refresh,
  Search,
  Check,
  Close,
  Box,
  Grid,
  Calendar,
  Clock,
  Upload,
  Picture,
  Promotion,
} from "@element-plus/icons-vue";
import {
  useMocStore,
  usePartsStore,
  useMasterDataStore,
  useAppStore,
} from "@/stores";
import type { MocList, MocStatus } from "@/types";
import { MOC_STATUS_OPTIONS } from "@/types";
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
const statusFilter = ref<MocStatus | "all">("all");
const viewMode = ref<"grid" | "kanban">("grid");
const statusDialogVisible = ref(false);
const statusChangeMoc = ref<MocList | null>(null);
const newStatus = ref<MocStatus>("planning");
const statusRemark = ref("");
const statusLogsVisible = ref(false);
const statusLogsMoc = ref<MocList | null>(null);
const coverDialogVisible = ref(false);
const coverImageMocId = ref<string | null>(null);
const coverImageInput = ref<HTMLInputElement | null>(null);

const coverImageMoc = computed(() => {
  if (!coverImageMocId.value) return null;
  return mocStore.mocLists.find((m) => m.id === coverImageMocId.value) ?? null;
});

const filteredMocs = computed(() => {
  let result = mocStore.mocLists;
  if (statusFilter.value !== "all") {
    result = result.filter((m) => m.status === statusFilter.value);
  }
  if (searchKeyword.value) {
    const keyword = searchKeyword.value.toLowerCase();
    result = result.filter(
      (m) =>
        m.name.toLowerCase().includes(keyword) ||
        m.description?.toLowerCase().includes(keyword)
    );
  }
  return result;
});

const kanbanColumns = computed(() => {
  const result = new Map<MocStatus, MocList[]>();
  for (const opt of MOC_STATUS_OPTIONS) {
    result.set(opt.value, []);
  }
  for (const moc of filteredMocs.value) {
    if (!result.has(moc.status)) {
      result.set(moc.status, []);
    }
    result.get(moc.status)!.push(moc);
  }
  return result;
});

function getStatusLabel(status: string) {
  const opt = MOC_STATUS_OPTIONS.find((o) => o.value === status);
  return opt?.label ?? status;
}

function getStatusColor(status: string) {
  const opt = MOC_STATUS_OPTIONS.find((o) => o.value === status);
  return opt?.color ?? "#909399";
}

function getStatusType(status: string) {
  const opt = MOC_STATUS_OPTIONS.find((o) => o.value === status);
  return opt?.type ?? "info";
}

function getImageUrl(path?: string) {
  if (!path) return "";
  if (path.startsWith("file://")) return path;
  return `file://${path}`;
}

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

async function handleDialogSave(
  mocData: Omit<MocList, "id" | "createdAt" | "updatedAt"> & {
    coverRemoved?: boolean;
  }
) {
  let savedMoc: MocList;
  try {
    if (editingMoc.value) {
      savedMoc = await mocStore.updateMocList({
        ...editingMoc.value,
        ...mocData,
      });
      appStore.showSuccess("更新成功");
    } else {
      savedMoc = await mocStore.addMocList(mocData);
      appStore.showSuccess("创建成功");
    }

    if (mocData.coverImagePath) {
      appStore.startLoading("正在上传封面...");
      try {
        await mocStore.saveMocCoverImage(savedMoc.id, mocData.coverImagePath);
      } finally {
        appStore.stopLoading();
      }
    } else if (mocData.coverRemoved && savedMoc) {
      appStore.startLoading("正在删除封面...");
      try {
        await mocStore.deleteMocCoverImage(savedMoc.id);
      } finally {
        appStore.stopLoading();
      }
    }
  } finally {
    dialogVisible.value = false;
  }
}

function openStatusChange(moc: MocList) {
  statusChangeMoc.value = moc;
  newStatus.value = moc.status;
  statusRemark.value = "";
  statusDialogVisible.value = true;
}

async function confirmStatusChange() {
  if (!statusChangeMoc.value) return;
  appStore.startLoading("正在更新状态...");
  try {
    await mocStore.changeMocStatus(
      statusChangeMoc.value.id,
      newStatus.value,
      statusRemark.value || undefined
    );
    appStore.showSuccess("状态更新成功");
    statusDialogVisible.value = false;
  } finally {
    appStore.stopLoading();
  }
}

async function openStatusLogs(moc: MocList) {
  statusLogsMoc.value = moc;
  mocStore.clearStatusLogs();
  appStore.startLoading("正在加载状态日志...");
  try {
    await mocStore.loadStatusLogs(moc.id);
    statusLogsVisible.value = true;
  } finally {
    appStore.stopLoading();
  }
}

function openCoverDialog(moc: MocList) {
  coverImageMocId.value = moc.id;
  coverDialogVisible.value = true;
}

function triggerCoverUpload() {
  coverImageInput.value?.click();
}

async function handleCoverImageChange(e: Event) {
  const target = e.target as HTMLInputElement;
  const file = target.files?.[0];
  if (!file || !coverImageMoc.value) return;

  const validTypes = ["image/jpeg", "image/png", "image/jpg"];
  if (!validTypes.includes(file.type)) {
    appStore.showError("仅支持 JPG、PNG 格式的图片");
    return;
  }

  appStore.startLoading("正在上传封面...");
  try {
    const reader = new FileReader();
    reader.onload = async (ev) => {
      const base64 = (ev.target?.result as string)?.split(",")[1];
      if (base64 && coverImageMoc.value) {
        await mocStore.saveMocCoverImage(coverImageMoc.value.id, base64);
        appStore.showSuccess("封面上传成功");
        coverDialogVisible.value = false;
      }
      appStore.stopLoading();
    };
    reader.onerror = () => {
      appStore.showError("图片读取失败");
      appStore.stopLoading();
    };
    reader.readAsDataURL(file);
  } catch (err) {
    appStore.stopLoading();
    appStore.showError("上传失败");
  }
  target.value = "";
}

async function handleDeleteCover() {
  if (!coverImageMoc.value) return;
  const confirmed = await appStore.showConfirm(
    "确定要删除此封面图片吗？",
    "删除封面"
  );
  if (confirmed) {
    appStore.startLoading("正在删除...");
    try {
      await mocStore.deleteMocCoverImage(coverImageMoc.value.id);
      appStore.showSuccess("封面已删除");
      coverDialogVisible.value = false;
    } finally {
      appStore.stopLoading();
    }
  }
}

function getMissingCount(moc: MocList) {
  return (moc.parts ?? []).filter((p) => p.isMissing).length;
}

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

function getStatusCount(status: MocStatus) {
  return mocStore.getStatusCount(status);
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
        <button
          class="brick-btn brick-btn-sm brick-btn-secondary"
          @click="viewMode = viewMode === 'grid' ? 'kanban' : 'grid'"
        >
          <el-icon>
            <component :is="viewMode === 'grid' ? Calendar : Grid" />
          </el-icon>
          {{ viewMode === "grid" ? "看板视图" : "网格视图" }}
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

          <div class="filter-item status-filter">
            <el-select
              v-model="statusFilter"
              placeholder="按状态筛选"
              class="brick-select"
              clearable
            >
              <el-option label="全部状态" value="all" />
              <el-option
                v-for="opt in MOC_STATUS_OPTIONS"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              />
            </el-select>
          </div>

          <div class="filter-stats">
            <span class="stat">共 {{ filteredMocs.length }} 个清单</span>
            <span class="stat stat-warning">
              缺少零件: {{ mocStore.totalMissingParts }} 种
            </span>
          </div>
        </div>

        <div v-if="viewMode === 'grid'" class="status-quick-row">
          <div
            v-for="opt in MOC_STATUS_OPTIONS"
            :key="opt.value"
            class="status-chip"
            :class="{ active: statusFilter === opt.value }"
            :style="{ borderColor: opt.color, color: statusFilter === opt.value ? '#fff' : opt.color, backgroundColor: statusFilter === opt.value ? opt.color : 'transparent' }"
            @click="statusFilter = statusFilter === opt.value ? 'all' : opt.value"
          >
            {{ opt.label }} ({{ getStatusCount(opt.value) }})
          </div>
        </div>
      </div>

      <!-- Grid View -->
      <template v-if="viewMode === 'grid'">
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
            <div
              v-if="moc.coverImagePath"
              class="moc-cover"
              @click="openCoverDialog(moc)"
            >
              <img :src="getImageUrl(moc.coverImagePath)" :alt="moc.name" />
              <div class="moc-cover-overlay">
                <el-icon><Picture /></el-icon>
                <span>点击更换封面</span>
              </div>
            </div>
            <div
              v-else
              class="moc-cover moc-cover-empty"
              @click="openCoverDialog(moc)"
            >
              <el-icon><Upload /></el-icon>
              <span>上传封面</span>
            </div>

            <div class="moc-header">
              <h3>{{ moc.name }}</h3>
              <el-tag
                :type="getStatusType(moc.status)"
                effect="dark"
                size="small"
                class="status-tag"
                @click="openStatusChange(moc)"
              >
                {{ getStatusLabel(moc.status) }}
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
                  class="brick-btn brick-btn-xs brick-btn-secondary"
                  title="状态变更记录"
                  @click="openStatusLogs(moc)"
                >
                  <el-icon><Clock /></el-icon>
                </button>
                <button
                  class="brick-btn brick-btn-xs brick-btn-secondary"
                  title="变更状态"
                  @click="openStatusChange(moc)"
                >
                  <el-icon><Promotion /></el-icon>
                </button>
                <button
                  class="brick-btn brick-btn-xs brick-btn-secondary"
                  @click="handleViewDetail(moc)"
                >
                  <el-icon><Check /></el-icon>
                  比对
                </button>
                <button
                  class="brick-btn brick-btn-xs"
                  @click="handleEdit(moc)"
                >
                  <el-icon><Edit /></el-icon>
                </button>
                <button
                  class="brick-btn brick-btn-xs brick-btn-danger"
                  @click="handleDelete(moc)"
                >
                  <el-icon><Delete /></el-icon>
                </button>
              </div>
            </div>
          </div>
        </div>
      </template>

      <!-- Kanban View -->
      <template v-else>
        <div class="kanban-container">
          <div
            v-for="opt in MOC_STATUS_OPTIONS"
            :key="opt.value"
            class="kanban-column"
          >
            <div
              class="kanban-header"
              :style="{ borderLeftColor: opt.color }"
            >
              <span class="kanban-title">{{ opt.label }}</span>
              <span
                class="kanban-count"
                :style="{ backgroundColor: opt.color }"
              >
                {{ kanbanColumns.get(opt.value)?.length ?? 0 }}
              </span>
            </div>

            <div class="kanban-cards">
              <div
                v-for="moc in (kanbanColumns.get(opt.value) ?? [])"
                :key="moc.id"
                class="kanban-card brick-card"
                @click="openStatusChange(moc)"
              >
                <div
                  v-if="moc.coverImagePath"
                  class="kanban-cover"
                >
                  <img :src="getImageUrl(moc.coverImagePath)" :alt="moc.name" />
                </div>
                <div class="kanban-card-title">{{ moc.name }}</div>
                <div v-if="moc.description" class="kanban-card-desc">
                  {{ moc.description }}
                </div>
                <div class="kanban-card-meta">
                  <span>{{ moc.parts.length }} 种零件</span>
                  <span
                    v-if="getMissingCount(moc) > 0"
                    class="text-danger"
                  >
                    缺 {{ getMissingCount(moc) }}
                  </span>
                </div>
              </div>
              <div
                v-if="!kanbanColumns.get(opt.value)?.length"
                class="kanban-empty"
              >
                暂无清单
              </div>
            </div>
          </div>
        </div>
      </template>
    </div>

    <MocDialog
      v-model="dialogVisible"
      :moc="editingMoc"
      @save="handleDialogSave"
    />

    <!-- Detail Dialog -->
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
        <div v-if="currentMocDetail.coverImagePath" class="detail-cover">
          <img
            :src="getImageUrl(currentMocDetail.coverImagePath)"
            :alt="currentMocDetail.name"
          />
        </div>

        <div class="detail-header-info">
          <el-tag
            :type="getStatusType(currentMocDetail.status)"
            effect="dark"
            size="large"
          >
            {{ getStatusLabel(currentMocDetail.status) }}
          </el-tag>
        </div>

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
                <span
                  :class="
                    row.inStock >= row.quantity ? 'text-success' : 'text-danger'
                  "
                >
                  {{ row.inStock - row.quantity >= 0 ? "+" : "" }}{{ row.inStock - row.quantity }}
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

        <div
          v-if="currentMocDetail.parts.some((p) => p.isMissing)"
          class="missing-list"
        >
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
        <div class="detail-footer-actions">
          <button
            class="brick-btn brick-btn-secondary"
            @click="
              currentMocDetail && openStatusChange(currentMocDetail)
            "
          >
            <el-icon><Promotion /></el-icon>
            变更状态
          </button>
          <button
            class="brick-btn brick-btn-secondary"
            @click="
              currentMocDetail && openStatusLogs(currentMocDetail)
            "
          >
            <el-icon><Clock /></el-icon>
            状态记录
          </button>
          <button class="brick-btn" @click="detailVisible = false">
            关闭
          </button>
        </div>
      </template>
    </el-dialog>

    <!-- Status Change Dialog -->
    <el-dialog
      v-model="statusDialogVisible"
      title="变更 MOC 状态"
      width="500px"
      :close-on-click-modal="false"
    >
      <template #title>
        <span class="brick-stud"></span>
        变更状态 - {{ statusChangeMoc?.name }}
      </template>

      <div class="status-change-form">
        <div class="current-status">
          <span class="label">当前状态：</span>
          <el-tag
            :type="statusChangeMoc ? getStatusType(statusChangeMoc.status) : 'info'"
            effect="dark"
          >
            {{ statusChangeMoc ? getStatusLabel(statusChangeMoc.status) : "" }}
          </el-tag>
        </div>

        <el-form label-width="100px">
          <el-form-item label="新状态">
            <el-select
              v-model="newStatus"
              class="status-select"
              placeholder="请选择新状态"
            >
              <el-option
                v-for="opt in MOC_STATUS_OPTIONS"
                :key="opt.value"
                :label="opt.label"
                :value="opt.value"
              >
                <div class="status-option">
                  <span
                    class="status-dot"
                    :style="{ backgroundColor: opt.color }"
                  ></span>
                  <span>{{ opt.label }}</span>
                </div>
              </el-option>
            </el-select>
          </el-form-item>

          <el-form-item label="备注说明">
            <el-input
              v-model="statusRemark"
              type="textarea"
              :rows="3"
              placeholder="可选：说明此次状态变更的原因"
            />
          </el-form-item>
        </el-form>
      </div>

      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="statusDialogVisible = false"
        >
          取消
        </button>
        <button
          class="brick-btn"
          :disabled="!statusChangeMoc || newStatus === statusChangeMoc?.status"
          @click="confirmStatusChange"
        >
          确认变更
        </button>
      </template>
    </el-dialog>

    <!-- Status Logs Dialog -->
    <el-dialog
      v-model="statusLogsVisible"
      title="状态变更记录"
      width="600px"
      :close-on-click-modal="false"
    >
      <template #title>
        <span class="brick-stud"></span>
        状态记录 - {{ statusLogsMoc?.name }}
      </template>

      <div class="status-logs">
        <div
          v-if="mocStore.statusLogsLoading"
          class="logs-loading"
        >
          加载中...
        </div>
        <div
          v-else-if="mocStore.statusLogs.length === 0"
          class="logs-empty"
        >
          暂无状态变更记录
        </div>
        <div v-else class="logs-timeline">
          <div
            v-for="log in mocStore.statusLogs"
            :key="log.id"
            class="log-item"
          >
            <div
              class="log-dot"
              :style="{ backgroundColor: getStatusColor(log.newStatus) }"
            ></div>
            <div class="log-content">
              <div class="log-status">
                <span
                  v-if="log.oldStatus"
                  class="log-status-old"
                >
                  {{ getStatusLabel(log.oldStatus) }}
                </span>
                <span v-if="log.oldStatus" class="log-arrow">→</span>
                <span
                  class="log-status-new"
                  :style="{ color: getStatusColor(log.newStatus) }"
                >
                  {{ getStatusLabel(log.newStatus) }}
                </span>
              </div>
              <div class="log-time">
                {{ new Date(log.changedAt).toLocaleString() }}
              </div>
              <div v-if="log.remark" class="log-remark">
                备注：{{ log.remark }}
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <button class="brick-btn" @click="statusLogsVisible = false">
          关闭
        </button>
      </template>
    </el-dialog>

    <!-- Cover Image Dialog -->
    <el-dialog
      v-model="coverDialogVisible"
      title="封面图片管理"
      width="500px"
      :close-on-click-modal="false"
    >
      <template #title>
        <span class="brick-stud"></span>
        封面管理 - {{ coverImageMoc?.name }}
      </template>

      <div class="cover-dialog">
        <div class="cover-preview-wrap">
          <div
            v-if="coverImageMoc?.coverImagePath"
            class="cover-preview has-image"
          >
            <img
              :src="getImageUrl(coverImageMoc.coverImagePath)"
              :alt="coverImageMoc.name"
            />
          </div>
          <div v-else class="cover-preview no-image">
            <el-icon><Picture /></el-icon>
            <span>暂无封面图片</span>
          </div>
        </div>

        <div class="cover-actions">
          <input
            ref="coverImageInput"
            type="file"
            accept=".jpg,.jpeg,.png"
            style="display: none"
            @change="handleCoverImageChange"
          />
          <button class="brick-btn" @click="triggerCoverUpload">
            <el-icon><Upload /></el-icon>
            {{ coverImageMoc?.coverImagePath ? "更换封面" : "上传封面" }}
          </button>
          <button
            v-if="coverImageMoc?.coverImagePath"
            class="brick-btn brick-btn-danger"
            @click="handleDeleteCover"
          >
            <el-icon><Delete /></el-icon>
            删除封面
          </button>
        </div>
      </div>

      <template #footer>
        <button class="brick-btn" @click="coverDialogVisible = false">
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
  flex-wrap: wrap;
}

.search-item {
  flex: 1;
  max-width: 400px;
}

.status-filter {
  min-width: 160px;
}

.brick-select {
  width: 100%;
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

.status-quick-row {
  display: flex;
  gap: $spacing-sm;
  margin-top: $spacing-md;
  padding-top: $spacing-md;
  border-top: 1px solid $color-dark-border;
  flex-wrap: wrap;
}

.status-chip {
  padding: $spacing-xs $spacing-md;
  border-radius: 20px;
  border: 1.5px solid;
  font-size: $font-size-sm;
  font-weight: 500;
  cursor: pointer;
  transition: all $transition-fast;
  user-select: none;

  &:hover {
    opacity: 0.85;
  }

  &.active {
    font-weight: 600;
  }
}

/* ========== Grid View ========== */

.moc-card {
  padding: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.moc-cover {
  position: relative;
  width: 100%;
  padding-top: 56.25%;
  overflow: hidden;
  cursor: pointer;
  background: $color-dark;

  img {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  &:hover .moc-cover-overlay {
    opacity: 1;
  }
}

.moc-cover-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: $color-gray-dark;
  padding: $spacing-lg;
  padding-top: 56.25%;
  position: relative;

  > * {
    position: absolute;
  }

  .el-icon {
    top: 35%;
    left: 50%;
    transform: translateX(-50%);
    font-size: 36px;
  }

  span {
    top: 60%;
    left: 50%;
    transform: translateX(-50%);
    font-size: $font-size-sm;
  }

  &:hover {
    color: $color-primary;
  }
}

.moc-cover-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: $spacing-xs;
  color: $color-white;
  opacity: 0;
  transition: opacity $transition-fast;
  font-size: $font-size-sm;

  .el-icon {
    font-size: 28px;
  }
}

.moc-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: $spacing-md $spacing-lg 0;
  gap: $spacing-sm;

  h3 {
    font-size: $font-size-lg;
    font-weight: 600;
    color: $color-white;
    margin: 0;
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

.status-tag {
  cursor: pointer;
  flex-shrink: 0;
}

.moc-desc {
  font-size: $font-size-sm;
  color: $color-gray-dark;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  padding: 0 $spacing-lg;
  margin-top: $spacing-xs;
}

.moc-stats {
  display: flex;
  justify-content: space-around;
  padding: $spacing-md $spacing-lg;
  border-top: 1px solid $color-dark-border;
  border-bottom: 1px solid $color-dark-border;
  margin: $spacing-sm 0;

  .stat-item {
    text-align: center;

    .stat-label {
      display: block;
      font-size: $font-size-xs;
      color: $color-gray-dark;
      margin-bottom: 2px;
    }

    .stat-value {
      font-size: $font-size-base;
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
  padding: 0 $spacing-lg;
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
  padding: $spacing-md $spacing-lg;
  border-top: 1px solid $color-dark-border;
  margin-top: $spacing-sm;

  .update-time {
    font-size: $font-size-xs;
    color: $color-gray-dark;
  }

  .moc-actions {
    display: flex;
    gap: $spacing-xs;
  }
}

.brick-btn-xs {
  padding: $spacing-xs $spacing-sm !important;
  font-size: $font-size-xs !important;

  .el-icon {
    font-size: 14px;
  }
}

/* ========== Kanban View ========== */

.kanban-container {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: $spacing-md;
  align-items: flex-start;
  min-height: 600px;
}

.kanban-column {
  background: rgba(255, 255, 255, 0.02);
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;
  display: flex;
  flex-direction: column;
  min-height: 400px;
}

.kanban-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-md;
  border-bottom: 1px solid $color-dark-border;
  border-left: 4px solid;
  border-radius: $brick-radius $brick-radius 0 0;
  background: $color-dark;
}

.kanban-title {
  font-size: $font-size-base;
  font-weight: 600;
  color: $color-white;
}

.kanban-count {
  min-width: 24px;
  height: 24px;
  padding: 0 8px;
  border-radius: 12px;
  color: $color-white;
  font-size: $font-size-xs;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}

.kanban-cards {
  padding: $spacing-sm;
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
  flex: 1;
  overflow-y: auto;
  max-height: calc(100vh - 380px);
}

.kanban-card {
  padding: $spacing-sm $spacing-md;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
    transform: translateY(-2px);
  }
}

.kanban-cover {
  width: 100%;
  padding-top: 56.25%;
  overflow: hidden;
  border-radius: calc(#{$brick-radius} - 2px);
  margin-bottom: $spacing-xs;
  position: relative;

  img {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
}

.kanban-card-title {
  font-size: $font-size-sm;
  font-weight: 600;
  color: $color-white;
  margin-bottom: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.kanban-card-desc {
  font-size: $font-size-xs;
  color: $color-gray-dark;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-bottom: $spacing-xs;
}

.kanban-card-meta {
  display: flex;
  justify-content: space-between;
  font-size: $font-size-xs;
  color: $color-gray-dark;
}

.text-danger {
  color: $color-danger;
}

.kanban-empty {
  padding: $spacing-lg;
  text-align: center;
  color: $color-gray-dark;
  font-size: $font-size-sm;
  opacity: 0.6;
}

/* ========== Detail Dialog ========== */

.moc-detail {
  .detail-cover {
    width: 100%;
    max-height: 200px;
    overflow: hidden;
    border-radius: $brick-radius;
    margin-bottom: $spacing-lg;

    img {
      width: 100%;
      height: 200px;
      object-fit: cover;
    }
  }

  .detail-header-info {
    margin-bottom: $spacing-md;
  }

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

  .detail-footer-actions {
    display: flex;
    justify-content: flex-end;
    gap: $spacing-sm;
  }
}

/* ========== Status Change Dialog ========== */

.status-change-form {
  padding: $spacing-md 0;

  .current-status {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
    padding: $spacing-md;
    background: $color-dark;
    border-radius: $brick-radius;
    margin-bottom: $spacing-lg;

    .label {
      color: $color-gray-dark;
      font-size: $font-size-sm;
    }
  }

  .status-select {
    width: 100%;
  }

  .status-option {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }

  .status-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }
}

/* ========== Status Logs Dialog ========== */

.status-logs {
  .logs-loading,
  .logs-empty {
    padding: $spacing-xl;
    text-align: center;
    color: $color-gray-dark;
  }

  .logs-timeline {
    position: relative;
    padding-left: 24px;

    &::before {
      content: "";
      position: absolute;
      left: 7px;
      top: 4px;
      bottom: 4px;
      width: 2px;
      background: $color-dark-border;
    }
  }

  .log-item {
    position: relative;
    padding-bottom: $spacing-lg;

    &:last-child {
      padding-bottom: 0;
    }
  }

  .log-dot {
    position: absolute;
    left: -21px;
    top: 4px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    border: 3px solid $color-dark;
  }

  .log-content {
    background: $color-dark;
    padding: $spacing-sm $spacing-md;
    border-radius: $brick-radius;
  }

  .log-status {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
    font-size: $font-size-base;
    font-weight: 500;
    margin-bottom: $spacing-xs;

    .log-status-old {
      color: $color-gray-dark;
    }

    .log-arrow {
      color: $color-gray-dark;
    }

    .log-status-new {
      font-weight: 600;
    }
  }

  .log-time {
    font-size: $font-size-xs;
    color: $color-gray-dark;
    margin-bottom: 2px;
  }

  .log-remark {
    font-size: $font-size-sm;
    color: $color-gray-light;
    padding-top: $spacing-xs;
    border-top: 1px solid $color-dark-border;
    margin-top: $spacing-xs;
  }
}

/* ========== Cover Dialog ========== */

.cover-dialog {
  .cover-preview-wrap {
    margin-bottom: $spacing-lg;
  }

  .cover-preview {
    width: 100%;
    padding-top: 56.25%;
    position: relative;
    border-radius: $brick-radius;
    overflow: hidden;
    border: 2px dashed $color-dark-border;

    &.has-image {
      border-style: solid;
      border-color: $color-dark-border;
      background: $color-dark;
    }

    img {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      object-fit: cover;
    }
  }

  .cover-preview.no-image {
    display: flex;
    align-items: center;
    justify-content: center;
    color: $color-gray-dark;
    background: $color-dark;

    > * {
      position: absolute;
    }

    .el-icon {
      font-size: 48px;
      top: 30%;
      left: 50%;
      transform: translateX(-50%);
    }

    span {
      font-size: $font-size-sm;
      top: 60%;
      left: 50%;
      transform: translateX(-50%);
    }
  }

  .cover-actions {
    display: flex;
    justify-content: center;
    gap: $spacing-md;
  }
}

@media (max-width: 1600px) {
  .kanban-container {
    grid-template-columns: repeat(3, 1fr);
  }
}

@media (max-width: 900px) {
  .kanban-container {
    grid-template-columns: 1fr;
  }

  .filter-row {
    flex-direction: column;
    align-items: stretch;

    .search-item,
    .status-filter {
      max-width: none;
    }
  }
}
</style>
