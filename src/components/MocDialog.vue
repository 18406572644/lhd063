<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { Plus, Delete, Search, Upload, Close } from "@element-plus/icons-vue";
import type { FormInstance, FormRules } from "element-plus";
import { usePartsStore, useMasterDataStore } from "@/stores";
import type { MocList, MocPart, MocStatus } from "@/types";
import { MOC_STATUS_OPTIONS } from "@/types";

function getImageUrl(path?: string) {
  if (!path) return "";
  if (path.startsWith("data:") || path.startsWith("file://")) return path;
  return `file://${path}`;
}

const props = defineProps<{
  modelValue: boolean;
  moc: MocList | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  save: [
    data: Omit<MocList, "id" | "createdAt" | "updatedAt"> & {
      coverRemoved?: boolean;
    }
  ];
}>();

const partsStore = usePartsStore();
const masterDataStore = useMasterDataStore();

const formRef = ref<FormInstance>();
const coverImageInput = ref<HTMLInputElement>();
const originalCoverPath = ref<string | undefined>(undefined);
const formData = ref({
  name: "",
  description: "",
  coverImage: undefined as string | undefined,
  status: "planning" as MocStatus,
});

const rules: FormRules = {
  name: [{ required: true, message: "请输入清单名称", trigger: "blur" }],
};

const mocParts = ref<MocPart[]>([]);
const searchKeyword = ref("");
const availableParts = ref<{ part: typeof partsStore.parts[0]; quantity: number }[]>([]);

const dialogTitle = computed(() => (props.moc ? "编辑 MOC 清单" : "新建 MOC 清单"));

const coverImagePreviewUrl = computed(() => {
  if (formData.value.coverImage) {
    return getImageUrl(formData.value.coverImage);
  }
  return getImageUrl(originalCoverPath.value);
});

const hasCoverImage = computed(() => {
  return !!(formData.value.coverImage || originalCoverPath.value);
});

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

const filteredAvailableParts = computed(() => {
  if (!searchKeyword.value) return availableParts.value;
  const keyword = searchKeyword.value.toLowerCase();
  return availableParts.value.filter(
    (p) =>
      p.part.name.toLowerCase().includes(keyword) ||
      p.part.partNumber.toLowerCase().includes(keyword)
  );
});

function initAvailableParts() {
  availableParts.value = partsStore.parts.map((part) => ({
    part,
    quantity: 1,
  }));
}

function addPart(item: { part: typeof partsStore.parts[0]; quantity: number }) {
  const existing = mocParts.value.find(
    (p) => p.partId === item.part.id && p.color === item.part.color
  );

  if (existing) {
    existing.quantity += item.quantity;
  } else {
    mocParts.value.push({
      partId: item.part.id,
      partNumber: item.part.partNumber,
      partName: item.part.name,
      color: item.part.color,
      quantity: item.quantity,
      inStock: item.part.quantity,
      isMissing: item.part.quantity < item.quantity,
    });
  }

  item.quantity = 1;
}

function removePart(index: number) {
  mocParts.value.splice(index, 1);
}

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

function handleCoverImageClick() {
  coverImageInput.value?.click();
}

function handleCoverImageChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;

  if (!file.type.startsWith("image/")) {
    alert("请选择图片文件");
    input.value = "";
    return;
  }

  const reader = new FileReader();
  reader.onload = (ev) => {
    formData.value.coverImage = ev.target?.result as string;
  };
  reader.readAsDataURL(file);
  input.value = "";
}

function handleRemoveCoverImage() {
  formData.value.coverImage = undefined;
  originalCoverPath.value = undefined;
}

function resetForm() {
  formData.value = {
    name: "",
    description: "",
    coverImage: undefined,
    status: "planning",
  };
  originalCoverPath.value = undefined;
  mocParts.value = [];
  searchKeyword.value = "";
  formRef.value?.resetFields();
}

async function handleSubmit() {
  if (!formRef.value) return;

  await formRef.value.validate(async (valid) => {
    if (valid) {
      if (mocParts.value.length === 0) {
        alert("请至少添加一个零件");
        return;
      }

      const coverImagePath = formData.value.coverImage?.includes(",")
        ? formData.value.coverImage.split(",")[1]
        : formData.value.coverImage;

      const coverRemoved = !!props.moc && !coverImagePath && !originalCoverPath.value;

      emit("save", {
        name: formData.value.name.trim(),
        description: formData.value.description.trim() || undefined,
        coverImagePath,
        status: props.moc ? formData.value.status : "planning",
        parts: mocParts.value.map((p) => ({
          ...p,
          inStock: 0,
          isMissing: false,
        })),
        coverRemoved,
      });

      visible.value = false;
    }
  });
}

watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal) {
      if (props.moc) {
        formData.value = {
          name: props.moc.name,
          description: props.moc.description || "",
          coverImage: undefined,
          status: props.moc.status,
        };
        originalCoverPath.value = props.moc.coverImagePath;
        mocParts.value = [...props.moc.parts];
      } else {
        resetForm();
      }
      initAvailableParts();
    }
  }
);

onMounted(() => {
  if (partsStore.parts.length === 0) {
    partsStore.loadParts();
  }
  if (masterDataStore.partColors.length === 0) {
    masterDataStore.loadAll();
  }
});
</script>

<template>
  <el-dialog
    v-model="visible"
    :title="dialogTitle"
    width="900px"
    :close-on-click-modal="false"
    @closed="resetForm"
  >
    <template #title>
      <span class="brick-stud"></span>
      {{ dialogTitle }}
    </template>

    <div class="moc-form">
      <div class="cover-image-section">
        <div
          class="cover-preview"
          @click="handleCoverImageClick"
        >
          <img
            v-if="hasCoverImage"
            :src="coverImagePreviewUrl"
            alt="封面图片"
            class="cover-image"
          />
          <template v-else>
            <el-icon class="upload-icon"><Upload /></el-icon>
            <p class="upload-text">点击上传封面</p>
          </template>
          <button
            v-if="hasCoverImage"
            class="remove-cover-btn"
            @click.stop="handleRemoveCoverImage"
          >
            <el-icon><Close /></el-icon>
          </button>
        </div>
        <input
          ref="coverImageInput"
          type="file"
          accept="image/jpeg,image/png,image/jpg"
          style="display: none"
          @change="handleCoverImageChange"
        />
      </div>

      <el-form
        ref="formRef"
        :model="formData"
        :rules="rules"
        label-width="100px"
      >
        <el-form-item label="清单名称" prop="name">
          <el-input
            v-model="formData.name"
            placeholder="请输入清单名称，如：保时捷 911"
          />
        </el-form-item>
        <el-form-item label="描述说明">
          <el-input
            v-model="formData.description"
            type="textarea"
            :rows="2"
            placeholder="请输入描述说明（可选）"
          />
        </el-form-item>
        <el-form-item v-if="moc" label="状态">
          <el-select v-model="formData.status" class="w-full">
            <el-option
              v-for="option in MOC_STATUS_OPTIONS"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
      </el-form>

      <div class="parts-section">
        <h4>已选零件 ({{ mocParts.length }})</h4>
        <div v-if="mocParts.length === 0" class="empty-parts">
          请从下方零件库中添加零件
        </div>
        <div v-else class="selected-parts">
          <div
            v-for="(part, index) in mocParts"
            :key="`${part.partId}-${part.color}-${index}`"
            class="selected-part"
          >
            <span
              class="color-dot"
              :style="{ backgroundColor: getColorHex(part.color) }"
            ></span>
            <span class="part-name">{{ part.partName }}</span>
            <span class="part-number">#{{ part.partNumber }}</span>
            <span class="part-color">{{ part.color }}</span>
            <div class="part-qty-control">
              <button
                class="qty-btn"
                @click="part.quantity = Math.max(1, part.quantity - 1)"
              >
                -
              </button>
              <span class="qty-value">{{ part.quantity }}</span>
              <button class="qty-btn" @click="part.quantity++">+</button>
            </div>
            <button
              class="remove-btn"
              @click="removePart(index)"
              title="移除"
            >
              <el-icon><Delete /></el-icon>
            </button>
          </div>
        </div>
      </div>

      <div class="parts-library">
        <h4>零件库</h4>
        <div class="search-bar">
          <el-input
            v-model="searchKeyword"
            placeholder="搜索零件名称或编号..."
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
        <div class="parts-grid">
          <div
            v-for="item in filteredAvailableParts"
            :key="item.part.id"
            class="part-item"
          >
            <div class="part-info">
              <span
                class="color-dot"
                :style="{ backgroundColor: getColorHex(item.part.color) }"
              ></span>
              <div>
                <div class="part-name">{{ item.part.name }}</div>
                <div class="part-meta">
                  #{{ item.part.partNumber }} · 库存 {{ item.part.quantity }}
                </div>
              </div>
            </div>
            <div class="part-action">
              <div class="qty-control small">
                <button
                  class="qty-btn"
                  @click="item.quantity = Math.max(1, item.quantity - 1)"
                >
                  -
                </button>
                <span class="qty-value">{{ item.quantity }}</span>
                <button class="qty-btn" @click="item.quantity++">+</button>
              </div>
              <button
                class="brick-btn brick-btn-sm"
                @click="addPart(item)"
              >
                <el-icon><Plus /></el-icon>
                添加
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <button
        class="brick-btn brick-btn-secondary"
        @click="visible = false"
      >
        取消
      </button>
      <button class="brick-btn" @click="handleSubmit">
        {{ moc ? "保存修改" : "创建清单" }}
      </button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.moc-form {
  display: flex;
  flex-direction: column;
  gap: $spacing-lg;
}

.cover-image-section {
  display: flex;
  justify-content: center;
}

.cover-preview {
  position: relative;
  width: 100%;
  max-width: 300px;
  height: 169px;
  background: $color-dark;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  overflow: hidden;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;

    .upload-icon,
    .upload-text {
      color: $color-primary;
    }
  }

  .cover-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .upload-icon {
    font-size: 40px;
    color: $color-gray-dark;
    margin-bottom: $spacing-xs;
    transition: color $transition-fast;
  }

  .upload-text {
    color: $color-gray-dark;
    font-size: $font-size-sm;
    margin: 0;
    transition: color $transition-fast;
  }

  .remove-cover-btn {
    position: absolute;
    top: $spacing-xs;
    right: $spacing-xs;
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: $color-danger;
    border: none;
    border-radius: 50%;
    color: $color-white;
    cursor: pointer;
    opacity: 0;
    transition: opacity $transition-fast;
    z-index: 1;

    &:hover {
      filter: brightness(0.9);
    }
  }

  &:hover .remove-cover-btn {
    opacity: 1;
  }
}

.parts-section,
.parts-library {
  background: $color-dark;
  padding: $spacing-lg;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;

  h4 {
    color: $color-white;
    margin: 0 0 $spacing-md 0;
    font-size: $font-size-base;
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

.empty-parts {
  text-align: center;
  padding: $spacing-lg;
  color: $color-gray-dark;
  font-size: $font-size-sm;
}

.selected-parts {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
  max-height: 200px;
  overflow-y: auto;
}

.selected-part {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm $spacing-md;
  background: $color-dark-light;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;

  .part-name {
    flex: 1;
    color: $color-white;
    font-weight: 500;
    min-width: 150px;
  }

  .part-number {
    color: $color-gray-dark;
    font-family: monospace;
    font-size: $font-size-sm;
    min-width: 80px;
  }

  .part-color {
    color: $color-gray-light;
    font-size: $font-size-sm;
    min-width: 80px;
  }
}

.part-qty-control,
.qty-control {
  display: flex;
  align-items: center;
  gap: $spacing-xs;

  &.small {
    .qty-btn {
      width: 24px;
      height: 24px;
      font-size: 14px;
    }

    .qty-value {
      min-width: 24px;
      font-size: $font-size-sm;
    }
  }
}

.qty-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $color-dark-lighter;
  border: none;
  border-radius: $brick-radius;
  color: $color-white;
  font-weight: 600;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    background: $color-primary;
    color: $color-dark;
  }
}

.qty-value {
  min-width: 32px;
  text-align: center;
  color: $color-white;
  font-weight: 600;
}

.remove-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  border-radius: $brick-radius;
  color: $color-gray-dark;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    background: $color-danger;
    color: $color-white;
  }
}

.search-bar {
  margin-bottom: $spacing-md;
}

.parts-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: $spacing-sm;
  max-height: 300px;
  overflow-y: auto;
}

.part-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-sm $spacing-md;
  background: $color-dark-light;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
  }
}

.part-info {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  flex: 1;
  min-width: 0;

  .part-name {
    color: $color-white;
    font-weight: 500;
    font-size: $font-size-sm;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .part-meta {
    color: $color-gray-dark;
    font-size: 11px;
  }
}

.part-action {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  flex-shrink: 0;
}

:deep(.w-full) {
  width: 100%;
}
</style>
