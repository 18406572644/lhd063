<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { ElForm, ElFormItem } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import { useMasterDataStore } from "@/stores";
import type { Part, LocationTreeNode, OperationLog } from "@/types";
import { OPERATION_TYPE_OPTIONS } from "@/types";
import { api } from "@/api";
import { View, Refresh, InfoFilled, Loading, Edit } from "@element-plus/icons-vue";

const props = defineProps<{
  modelValue: boolean;
  part: Part | null;
  mode?: "edit" | "view";
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  save: [
    data: Omit<Part, "id" | "createdAt" | "updatedAt">
  ];
  edit: [];
}>();

const masterDataStore = useMasterDataStore();

const activeTab = ref("info");
const formRef = ref<FormInstance>();
const formData = ref({
  name: "",
  partNumber: "",
  type: "",
  color: "",
  size: "",
  quantity: 0,
  location: "",
  description: "",
  imagePath: undefined as string | undefined,
});

const rules: FormRules = {
  name: [{ required: true, message: "请输入零件名称", trigger: "blur" }],
  partNumber: [
    { required: true, message: "请输入零件编号", trigger: "blur" },
  ],
  type: [{ required: true, message: "请选择零件类型", trigger: "change" }],
  color: [{ required: true, message: "请选择颜色", trigger: "change" }],
  size: [{ required: true, message: "请选择尺寸", trigger: "change" }],
  quantity: [
    { required: true, message: "请输入数量", trigger: "blur" },
    { type: "number", min: 0, message: "数量不能为负数", trigger: "blur" },
  ],
  location: [
    { required: true, message: "请选择存放位置", trigger: "change" },
  ],
};

const operationLogs = ref<OperationLog[]>([]);
const logsLoading = ref(false);
const compareDialogVisible = ref(false);
const selectedCompareLogs = ref<string[]>([]);
const compareBefore = ref<any>(null);
const compareAfter = ref<any>(null);
const diffFields = ref<{ field: string; before: any; after: any; label: string }[]>([]);

const typeOptions = computed(() =>
  masterDataStore.partTypes.map((t) => ({ label: t.name, value: t.code }))
);

const colorOptions = computed(() =>
  masterDataStore.partColors.map((c) => ({ label: c.name, value: c.name }))
);

const sizeOptions = computed(() =>
  masterDataStore.partSizes.map((s) => ({ label: s.name, value: s.name }))
);

const locationTreeOptions = computed(() => {
  const tree = masterDataStore.buildLocationTree();
  function toSelectOptions(
    nodes: LocationTreeNode[]
  ): { value: string; label: string; children?: any[] }[] {
    return nodes.map((node) => ({
      value: node.code,
      label: node.name,
      children: node.children?.length
        ? toSelectOptions(node.children)
        : undefined,
    }));
  }
  return toSelectOptions(tree);
});

const isViewMode = computed(() => props.mode === "view");

const dialogTitle = computed(() => {
  if (isViewMode.value) return "零件详情";
  return props.part ? "编辑零件" : "新增零件";
});

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

function getTypeName(code: string) {
  return masterDataStore.getPartTypeName(code);
}

function getLocationName(code: string) {
  return masterDataStore.getLocationName(code);
}

function getOperationTypeLabel(type: string) {
  const opt = OPERATION_TYPE_OPTIONS.find((o) => o.value === type);
  return opt?.label || type;
}

function getOperationTypeTagType(type: string) {
  const opt = OPERATION_TYPE_OPTIONS.find((o) => o.value === type);
  return opt?.type || "info";
}

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleString("zh-CN");
  } catch {
    return dateStr;
  }
}

function resetForm() {
  formData.value = {
    name: "",
    partNumber: "",
    type: "",
    color: "",
    size: "",
    quantity: 0,
    location: "",
    description: "",
    imagePath: undefined,
  };
  formRef.value?.resetFields();
}

async function loadOperationLogs() {
  if (!props.part?.id) return;
  logsLoading.value = true;
  try {
    const response = await api.getOperationLogs({
      objectType: "part",
      objectId: props.part.id,
    });
    if (response.success) {
      operationLogs.value = response.data;
    }
  } finally {
    logsLoading.value = false;
  }
}

async function handleSubmit() {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid) => {
    if (valid) {
      emit("save", {
        name: formData.value.name.trim(),
        partNumber: formData.value.partNumber.trim(),
        type: formData.value.type,
        color: formData.value.color,
        size: formData.value.size,
        quantity: formData.value.quantity,
        location: formData.value.location,
        description: formData.value.description.trim() || undefined,
        imagePath: formData.value.imagePath,
      });
    }
  });
}

function handleEdit() {
  emit("edit");
}

function parseSnapshot(snapshot?: string) {
  if (!snapshot) return null;
  try {
    return JSON.parse(snapshot);
  } catch {
    return null;
  }
}

function openCompareDialog(log: OperationLog) {
  selectedCompareLogs.value = [log.id];
  compareBefore.value = parseSnapshot(log.beforeSnapshot);
  compareAfter.value = parseSnapshot(log.afterSnapshot);
  computeDiff();
  compareDialogVisible.value = true;
}

function toggleCompareSelection(logId: string) {
  const idx = selectedCompareLogs.value.indexOf(logId);
  if (idx === -1) {
    if (selectedCompareLogs.value.length < 2) {
      selectedCompareLogs.value.push(logId);
    }
  } else {
    selectedCompareLogs.value.splice(idx, 1);
  }
  if (selectedCompareLogs.value.length === 2) {
    const logs = selectedCompareLogs.value
      .map((id) => operationLogs.value.find((l) => l.id === id)!)
      .sort(
        (a, b) =>
          new Date(a.changedAt).getTime() - new Date(b.changedAt).getTime()
      );
    const older = logs[0];
    const newer = logs[1];
    const olderSnapshot = parseSnapshot(older.afterSnapshot || older.beforeSnapshot);
    const newerSnapshot = parseSnapshot(newer.beforeSnapshot || newer.afterSnapshot);
    compareBefore.value = olderSnapshot;
    compareAfter.value = newerSnapshot;
    computeDiff();
    compareDialogVisible.value = true;
  }
}

const FIELD_LABELS: Record<string, string> = {
  name: "零件名称",
  partNumber: "零件编号",
  type: "零件类型",
  color: "颜色",
  size: "尺寸",
  quantity: "库存数量",
  location: "存放位置",
  description: "备注说明",
  imagePath: "图片路径",
  status: "状态",
  coverImagePath: "封面图片",
  parts: "零件列表",
};

function formatFieldValue(field: string, value: any) {
  if (value === undefined || value === null) return "-";
  if (field === "type") return getTypeName(value);
  if (field === "location") return getLocationName(value);
  if (field === "parts" && Array.isArray(value)) {
    return value
      .map(
        (p: any) =>
          `${p.partName || p.part_name}(${p.partNumber || p.part_number}) x${p.quantity}`
      )
      .join(", ");
  }
  if (typeof value === "boolean") return value ? "是" : "否";
  return String(value);
}

function computeDiff() {
  diffFields.value = [];
  if (!compareBefore.value && !compareAfter.value) return;

  const allKeys = new Set<string>();
  if (compareBefore.value) Object.keys(compareBefore.value).forEach((k) => allKeys.add(k));
  if (compareAfter.value) Object.keys(compareAfter.value).forEach((k) => allKeys.add(k));

  for (const key of allKeys) {
    if (["id", "createdAt", "updatedAt", "changedAt"].includes(key)) continue;
    const beforeVal = compareBefore.value?.[key];
    const afterVal = compareAfter.value?.[key];
    const beforeStr = JSON.stringify(beforeVal);
    const afterStr = JSON.stringify(afterVal);
    if (beforeStr !== afterStr) {
      diffFields.value.push({
        field: key,
        before: beforeVal,
        after: afterVal,
        label: FIELD_LABELS[key] || key,
      });
    }
  }
}

watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      if (props.part) {
        formData.value = {
          name: props.part.name,
          partNumber: props.part.partNumber,
          type: props.part.type,
          color: props.part.color,
          size: props.part.size,
          quantity: props.part.quantity,
          location: props.part.location,
          description: props.part.description || "",
          imagePath: props.part.imagePath,
        };
        if (isViewMode.value) {
          activeTab.value = "info";
          loadOperationLogs();
        }
      } else {
        resetForm();
      }
    } else {
      selectedCompareLogs.value = [];
      operationLogs.value = [];
    }
  }
);

watch(
  () => props.mode,
  () => {
    if (props.modelValue && isViewMode.value && props.part) {
      loadOperationLogs();
    }
  }
);

onMounted(() => {
  if (masterDataStore.partTypes.length === 0) {
    masterDataStore.loadAll();
  }
});
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="dialogTitle"
    :width="isViewMode ? '720px' : '560px'"
    :close-on-click-modal="false"
    @closed="resetForm"
  >
    <template #title>
      <span class="brick-stud"></span>
      {{ dialogTitle }}
    </template>

    <template v-if="isViewMode">
      <el-tabs v-model="activeTab" class="part-detail-tabs">
        <el-tab-pane label="基本信息" name="info">
          <div class="part-detail-info">
            <div class="detail-section">
              <div v-if="part?.imagePath" class="detail-image">
                <img :src="part.imagePath" alt="零件图片" />
              </div>

              <el-descriptions :column="2" border>
                <el-descriptions-item label="零件名称" :span="2">
                  {{ part?.name }}
                </el-descriptions-item>
                <el-descriptions-item label="零件编号">
                  <span class="mono-text">#{{ part?.partNumber }}</span>
                </el-descriptions-item>
                <el-descriptions-item label="零件类型">
                  {{ getTypeName(part?.type || "") }}
                </el-descriptions-item>
                <el-descriptions-item label="颜色">
                  <span class="color-option">
                    <span
                      class="color-dot"
                      :style="{ backgroundColor: getColorHex(part?.color || '') }"
                    ></span>
                    {{ part?.color }}
                  </span>
                </el-descriptions-item>
                <el-descriptions-item label="尺寸">
                  {{ part?.size }}
                </el-descriptions-item>
                <el-descriptions-item label="库存数量">
                  <span
                    class="quantity-badge"
                    :class="{ 'low-stock': (part?.quantity || 0) <= 5 }"
                  >
                    {{ part?.quantity }}
                  </span>
                </el-descriptions-item>
                <el-descriptions-item label="存放位置" :span="2">
                  {{ getLocationName(part?.location || "") }}
                </el-descriptions-item>
                <el-descriptions-item label="创建时间">
                  {{ formatDate(part?.createdAt || "") }}
                </el-descriptions-item>
                <el-descriptions-item label="更新时间">
                  {{ formatDate(part?.updatedAt || "") }}
                </el-descriptions-item>
                <el-descriptions-item v-if="part?.description" label="备注说明" :span="2">
                  {{ part.description }}
                </el-descriptions-item>
              </el-descriptions>
            </div>
          </div>
        </el-tab-pane>

        <el-tab-pane label="变更历史" name="history">
          <div class="history-header">
            <div class="history-tip">
              <el-icon><InfoFilled /></el-icon>
              勾选两条记录可对比任意两个版本的差异
            </div>
            <button class="brick-btn brick-btn-sm brick-btn-secondary" @click="loadOperationLogs">
              <el-icon><Refresh /></el-icon>
              刷新
            </button>
          </div>

          <div v-if="logsLoading" class="loading-state">
            <el-icon class="is-loading"><Loading /></el-icon>
            加载中...
          </div>
          <div v-else-if="operationLogs.length === 0" class="empty-state">
            暂无变更记录
          </div>
          <div v-else class="history-list">
            <div
              v-for="log in operationLogs"
              :key="log.id"
              class="history-item"
              :class="{ selected: selectedCompareLogs.includes(log.id) }"
              @click="toggleCompareSelection(log.id)"
            >
              <div class="history-item-header">
                <el-checkbox
                  :model-value="selectedCompareLogs.includes(log.id)"
                  @click.stop
                  @change="() => toggleCompareSelection(log.id)"
                >
                  对比
                </el-checkbox>
                <el-tag :type="getOperationTypeTagType(log.operationType)" effect="dark" size="small">
                  {{ getOperationTypeLabel(log.operationType) }}
                </el-tag>
                <span class="history-time">{{ formatDate(log.changedAt) }}</span>
                <button
                  class="link-btn"
                  @click.stop="openCompareDialog(log)"
                >
                  <el-icon><View /></el-icon>
                  查看本次变更
                </button>
              </div>
              <div class="history-item-body">
                <span class="field-label">变更前：</span>
                <span class="snapshot-preview">
                  {{ log.beforeSnapshot ? '有数据' : '无' }}
                </span>
                <span class="arrow">→</span>
                <span class="field-label">变更后：</span>
                <span class="snapshot-preview">
                  {{ log.afterSnapshot ? '有数据' : '无' }}
                </span>
              </div>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </template>

    <template v-else>
      <el-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-width="100px"
        class="part-form"
      >
        <el-row :gutter="16">
          <el-col :span="16">
            <el-form-item label="零件名称" prop="name">
              <el-input v-model="formData.name" placeholder="请输入零件名称" />
            </el-form-item>
          </el-col>
          <el-col :span="8">
            <el-form-item label="零件编号" prop="partNumber">
              <el-input
                v-model="formData.partNumber"
                placeholder="如: 3001"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="零件类型" prop="type">
              <el-select
                v-model="formData.type"
                placeholder="请选择零件类型"
                class="w-full"
              >
                <el-option
                  v-for="opt in typeOptions"
                  :key="opt.value"
                  :label="opt.label"
                  :value="opt.value"
                />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="颜色" prop="color">
              <el-select
                v-model="formData.color"
                placeholder="请选择颜色"
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
            </el-form-item>
          </el-col>
        </el-row>

        <el-row :gutter="16">
          <el-col :span="12">
            <el-form-item label="尺寸" prop="size">
              <el-select
                v-model="formData.size"
                placeholder="请选择尺寸"
                class="w-full"
              >
                <el-option
                  v-for="opt in sizeOptions"
                  :key="opt.value"
                  :label="opt.label"
                  :value="opt.value"
                />
              </el-select>
            </el-form-item>
          </el-col>
          <el-col :span="12">
            <el-form-item label="库存数量" prop="quantity">
              <el-input-number
                v-model="formData.quantity"
                :min="0"
                class="w-full"
                controls-position="right"
              />
            </el-form-item>
          </el-col>
        </el-row>

        <el-form-item label="存放位置" prop="location">
          <el-tree-select
            v-model="formData.location"
            :data="locationTreeOptions"
            placeholder="请选择存放位置"
            check-strictly
            :render-after-expand="false"
            class="w-full"
          />
        </el-form-item>

        <el-form-item label="备注说明">
          <el-input
            v-model="formData.description"
            type="textarea"
            :rows="3"
            placeholder="请输入备注说明（可选）"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
      </el-form>
    </template>

    <template #footer>
      <button
        class="brick-btn brick-btn-secondary"
        @click="visible = false"
      >
        {{ isViewMode ? '关闭' : '取消' }}
      </button>
      <template v-if="isViewMode">
        <button class="brick-btn" @click="handleEdit">
          <el-icon><Edit /></el-icon>
          编辑
        </button>
      </template>
      <template v-else>
        <button class="brick-btn" @click="handleSubmit">
          {{ part ? "保存修改" : "添加零件" }}
        </button>
      </template>
    </template>
  </el-dialog>

  <el-dialog
    v-model="compareDialogVisible"
    title="版本差异对比"
    width="640px"
  >
    <div v-if="diffFields.length === 0" class="empty-state">
      两个版本没有差异
    </div>
    <div v-else class="diff-table">
      <el-table :data="diffFields" border stripe>
        <el-table-column prop="label" label="字段" width="140" />
        <el-table-column label="变更前">
          <template #default="{ row }">
            <span class="diff-before">{{ formatFieldValue(row.field, row.before) }}</span>
          </template>
        </el-table-column>
        <el-table-column label="变更后">
          <template #default="{ row }">
            <span class="diff-after">{{ formatFieldValue(row.field, row.after) }}</span>
          </template>
        </el-table-column>
      </el-table>
    </div>
    <template #footer>
      <button class="brick-btn brick-btn-secondary" @click="compareDialogVisible = false">
        关闭
      </button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.color-dot {
  display: inline-block;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  margin-right: $spacing-sm;
  vertical-align: middle;
  border: 2px solid $color-dark-border;
}

.part-form {
  padding-top: $spacing-sm;
}

.part-detail-tabs {
  margin-top: -$spacing-md;

  :deep(.el-tabs__item) {
    color: $color-gray-light;
  }

  :deep(.el-tabs__item.is-active) {
    color: $color-primary;
  }
}

.part-detail-info {
  max-height: 500px;
  overflow-y: auto;
  padding: $spacing-sm 0;
}

.detail-section {
  display: flex;
  flex-direction: column;
  gap: $spacing-md;
}

.detail-image {
  text-align: center;

  img {
    max-width: 200px;
    max-height: 200px;
    border-radius: $brick-radius;
    border: 1px solid $color-dark-border;
  }
}

.mono-text {
  font-family: monospace;
}

.quantity-badge {
  display: inline-block;
  padding: 2px 10px;
  background: $color-success;
  color: white;
  border-radius: 12px;
  font-weight: 600;

  &.low-stock {
    background: $color-danger;
  }
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: $spacing-md;

  .history-tip {
    display: flex;
    align-items: center;
    gap: $spacing-xs;
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
  max-height: 400px;
  overflow-y: auto;
}

.history-item {
  padding: $spacing-md;
  background: $color-dark-lighter;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
  }

  &.selected {
    border-color: $color-primary;
    background: rgba($color-primary, 0.1);
  }
}

.history-item-header {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  margin-bottom: $spacing-sm;

  .history-time {
    flex: 1;
    color: $color-gray-dark;
    font-size: $font-size-sm;
  }
}

.history-item-body {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  font-size: $font-size-sm;
  color: $color-gray-light;

  .field-label {
    color: $color-gray-dark;
  }

  .arrow {
    color: $color-primary;
    font-weight: bold;
  }

  .snapshot-preview {
    font-family: monospace;
    color: $color-gray-light;
  }
}

.link-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  color: $color-primary;
  font-size: $font-size-sm;
  cursor: pointer;
  padding: 0;

  &:hover {
    text-decoration: underline;
  }
}

.loading-state,
.empty-state {
  text-align: center;
  padding: $spacing-xl 0;
  color: $color-gray-dark;
}

.diff-table {
  .diff-before {
    color: $color-danger;
    text-decoration: line-through;
  }

  .diff-after {
    color: $color-success;
    font-weight: 500;
  }
}

:deep(.el-descriptions) {
  --el-descriptions-table-border: 1px solid $color-dark-border;
  --el-descriptions-item-label-bg: $color-dark-lighter;
  --el-descriptions-item-content-bg: $color-dark;
  --el-text-color-primary: $color-white;
  --el-text-color-regular: $color-gray-light;
}
</style>
