<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { ElForm, ElFormItem } from "element-plus";
import type { FormInstance, FormRules } from "element-plus";
import { useMasterDataStore } from "@/stores";
import type { Part, LocationTreeNode } from "@/types";

const props = defineProps<{
  modelValue: boolean;
  part: Part | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  save: [
    data: Omit<Part, "id" | "createdAt" | "updatedAt">
  ];
}>();

const masterDataStore = useMasterDataStore();

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

const dialogTitle = computed(() =>
  props.part ? "编辑零件" : "新增零件"
);

const visible = computed({
  get: () => props.modelValue,
  set: (val) => emit("update:modelValue", val),
});

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
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
      } else {
        resetForm();
      }
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
    width="560px"
    :close-on-click-modal="false"
    @closed="resetForm"
  >
    <template #title>
      <span class="brick-stud"></span>
      {{ dialogTitle }}
    </template>

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

    <template #footer>
      <button
        class="brick-btn brick-btn-secondary"
        @click="visible = false"
      >
        取消
      </button>
      <button class="brick-btn" @click="handleSubmit">
        {{ part ? "保存修改" : "添加零件" }}
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
</style>
