<script setup lang="ts">
import { ref, computed, watch } from "vue";
import { Upload, Plus, Delete, Refresh } from "@element-plus/icons-vue";
import type { UploadProps, UploadUserFile } from "element-plus";
import { api } from "@/api";
import { useAppStore, usePartsStore } from "@/stores";

const props = defineProps<{
  modelValue: boolean;
  partId: string;
  partName: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
}>();

const appStore = useAppStore();
const partsStore = usePartsStore();

const imageUrl = ref<string | null>(null);
const fileList = ref<UploadUserFile[]>([]);
const uploadVisible = ref(false);
const dragOver = ref(false);

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

async function loadImage() {
  if (!props.partId) return;

  const path = await api.getPartImagePath(props.partId);
  if (path) {
    imageUrl.value = `file://${path}`;
  } else {
    imageUrl.value = null;
  }
}

const beforeUpload: UploadProps["beforeUpload"] = (file) => {
  const isImage = file.type.startsWith("image/");
  if (!isImage) {
    appStore.showError("请选择图片文件");
    return false;
  }

  const isLt5M = file.size / 1024 / 1024 < 5;
  if (!isLt5M) {
    appStore.showError("图片大小不能超过 5MB");
    return false;
  }

  return true;
};

const handleFileChange: UploadProps["onChange"] = (uploadFile) => {
  if (uploadFile.raw) {
    const reader = new FileReader();
    reader.onload = async (e) => {
      const base64 = (e.target?.result as string).split(",")[1];
      try {
        appStore.startLoading("正在上传图片...");
        const path = await api.savePartImage(props.partId, base64);
        imageUrl.value = `file://${path}`;
        uploadVisible.value = false;
        fileList.value = [];
        appStore.showSuccess("图片上传成功");

        const part = partsStore.getPartById(props.partId);
        if (part) {
          part.imagePath = path;
          await partsStore.updatePart(part);
        }
      } catch (error) {
        console.error("Upload failed:", error);
        appStore.showError("图片上传失败");
      } finally {
        appStore.stopLoading();
      }
    };
    reader.readAsDataURL(uploadFile.raw);
  }
};

async function handleDeleteImage() {
  const confirmed = await appStore.showConfirm(
    "确定要删除这张图片吗？",
    "删除图片"
  );
  if (confirmed) {
    try {
      appStore.startLoading("正在删除图片...");
      await api.deletePartImage(props.partId);
      imageUrl.value = null;
      appStore.showSuccess("图片删除成功");

      const part = partsStore.getPartById(props.partId);
      if (part) {
        part.imagePath = undefined;
        await partsStore.updatePart(part);
      }
    } catch (error) {
      console.error("Delete failed:", error);
      appStore.showError("图片删除失败");
    } finally {
      appStore.stopLoading();
    }
  }
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  dragOver.value = true;
}

function handleDragLeave() {
  dragOver.value = false;
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  dragOver.value = false;

  const files = e.dataTransfer?.files;
  if (files && files.length > 0) {
    const file = files[0];
    if (file.type.startsWith("image/")) {
      const reader = new FileReader();
      reader.onload = async (ev) => {
        const base64 = (ev.target?.result as string).split(",")[1];
        try {
          appStore.startLoading("正在上传图片...");
          const path = await api.savePartImage(props.partId, base64);
          imageUrl.value = `file://${path}`;
          appStore.showSuccess("图片上传成功");

          const part = partsStore.getPartById(props.partId);
          if (part) {
            part.imagePath = path;
            await partsStore.updatePart(part);
          }
        } catch (error) {
          console.error("Upload failed:", error);
          appStore.showError("图片上传失败");
        } finally {
          appStore.stopLoading();
        }
      };
      reader.readAsDataURL(file);
    } else {
      appStore.showError("请选择图片文件");
    }
  }
}

watch(
  () => props.modelValue,
  (newVal) => {
    if (newVal && props.partId) {
      loadImage();
    } else {
      uploadVisible.value = false;
      fileList.value = [];
    }
  }
);
</script>

<template>
  <el-dialog
    v-model="visible"
    title="零件图片管理"
    width="600px"
    :close-on-click-modal="false"
  >
    <template #title>
      <span class="brick-stud"></span>
      零件图片管理 - {{ partName }}
    </template>

    <div class="image-manager">
      <div
        v-if="imageUrl && !uploadVisible"
        class="image-preview"
      >
        <img :src="imageUrl" alt="零件图片" class="preview-image" />
        <div class="image-actions-overlay">
          <button
            class="brick-btn brick-btn-sm"
            @click="uploadVisible = true"
          >
            <el-icon><Refresh /></el-icon>
            更换图片
          </button>
          <button
            class="brick-btn brick-btn-sm brick-btn-danger"
            @click="handleDeleteImage"
          >
            <el-icon><Delete /></el-icon>
            删除图片
          </button>
        </div>
      </div>

      <div v-else-if="!imageUrl && !uploadVisible" class="no-image">
        <el-icon class="no-image-icon"><Upload /></el-icon>
        <p class="no-image-text">暂无图片</p>
        <p class="no-image-desc">点击下方按钮上传零件实拍图</p>
        <button class="brick-btn" @click="uploadVisible = true">
          <el-icon><Plus /></el-icon>
          上传图片
        </button>
      </div>

      <div
        v-if="uploadVisible"
        class="upload-area"
        :class="{ 'drag-over': dragOver }"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <el-upload
          drag
          :auto-upload="false"
          :show-file-list="false"
          :before-upload="beforeUpload"
          :on-change="handleFileChange"
          accept="image/*"
          class="uploader"
        >
          <el-icon class="upload-icon"><Upload /></el-icon>
          <div class="upload-text">
            将图片拖到此处，或
            <em>点击选择文件</em>
          </div>
          <div class="upload-hint">支持 JPG、PNG 格式，单张不超过 5MB</div>
        </el-upload>

        <button
          class="brick-btn brick-btn-secondary brick-btn-sm cancel-btn"
          @click="uploadVisible = false"
        >
          取消
        </button>
      </div>
    </div>

    <template #footer>
      <button class="brick-btn brick-btn-secondary" @click="visible = false">
        关闭
      </button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.image-manager {
  min-height: 300px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.image-preview {
  position: relative;
  width: 100%;
  max-width: 400px;
  border-radius: $brick-radius;
  overflow: hidden;
  border: $brick-border solid $color-dark-border;

  .preview-image {
    width: 100%;
    height: auto;
    display: block;
  }

  .image-actions-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: $spacing-md;
    background: linear-gradient(transparent, rgba(0, 0, 0, 0.8));
    display: flex;
    gap: $spacing-sm;
    justify-content: center;
    opacity: 0;
    transition: opacity $transition-fast;
  }

  &:hover .image-actions-overlay {
    opacity: 1;
  }
}

.no-image {
  text-align: center;
  padding: $spacing-xl;

  .no-image-icon {
    font-size: 64px;
    color: $color-gray-dark;
    margin-bottom: $spacing-md;
  }

  .no-image-text {
    font-size: $font-size-lg;
    color: $color-gray-light;
    margin-bottom: $spacing-xs;
  }

  .no-image-desc {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-bottom: $spacing-lg;
  }
}

.upload-area {
  width: 100%;
  text-align: center;
  transition: all $transition-fast;
  border-radius: $brick-radius;
  padding: $spacing-lg;

  &.drag-over {
    background: rgba(255, 214, 0, 0.1);
    border: 2px dashed $color-primary;
  }
}

.uploader {
  width: 100%;

  :deep(.el-upload-dragger) {
    background: $color-dark !important;
    border: 2px dashed $color-dark-border !important;
    border-radius: $brick-radius !important;
    padding: $spacing-xl !important;
    transition: all $transition-fast !important;

    &:hover {
      border-color: $color-primary !important;
      background: $color-dark-light !important;
    }
  }
}

.upload-icon {
  font-size: 48px;
  color: $color-primary;
  margin-bottom: $spacing-sm;
}

.upload-text {
  font-size: $font-size-base;
  color: $color-gray-light;
  margin-bottom: $spacing-xs;

  em {
    color: $color-primary;
    font-style: normal;
    font-weight: 600;
  }
}

.upload-hint {
  font-size: $font-size-sm;
  color: $color-gray-dark;
}

.cancel-btn {
  margin-top: $spacing-md;
}
</style>
