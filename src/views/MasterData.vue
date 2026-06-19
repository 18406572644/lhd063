<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Plus, Edit, Delete, Refresh } from "@element-plus/icons-vue";
import { useMasterDataStore, useAppStore } from "@/stores";
import type { PartType, PartColor, PartSize, Location } from "@/types";

const masterDataStore = useMasterDataStore();
const appStore = useAppStore();

const activeTab = ref("types");

const typeDialogVisible = ref(false);
const editingType = ref<PartType | null>(null);
const typeForm = ref({
  name: "",
  code: "",
  description: "",
});

const colorDialogVisible = ref(false);
const editingColor = ref<PartColor | null>(null);
const colorForm = ref({
  name: "",
  hex: "#FFD600",
  legoCode: "",
});

const sizeDialogVisible = ref(false);
const editingSize = ref<PartSize | null>(null);
const sizeForm = ref({
  name: "",
  width: 0,
  height: 0,
  unit: "stud",
});

const locationDialogVisible = ref(false);
const editingLocation = ref<Location | null>(null);
const locationForm = ref({
  name: "",
  code: "",
  description: "",
  parentId: undefined as string | undefined,
});

async function loadData() {
  await masterDataStore.loadAll();
}

function openTypeDialog(type?: PartType) {
  if (type) {
    editingType.value = { ...type };
    typeForm.value = {
      name: type.name,
      code: type.code,
      description: type.description || "",
    };
  } else {
    editingType.value = null;
    typeForm.value = {
      name: "",
      code: "",
      description: "",
    };
  }
  typeDialogVisible.value = true;
}

async function saveType() {
  if (!typeForm.value.name.trim() || !typeForm.value.code.trim()) {
    appStore.showError("请填写名称和编码");
    return;
  }

  if (editingType.value) {
    await masterDataStore.updatePartType({
      ...editingType.value,
      name: typeForm.value.name.trim(),
      code: typeForm.value.code.trim().toUpperCase(),
      description: typeForm.value.description.trim() || undefined,
    });
    appStore.showSuccess("更新成功");
  } else {
    await masterDataStore.addPartType({
      name: typeForm.value.name.trim(),
      code: typeForm.value.code.trim().toUpperCase(),
      description: typeForm.value.description.trim() || undefined,
    });
    appStore.showSuccess("添加成功");
  }
  typeDialogVisible.value = false;
}

async function deleteType(type: PartType) {
  const confirmed = await appStore.showConfirm(
    `确定要删除类型「${type.name}」吗？`,
    "删除类型"
  );
  if (confirmed) {
    await masterDataStore.deletePartType(type.id);
    appStore.showSuccess("删除成功");
  }
}

function openColorDialog(color?: PartColor) {
  if (color) {
    editingColor.value = { ...color };
    colorForm.value = {
      name: color.name,
      hex: color.hex,
      legoCode: color.legoCode || "",
    };
  } else {
    editingColor.value = null;
    colorForm.value = {
      name: "",
      hex: "#FFD600",
      legoCode: "",
    };
  }
  colorDialogVisible.value = true;
}

async function saveColor() {
  if (!colorForm.value.name.trim()) {
    appStore.showError("请填写颜色名称");
    return;
  }

  if (editingColor.value) {
    await masterDataStore.updatePartColor({
      ...editingColor.value,
      name: colorForm.value.name.trim(),
      hex: colorForm.value.hex,
      legoCode: colorForm.value.legoCode.trim() || undefined,
    });
    appStore.showSuccess("更新成功");
  } else {
    await masterDataStore.addPartColor({
      name: colorForm.value.name.trim(),
      hex: colorForm.value.hex,
      legoCode: colorForm.value.legoCode.trim() || undefined,
    });
    appStore.showSuccess("添加成功");
  }
  colorDialogVisible.value = false;
}

async function deleteColor(color: PartColor) {
  const confirmed = await appStore.showConfirm(
    `确定要删除颜色「${color.name}」吗？`,
    "删除颜色"
  );
  if (confirmed) {
    await masterDataStore.deletePartColor(color.id);
    appStore.showSuccess("删除成功");
  }
}

function openSizeDialog(size?: PartSize) {
  if (size) {
    editingSize.value = { ...size };
    sizeForm.value = {
      name: size.name,
      width: size.width,
      height: size.height,
      unit: size.unit,
    };
  } else {
    editingSize.value = null;
    sizeForm.value = {
      name: "",
      width: 1,
      height: 1,
      unit: "stud",
    };
  }
  sizeDialogVisible.value = true;
}

async function saveSize() {
  if (!sizeForm.value.name.trim()) {
    appStore.showError("请填写尺寸名称");
    return;
  }

  if (editingSize.value) {
    await masterDataStore.updatePartSize({
      ...editingSize.value,
      name: sizeForm.value.name.trim(),
      width: sizeForm.value.width,
      height: sizeForm.value.height,
      unit: sizeForm.value.unit,
    });
    appStore.showSuccess("更新成功");
  } else {
    await masterDataStore.addPartSize({
      name: sizeForm.value.name.trim(),
      width: sizeForm.value.width,
      height: sizeForm.value.height,
      unit: sizeForm.value.unit,
    });
    appStore.showSuccess("添加成功");
  }
  sizeDialogVisible.value = false;
}

async function deleteSize(size: PartSize) {
  const confirmed = await appStore.showConfirm(
    `确定要删除尺寸「${size.name}」吗？`,
    "删除尺寸"
  );
  if (confirmed) {
    await masterDataStore.deletePartSize(size.id);
    appStore.showSuccess("删除成功");
  }
}

function openLocationDialog(location?: Location) {
  if (location) {
    editingLocation.value = { ...location };
    locationForm.value = {
      name: location.name,
      code: location.code,
      description: location.description || "",
      parentId: location.parentId,
    };
  } else {
    editingLocation.value = null;
    locationForm.value = {
      name: "",
      code: "",
      description: "",
      parentId: undefined,
    };
  }
  locationDialogVisible.value = true;
}

async function saveLocation() {
  if (!locationForm.value.name.trim() || !locationForm.value.code.trim()) {
    appStore.showError("请填写名称和编码");
    return;
  }

  if (editingLocation.value) {
    await masterDataStore.updateLocation({
      ...editingLocation.value,
      name: locationForm.value.name.trim(),
      code: locationForm.value.code.trim().toUpperCase(),
      description: locationForm.value.description.trim() || undefined,
      parentId: locationForm.value.parentId,
    });
    appStore.showSuccess("更新成功");
  } else {
    await masterDataStore.addLocation({
      name: locationForm.value.name.trim(),
      code: locationForm.value.code.trim().toUpperCase(),
      description: locationForm.value.description.trim() || undefined,
      parentId: locationForm.value.parentId,
    });
    appStore.showSuccess("添加成功");
  }
  locationDialogVisible.value = false;
}

async function deleteLocation(location: Location) {
  const confirmed = await appStore.showConfirm(
    `确定要删除位置「${location.name}」吗？`,
    "删除位置"
  );
  if (confirmed) {
    await masterDataStore.deleteLocation(location.id);
    appStore.showSuccess("删除成功");
  }
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
        基础数据管理
      </h1>
      <div class="header-actions">
        <button class="brick-btn brick-btn-sm" @click="loadData">
          <el-icon><Refresh /></el-icon>
          刷新
        </button>
      </div>
    </div>

    <div class="page-content">
      <div class="master-tabs brick-card">
        <el-tabs v-model="activeTab" class="tabs">
          <el-tab-pane label="零件类型" name="types">
            <div class="tab-header">
              <span class="tab-desc">管理零件的分类类型</span>
              <button class="brick-btn brick-btn-sm" @click="openTypeDialog()">
                <el-icon><Plus /></el-icon>
                添加类型
              </button>
            </div>
            <div class="data-grid">
              <div
                v-for="type in masterDataStore.partTypes"
                :key="type.id"
                class="data-item"
              >
                <div class="item-info">
                  <div class="item-name">{{ type.name }}</div>
                  <div class="item-code">{{ type.code }}</div>
                  <div v-if="type.description" class="item-desc">
                    {{ type.description }}
                  </div>
                </div>
                <div class="item-actions">
                  <button
                    class="icon-btn"
                    @click="openTypeDialog(type)"
                    title="编辑"
                  >
                    <el-icon><Edit /></el-icon>
                  </button>
                  <button
                    class="icon-btn icon-btn-danger"
                    @click="deleteType(type)"
                    title="删除"
                  >
                    <el-icon><Delete /></el-icon>
                  </button>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="颜色" name="colors">
            <div class="tab-header">
              <span class="tab-desc">管理零件颜色列表</span>
              <button class="brick-btn brick-btn-sm" @click="openColorDialog()">
                <el-icon><Plus /></el-icon>
                添加颜色
              </button>
            </div>
            <div class="data-grid colors-grid">
              <div
                v-for="color in masterDataStore.partColors"
                :key="color.id"
                class="data-item"
              >
                <div class="color-preview" :style="{ backgroundColor: color.hex }"></div>
                <div class="item-info">
                  <div class="item-name">{{ color.name }}</div>
                  <div class="item-code">{{ color.hex }}</div>
                  <div v-if="color.legoCode" class="item-desc">
                    乐高代码: {{ color.legoCode }}
                  </div>
                </div>
                <div class="item-actions">
                  <button
                    class="icon-btn"
                    @click="openColorDialog(color)"
                    title="编辑"
                  >
                    <el-icon><Edit /></el-icon>
                  </button>
                  <button
                    class="icon-btn icon-btn-danger"
                    @click="deleteColor(color)"
                    title="删除"
                  >
                    <el-icon><Delete /></el-icon>
                  </button>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="尺寸" name="sizes">
            <div class="tab-header">
              <span class="tab-desc">管理零件尺寸规格</span>
              <button class="brick-btn brick-btn-sm" @click="openSizeDialog()">
                <el-icon><Plus /></el-icon>
                添加尺寸
              </button>
            </div>
            <div class="data-grid">
              <div
                v-for="size in masterDataStore.partSizes"
                :key="size.id"
                class="data-item"
              >
                <div class="size-preview">
                  <div
                    class="size-block"
                    :style="{
                      width: Math.min(size.width * 16, 64) + 'px',
                      height: Math.min(size.height * 16, 64) + 'px',
                    }"
                  >
                    <div class="brick-stud-row">
                      <span
                        v-for="w in Math.min(size.width, 4)"
                        :key="'w' + w"
                        class="brick-stud"
                        style="width: 10px; height: 10px;"
                      ></span>
                    </div>
                  </div>
                </div>
                <div class="item-info">
                  <div class="item-name">{{ size.name }}</div>
                  <div class="item-code">
                    {{ size.width }} x {{ size.height }} {{ size.unit }}
                  </div>
                </div>
                <div class="item-actions">
                  <button
                    class="icon-btn"
                    @click="openSizeDialog(size)"
                    title="编辑"
                  >
                    <el-icon><Edit /></el-icon>
                  </button>
                  <button
                    class="icon-btn icon-btn-danger"
                    @click="deleteSize(size)"
                    title="删除"
                  >
                    <el-icon><Delete /></el-icon>
                  </button>
                </div>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="存放位置" name="locations">
            <div class="tab-header">
              <span class="tab-desc">管理零件的存放位置</span>
              <button
                class="brick-btn brick-btn-sm"
                @click="openLocationDialog()"
              >
                <el-icon><Plus /></el-icon>
                添加位置
              </button>
            </div>
            <div class="data-grid">
              <div
                v-for="location in masterDataStore.locations"
                :key="location.id"
                class="data-item"
              >
                <div class="item-info">
                  <div class="item-name">{{ location.name }}</div>
                  <div class="item-code">{{ location.code }}</div>
                  <div v-if="location.description" class="item-desc">
                    {{ location.description }}
                  </div>
                </div>
                <div class="item-actions">
                  <button
                    class="icon-btn"
                    @click="openLocationDialog(location)"
                    title="编辑"
                  >
                    <el-icon><Edit /></el-icon>
                  </button>
                  <button
                    class="icon-btn icon-btn-danger"
                    @click="deleteLocation(location)"
                    title="删除"
                  >
                    <el-icon><Delete /></el-icon>
                  </button>
                </div>
              </div>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>

    <el-dialog
      v-model="typeDialogVisible"
      :title="editingType ? '编辑零件类型' : '添加零件类型'"
      width="420px"
    >
      <el-form label-width="80px">
        <el-form-item label="名称">
          <el-input v-model="typeForm.name" placeholder="如: 砖" />
        </el-form-item>
        <el-form-item label="编码">
          <el-input v-model="typeForm.code" placeholder="如: BRICK" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="typeForm.description"
            type="textarea"
            :rows="2"
            placeholder="可选"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="typeDialogVisible = false"
        >
          取消
        </button>
        <button class="brick-btn" @click="saveType">保存</button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="colorDialogVisible"
      :title="editingColor ? '编辑颜色' : '添加颜色'"
      width="420px"
    >
      <el-form label-width="80px">
        <el-form-item label="名称">
          <el-input v-model="colorForm.name" placeholder="如: 红色" />
        </el-form-item>
        <el-form-item label="颜色值">
          <el-color-picker v-model="colorForm.hex" />
          <span class="color-hex-text">{{ colorForm.hex }}</span>
        </el-form-item>
        <el-form-item label="乐高代码">
          <el-input v-model="colorForm.legoCode" placeholder="可选，如: 21" />
        </el-form-item>
      </el-form>
      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="colorDialogVisible = false"
        >
          取消
        </button>
        <button class="brick-btn" @click="saveColor">保存</button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="sizeDialogVisible"
      :title="editingSize ? '编辑尺寸' : '添加尺寸'"
      width="420px"
    >
      <el-form label-width="80px">
        <el-form-item label="名称">
          <el-input v-model="sizeForm.name" placeholder="如: 2x4" />
        </el-form-item>
        <el-form-item label="宽度">
          <el-input-number v-model="sizeForm.width" :min="0.5" :step="0.5" />
        </el-form-item>
        <el-form-item label="高度">
          <el-input-number v-model="sizeForm.height" :min="0.5" :step="0.5" />
        </el-form-item>
        <el-form-item label="单位">
          <el-input v-model="sizeForm.unit" placeholder="stud" />
        </el-form-item>
      </el-form>
      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="sizeDialogVisible = false"
        >
          取消
        </button>
        <button class="brick-btn" @click="saveSize">保存</button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="locationDialogVisible"
      :title="editingLocation ? '编辑位置' : '添加位置'"
      width="420px"
    >
      <el-form label-width="80px">
        <el-form-item label="名称">
          <el-input v-model="locationForm.name" placeholder="如: 收纳盒 A" />
        </el-form-item>
        <el-form-item label="编码">
          <el-input v-model="locationForm.code" placeholder="如: BOX_A" />
        </el-form-item>
        <el-form-item label="描述">
          <el-input
            v-model="locationForm.description"
            type="textarea"
            :rows="2"
            placeholder="可选"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="locationDialogVisible = false"
        >
          取消
        </button>
        <button class="brick-btn" @click="saveLocation">保存</button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.master-tabs {
  padding: 0;
  overflow: hidden;

  :deep(.el-tabs) {
    margin: 0;
  }

  :deep(.el-tabs__header) {
    margin: 0;
    padding: 0 $spacing-lg;
    background: $color-dark-lighter;
  }
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-md $spacing-lg;
  border-bottom: 1px solid $color-dark-border;

  .tab-desc {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

.data-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: $spacing-md;
  padding: $spacing-lg;
}

.colors-grid {
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
}

.data-item {
  display: flex;
  align-items: center;
  gap: $spacing-md;
  padding: $spacing-md;
  background: $color-dark;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
  }

  .item-info {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-weight: 600;
    color: $color-white;
    margin-bottom: 2px;
  }

  .item-code {
    font-size: $font-size-sm;
    color: $color-primary;
    font-family: monospace;
  }

  .item-desc {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-top: 4px;
  }

  .item-actions {
    display: flex;
    gap: $spacing-xs;
  }
}

.icon-btn {
  width: 32px;
  height: 32px;
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

.color-preview {
  width: 40px;
  height: 40px;
  border-radius: $brick-radius;
  border: 2px solid $color-dark-border;
  flex-shrink: 0;
}

.size-preview {
  width: 60px;
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;

  .size-block {
    background: $color-primary;
    border-radius: 3px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    padding: 4px;
    box-shadow: inset 0 -3px 0 rgba(0, 0, 0, 0.2);
  }

  .brick-stud-row {
    display: flex;
    gap: 2px;
  }
}

.color-hex-text {
  margin-left: $spacing-sm;
  font-family: monospace;
  color: $color-gray-light;
}
</style>
