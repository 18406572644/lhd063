<script setup lang="ts">
import { ref } from "vue";
import {
  Upload,
  Download,
  Document,
  List,
  Check,
  Warning,
} from "@element-plus/icons-vue";
import { usePartsStore, useAppStore } from "@/stores";
import { api } from "@/api";
import Papa from "papaparse";

const partsStore = usePartsStore();
const appStore = useAppStore();

const activeTab = ref("export");
const exportFormat = ref<"json" | "csv">("json");
const exportAll = ref(true);
const selectedPartIds = ref<string[]>([]);

const importFormat = ref<"json" | "csv">("json");
const importResult = ref<{ imported: number; errors: string[] } | null>(null);
const isImporting = ref(false);

async function handleExport() {
  appStore.startLoading("正在导出数据...");
  try {
    const partIds = exportAll.value ? undefined : selectedPartIds.value;
    const data = await api.exportParts(exportFormat.value, partIds);

    const blob = new Blob([data], {
      type: exportFormat.value === "json" ? "application/json" : "text/csv",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `lego-parts-${new Date().toISOString().slice(0, 10)}.${exportFormat.value}`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    appStore.showSuccess("导出成功");
  } catch (error) {
    console.error("Export failed:", error);
    appStore.showError("导出失败");
  } finally {
    appStore.stopLoading();
  }
}

async function handleFileUpload(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  isImporting.value = true;
  importResult.value = null;
  appStore.startLoading("正在导入数据...");

  try {
    const text = await file.text();

    if (importFormat.value === "csv") {
      const result = await api.importParts("csv", text);
      importResult.value = result;
    } else {
      const result = await api.importParts("json", text);
      importResult.value = result;
    }

    if (importResult.value.imported > 0) {
      appStore.showSuccess(`成功导入 ${importResult.value.imported} 条数据`);
      await partsStore.loadParts();
    }

    if (importResult.value.errors.length > 0) {
      appStore.showWarning(`有 ${importResult.value.errors.length} 条数据导入失败`);
    }
  } catch (error) {
    console.error("Import failed:", error);
    appStore.showError("导入失败，请检查文件格式");
  } finally {
    appStore.stopLoading();
    isImporting.value = false;
    input.value = "";
  }
}

function toggleSelectAll() {
  if (selectedPartIds.value.length === partsStore.parts.length) {
    selectedPartIds.value = [];
  } else {
    selectedPartIds.value = partsStore.parts.map((p) => p.id);
  }
}

function togglePartSelection(id: string) {
  const index = selectedPartIds.value.indexOf(id);
  if (index > -1) {
    selectedPartIds.value.splice(index, 1);
  } else {
    selectedPartIds.value.push(id);
  }
}

function downloadTemplate() {
  const template = [
    {
      name: "2x4 基础砖",
      partNumber: "3001",
      type: "BRICK",
      color: "红色",
      size: "2x4",
      quantity: 50,
      location: "BOX_A",
      description: "标准乐高2x4砖",
    },
  ];

  let data: string;
  let filename: string;

  if (importFormat.value === "json") {
    data = JSON.stringify(template, null, 2);
    filename = "import-template.json";
  } else {
    data = Papa.unparse(template);
    filename = "import-template.csv";
  }

  const blob = new Blob([data], {
    type: importFormat.value === "json" ? "application/json" : "text/csv",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);

  appStore.showSuccess("模板下载成功");
}
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>
        <span class="brick-stud"></span>
        导入导出
      </h1>
    </div>

    <div class="page-content">
      <div class="ie-card brick-card">
        <el-tabs v-model="activeTab" class="tabs">
          <el-tab-pane label="导出数据" name="export">
            <div class="tab-content">
              <div class="section">
                <h3>导出格式</h3>
                <div class="format-options">
                  <label
                    class="format-option"
                    :class="{ active: exportFormat === 'json' }"
                  >
                    <input
                      type="radio"
                      v-model="exportFormat"
                      value="json"
                      hidden
                    />
                    <el-icon :size="24"><Document /></el-icon>
                    <span>JSON</span>
                    <small>结构化数据，推荐使用</small>
                  </label>
                  <label
                    class="format-option"
                    :class="{ active: exportFormat === 'csv' }"
                  >
                    <input
                      type="radio"
                      v-model="exportFormat"
                      value="csv"
                      hidden
                    />
                    <el-icon :size="24"><List /></el-icon>
                    <span>CSV</span>
                    <small>表格格式，可用 Excel 打开</small>
                  </label>
                </div>
              </div>

              <div class="section">
                <h3>导出范围</h3>
                <div class="range-options">
                  <label class="range-option">
                    <input
                      type="radio"
                      v-model="exportAll"
                      :value="true"
                    />
                    <span>全部零件 ({{ partsStore.parts.length }} 种)</span>
                  </label>
                  <label class="range-option">
                    <input
                      type="radio"
                      v-model="exportAll"
                      :value="false"
                    />
                    <span>选择导出</span>
                  </label>
                </div>

                <div v-if="!exportAll" class="part-selection">
                  <div class="selection-header">
                    <label class="select-all">
                      <input
                        type="checkbox"
                        :checked="
                          selectedPartIds.length === partsStore.parts.length &&
                          partsStore.parts.length > 0
                        "
                        @change="toggleSelectAll"
                      />
                      全选 ({{ selectedPartIds.length }}/{{ partsStore.parts.length }})
                    </label>
                  </div>
                  <div class="part-list">
                    <label
                      v-for="part in partsStore.parts"
                      :key="part.id"
                      class="part-item"
                      :class="{ selected: selectedPartIds.includes(part.id) }"
                    >
                      <input
                        type="checkbox"
                        :checked="selectedPartIds.includes(part.id)"
                        @change="togglePartSelection(part.id)"
                      />
                      <span class="part-name">{{ part.name }}</span>
                      <span class="part-number">#{{ part.partNumber }}</span>
                    </label>
                  </div>
                </div>
              </div>

              <div class="action-section">
                <button class="brick-btn brick-btn-lg" @click="handleExport">
                  <el-icon><Download /></el-icon>
                  导出数据
                </button>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="导入数据" name="import">
            <div class="tab-content">
              <div class="section">
                <h3>导入格式</h3>
                <div class="format-options">
                  <label
                    class="format-option"
                    :class="{ active: importFormat === 'json' }"
                  >
                    <input
                      type="radio"
                      v-model="importFormat"
                      value="json"
                      hidden
                    />
                    <el-icon :size="24"><Document /></el-icon>
                    <span>JSON</span>
                  </label>
                  <label
                    class="format-option"
                    :class="{ active: importFormat === 'csv' }"
                  >
                    <input
                      type="radio"
                      v-model="importFormat"
                      value="csv"
                      hidden
                    />
                    <el-icon :size="24"><List /></el-icon>
                    <span>CSV</span>
                  </label>
                </div>
              </div>

              <div class="section">
                <h3>下载模板</h3>
                <p class="section-desc">
                  请先下载模板，按照模板格式整理数据后再导入
                </p>
                <button
                  class="brick-btn brick-btn-sm brick-btn-secondary"
                  @click="downloadTemplate"
                >
                  <el-icon><Download /></el-icon>
                  下载导入模板
                </button>
              </div>

              <div class="section">
                <h3>选择文件</h3>
                <div class="upload-area">
                  <input
                    type="file"
                    :accept="importFormat === 'json' ? '.json' : '.csv'"
                    @change="handleFileUpload"
                    class="file-input"
                    id="file-upload"
                    :disabled="isImporting"
                  />
                  <label for="file-upload" class="upload-label">
                    <el-icon :size="48"><Upload /></el-icon>
                    <div class="upload-text">
                      点击选择文件，或拖放文件到此处
                    </div>
                    <div class="upload-hint">
                      支持 {{ importFormat.toUpperCase() }} 格式文件
                    </div>
                  </label>
                </div>
              </div>

              <div v-if="importResult" class="import-result">
                <div class="result-header">
                  <h4>导入结果</h4>
                </div>
                <div class="result-stats">
                  <div class="result-item success">
                    <el-icon><Check /></el-icon>
                    <span>成功: {{ importResult.imported }} 条</span>
                  </div>
                  <div v-if="importResult.errors.length > 0" class="result-item error">
                    <el-icon><Warning /></el-icon>
                    <span>失败: {{ importResult.errors.length }} 条</span>
                  </div>
                </div>
                <div v-if="importResult.errors.length > 0" class="error-list">
                  <h5>错误详情</h5>
                  <ul>
                    <li v-for="(error, index) in importResult.errors" :key="index">
                      {{ error }}
                    </li>
                  </ul>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.ie-card {
  padding: 0;
  overflow: hidden;
  max-width: 800px;
  margin: 0 auto;

  :deep(.el-tabs) {
    margin: 0;
  }

  :deep(.el-tabs__header) {
    margin: 0;
    padding: 0 $spacing-lg;
    background: $color-dark-lighter;
  }
}

.tab-content {
  padding: $spacing-lg;
}

.section {
  margin-bottom: $spacing-lg;
  padding-bottom: $spacing-lg;
  border-bottom: 1px solid $color-dark-border;

  &:last-child {
    border-bottom: none;
    margin-bottom: 0;
  }

  h3 {
    font-size: $font-size-base;
    font-weight: 600;
    color: $color-white;
    margin: 0 0 $spacing-md 0;
  }

  .section-desc {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-bottom: $spacing-md;
  }
}

.format-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: $spacing-md;
}

.format-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-xs;
  padding: $spacing-lg $spacing-md;
  background: $color-dark;
  border: 2px solid $color-dark-border;
  border-radius: $brick-radius;
  cursor: pointer;
  transition: all $transition-fast;

  span {
    color: $color-gray-light;
    font-weight: 600;
  }

  small {
    font-size: 11px;
    color: $color-gray-dark;
  }

  el-icon {
    color: $color-gray-dark;
    transition: color $transition-fast;
  }

  &:hover {
    border-color: $color-dark-lighter;
  }

  &.active {
    border-color: $color-primary;
    background: rgba(255, 214, 0, 0.1);

    el-icon {
      color: $color-primary;
    }

    span {
      color: $color-primary;
    }
  }
}

.range-options {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
  margin-bottom: $spacing-md;
}

.range-option {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  cursor: pointer;
  color: $color-gray-light;
  padding: $spacing-sm $spacing-md;
  background: $color-dark;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;

  input {
    accent-color: $color-primary;
  }
}

.part-selection {
  background: $color-dark;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;
  overflow: hidden;
}

.selection-header {
  padding: $spacing-sm $spacing-md;
  background: $color-dark-lighter;
  border-bottom: 1px solid $color-dark-border;

  .select-all {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
    color: $color-gray-light;
    font-size: $font-size-sm;
    cursor: pointer;

    input {
      accent-color: $color-primary;
    }
  }
}

.part-list {
  max-height: 300px;
  overflow-y: auto;
}

.part-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm $spacing-md;
  cursor: pointer;
  transition: background $transition-fast;

  &:hover {
    background: $color-dark-lighter;
  }

  &.selected {
    background: rgba(255, 214, 0, 0.1);
  }

  input {
    accent-color: $color-primary;
  }

  .part-name {
    flex: 1;
    color: $color-gray-light;
    font-size: $font-size-sm;
  }

  .part-number {
    color: $color-gray-dark;
    font-family: monospace;
    font-size: $font-size-sm;
  }
}

.action-section {
  text-align: center;
  padding-top: $spacing-md;
}

.upload-area {
  position: relative;
}

.file-input {
  display: none;
}

.upload-label {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-xl;
  background: $color-dark;
  border: 2px dashed $color-dark-border;
  border-radius: $brick-radius;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
    background: rgba(255, 214, 0, 0.05);
  }

  el-icon {
    color: $color-primary;
  }

  .upload-text {
    color: $color-gray-light;
    font-weight: 500;
  }

  .upload-hint {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

.import-result {
  background: $color-dark;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;
  padding: $spacing-lg;
}

.result-header {
  margin-bottom: $spacing-md;

  h4 {
    color: $color-white;
    margin: 0;
    font-size: $font-size-base;
  }
}

.result-stats {
  display: flex;
  gap: $spacing-lg;
  margin-bottom: $spacing-md;
}

.result-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  font-weight: 500;

  &.success {
    color: $color-success;
  }

  &.error {
    color: $color-danger;
  }
}

.error-list {
  padding-top: $spacing-md;
  border-top: 1px solid $color-dark-border;

  h5 {
    color: $color-danger;
    margin: 0 0 $spacing-sm 0;
    font-size: $font-size-sm;
  }

  ul {
    margin: 0;
    padding-left: $spacing-lg;
    color: $color-gray-dark;
    font-size: $font-size-sm;

    li {
      margin-bottom: 4px;
    }
  }
}
</style>
