<script setup lang="ts">
import { computed, ref, watch } from "vue";
import {
  Plus,
  Delete,
  CopyDocument,
  FolderAdd,
} from "@element-plus/icons-vue";
import { useMasterDataStore } from "@/stores";
import type {
  AdvancedFilterCondition,
  AdvancedFilterGroup,
  FilterField,
  FilterOperator,
} from "@/types";
import {
  FILTER_FIELD_OPTIONS as FIELD_OPTIONS,
  FILTER_OPERATORS_BY_TYPE,
} from "@/types";

const props = defineProps<{
  modelValue: AdvancedFilterGroup;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: AdvancedFilterGroup): void;
}>();

const masterDataStore = useMasterDataStore();

function genId() {
  return "c_" + Math.random().toString(36).slice(2, 10);
}

const localGroup = ref<AdvancedFilterGroup>(cloneGroup(props.modelValue));

function cloneGroup(g: AdvancedFilterGroup): AdvancedFilterGroup {
  return JSON.parse(JSON.stringify(g));
}

watch(
  () => props.modelValue,
  (val) => {
    localGroup.value = cloneGroup(val);
  },
  { deep: true }
);

function emitUpdate() {
  emit("update:modelValue", cloneGroup(localGroup.value));
}

function fieldInfo(field: FilterField) {
  return FIELD_OPTIONS.find((f) => f.value === field);
}

function operatorOptions(field: FilterField) {
  const info = fieldInfo(field);
  if (!info) return [];
  return FILTER_OPERATORS_BY_TYPE[info.type] || [];
}

function needsValue(op: FilterOperator) {
  return !["isNull", "isNotNull", "true", "false"].includes(op);
}

function needsValueTo(op: FilterOperator) {
  return op === "between";
}

function isMultiSelect(op: FilterOperator) {
  return op === "in" || op === "notIn";
}

function addCondition(group: AdvancedFilterGroup) {
  const firstField = FIELD_OPTIONS[0].value;
  const firstOp = FILTER_OPERATORS_BY_TYPE[FIELD_OPTIONS[0].type][0].value;
  const cond: AdvancedFilterCondition = {
    id: genId(),
    field: firstField,
    operator: firstOp,
    value: undefined,
  };
  group.conditions.push(cond);
  emitUpdate();
}

function removeCondition(group: AdvancedFilterGroup, idx: number) {
  group.conditions.splice(idx, 1);
  emitUpdate();
}

function duplicateCondition(group: AdvancedFilterGroup, cond: AdvancedFilterCondition) {
  const copy: AdvancedFilterCondition = {
    ...JSON.parse(JSON.stringify(cond)),
    id: genId(),
  };
  const idx = group.conditions.findIndex((c) => c.id === cond.id);
  group.conditions.splice(idx + 1, 0, copy);
  emitUpdate();
}

function addSubGroup(parent: AdvancedFilterGroup) {
  if (!parent.groups) parent.groups = [];
  const firstField = FIELD_OPTIONS[0].value;
  const firstOp = FILTER_OPERATORS_BY_TYPE[FIELD_OPTIONS[0].type][0].value;
  parent.groups.push({
    id: genId(),
    logic: "AND",
    conditions: [
      {
        id: genId(),
        field: firstField,
        operator: firstOp,
        value: undefined,
      },
    ],
    groups: [],
  });
  emitUpdate();
}

function removeGroup(parent: AdvancedFilterGroup, idx: number) {
  parent.groups?.splice(idx, 1);
  emitUpdate();
}

function onConditionChange() {
  emitUpdate();
}

function onFieldChange(cond: AdvancedFilterCondition) {
  const info = fieldInfo(cond.field);
  const availableOps = FILTER_OPERATORS_BY_TYPE[info?.type || "text"];
  if (!availableOps.find((o) => o.value === cond.operator)) {
    cond.operator = availableOps[0].value;
  }
  cond.value = undefined;
  cond.valueTo = undefined;
  onConditionChange();
}

const locationTreeOptions = computed(() => {
  const tree = masterDataStore.buildLocationTree();
  function toOptions(nodes: any[]): any[] {
    return nodes.map((n) => ({
      value: n.code,
      label: n.name,
      children: n.children?.length ? toOptions(n.children) : undefined,
    }));
  }
  return toOptions(tree);
});

const typeOptions = computed(() =>
  masterDataStore.partTypes.map((t) => ({ label: t.name, value: t.code }))
);
const colorOptions = computed(() =>
  masterDataStore.partColors.map((c) => ({ label: c.name, value: c.name, hex: c.hex }))
);
const sizeOptions = computed(() =>
  masterDataStore.partSizes.map((s) => ({ label: s.name, value: s.name }))
);

function getSelectOptions(field: FilterField) {
  switch (field) {
    case "type":
      return typeOptions.value;
    case "color":
      return colorOptions.value;
    case "size":
      return sizeOptions.value;
    case "location":
      return locationTreeOptions.value;
    default:
      return [];
  }
}

function isTreeSelect(field: FilterField) {
  return field === "location";
}
</script>

<template>
  <div class="adv-filter">
    <div class="group-card" style="border: none; padding: 0">
      <div class="group-header">
        <el-radio-group
          v-model="localGroup.logic"
          size="small"
          @change="emitUpdate"
        >
          <el-radio-button value="AND">并且</el-radio-button>
          <el-radio-button value="OR">或者</el-radio-button>
        </el-radio-group>
        <div class="group-actions">
          <button class="icon-btn-mini" @click="addCondition(localGroup)" title="添加条件">
            <el-icon><Plus /></el-icon>
          </button>
        </div>
      </div>

      <div class="conditions-list">
        <div
          v-for="(cond, idx) in localGroup.conditions"
          :key="cond.id"
          class="condition-row"
        >
          <el-select
            v-model="cond.field"
            size="small"
            class="field-select"
            @change="onFieldChange(cond)"
          >
            <el-option
              v-for="f in FIELD_OPTIONS"
              :key="f.value"
              :label="f.label"
              :value="f.value"
            />
          </el-select>

          <el-select
            v-model="cond.operator"
            size="small"
            class="op-select"
            @change="onConditionChange"
          >
            <el-option
              v-for="op in operatorOptions(cond.field)"
              :key="op.value"
              :label="op.label"
              :value="op.value"
            />
          </el-select>

          <template v-if="needsValue(cond.operator)">
            <template v-if="fieldInfo(cond.field)?.type === 'text'">
              <el-input
                v-model="cond.value"
                size="small"
                class="value-input"
                placeholder="输入值"
                @change="onConditionChange"
              />
            </template>

            <template v-else-if="fieldInfo(cond.field)?.type === 'number'">
              <el-input-number
                v-model="cond.value"
                size="small"
                :min="0"
                class="value-input-num"
                controls-position="right"
                @change="onConditionChange"
              />
              <el-input-number
                v-if="needsValueTo(cond.operator)"
                v-model="cond.valueTo"
                size="small"
                :min="0"
                class="value-input-num"
                controls-position="right"
                placeholder="至"
                @change="onConditionChange"
              />
            </template>

            <template v-else-if="fieldInfo(cond.field)?.type === 'date'">
              <el-date-picker
                v-model="cond.value"
                type="datetime"
                size="small"
                class="value-input"
                placeholder="选择日期"
                value-format="YYYY-MM-DDTHH:mm:ss"
                @change="onConditionChange"
              />
              <el-date-picker
                v-if="needsValueTo(cond.operator)"
                v-model="cond.valueTo"
                type="datetime"
                size="small"
                class="value-input"
                placeholder="至"
                value-format="YYYY-MM-DDTHH:mm:ss"
                @change="onConditionChange"
              />
            </template>

            <template v-else-if="fieldInfo(cond.field)?.type === 'select'">
              <template v-if="isMultiSelect(cond.operator)">
                <el-tree-select
                  v-if="isTreeSelect(cond.field)"
                  v-model="cond.value"
                  :data="getSelectOptions(cond.field)"
                  multiple
                  check-strictly
                  :render-after-expand="false"
                  size="small"
                  class="value-input"
                  collapse-tags
                  collapse-tags-tooltip
                  placeholder="选择..."
                  @change="onConditionChange"
                />
                <el-select
                  v-else
                  v-model="cond.value"
                  multiple
                  size="small"
                  class="value-input"
                  collapse-tags
                  collapse-tags-tooltip
                  placeholder="选择..."
                  @change="onConditionChange"
                >
                  <el-option
                    v-for="opt in getSelectOptions(cond.field)"
                    :key="opt.value"
                    :label="opt.label"
                    :value="opt.value"
                  />
                </el-select>
              </template>
              <template v-else>
                <el-tree-select
                  v-if="isTreeSelect(cond.field)"
                  v-model="cond.value"
                  :data="getSelectOptions(cond.field)"
                  check-strictly
                  :render-after-expand="false"
                  size="small"
                  class="value-input"
                  placeholder="选择..."
                  @change="onConditionChange"
                />
                <el-select
                  v-else
                  v-model="cond.value"
                  size="small"
                  class="value-input"
                  placeholder="选择..."
                  @change="onConditionChange"
                >
                  <el-option
                    v-for="opt in getSelectOptions(cond.field)"
                    :key="opt.value"
                    :label="opt.label"
                    :value="opt.value"
                  />
                </el-select>
              </template>
            </template>
          </template>

          <div class="cond-actions">
            <button
              class="icon-btn-mini"
              @click="duplicateCondition(localGroup, cond)"
              title="复制条件"
            >
              <el-icon><CopyDocument /></el-icon>
            </button>
            <button
              class="icon-btn-mini icon-btn-danger"
              @click="removeCondition(localGroup, idx)"
              title="删除条件"
              :disabled="localGroup.conditions.length <= 1 && (!localGroup.groups || localGroup.groups.length === 0)"
            >
              <el-icon><Delete /></el-icon>
            </button>
          </div>
        </div>
      </div>

      <div
        v-for="(sub, sIdx) in localGroup.groups"
        :key="sub.id"
        class="sub-group-wrap"
      >
        <div class="logic-connector">{{ localGroup.logic }}</div>
        <div class="group-card">
          <div class="group-header">
            <el-radio-group
              v-model="sub.logic"
              size="small"
              @change="emitUpdate"
            >
              <el-radio-button value="AND">并且</el-radio-button>
              <el-radio-button value="OR">或者</el-radio-button>
            </el-radio-group>
            <div class="group-actions">
              <button
                class="icon-btn-mini"
                @click="addCondition(sub)"
                title="添加条件"
              >
                <el-icon><Plus /></el-icon>
              </button>
              <button
                class="icon-btn-mini icon-btn-danger"
                @click="removeGroup(localGroup, sIdx)"
                title="删除组"
              >
                <el-icon><Delete /></el-icon>
              </button>
            </div>
          </div>

          <div class="conditions-list">
            <div
              v-for="(cond, cIdx) in sub.conditions"
              :key="cond.id"
              class="condition-row"
            >
              <el-select
                v-model="cond.field"
                size="small"
                class="field-select"
                @change="onFieldChange(cond)"
              >
                <el-option
                  v-for="f in FIELD_OPTIONS"
                  :key="f.value"
                  :label="f.label"
                  :value="f.value"
                />
              </el-select>

              <el-select
                v-model="cond.operator"
                size="small"
                class="op-select"
                @change="onConditionChange"
              >
                <el-option
                  v-for="op in operatorOptions(cond.field)"
                  :key="op.value"
                  :label="op.label"
                  :value="op.value"
                />
              </el-select>

              <template v-if="needsValue(cond.operator)">
                <template v-if="fieldInfo(cond.field)?.type === 'text'">
                  <el-input
                    v-model="cond.value"
                    size="small"
                    class="value-input"
                    placeholder="输入值"
                    @change="onConditionChange"
                  />
                </template>

                <template v-else-if="fieldInfo(cond.field)?.type === 'number'">
                  <el-input-number
                    v-model="cond.value"
                    size="small"
                    :min="0"
                    class="value-input-num"
                    controls-position="right"
                    @change="onConditionChange"
                  />
                  <el-input-number
                    v-if="needsValueTo(cond.operator)"
                    v-model="cond.valueTo"
                    size="small"
                    :min="0"
                    class="value-input-num"
                    controls-position="right"
                    @change="onConditionChange"
                  />
                </template>

                <template v-else-if="fieldInfo(cond.field)?.type === 'date'">
                  <el-date-picker
                    v-model="cond.value"
                    type="datetime"
                    size="small"
                    class="value-input"
                    placeholder="选择日期"
                    value-format="YYYY-MM-DDTHH:mm:ss"
                    @change="onConditionChange"
                  />
                  <el-date-picker
                    v-if="needsValueTo(cond.operator)"
                    v-model="cond.valueTo"
                    type="datetime"
                    size="small"
                    class="value-input"
                    placeholder="至"
                    value-format="YYYY-MM-DDTHH:mm:ss"
                    @change="onConditionChange"
                  />
                </template>

                <template v-else-if="fieldInfo(cond.field)?.type === 'select'">
                  <template v-if="isMultiSelect(cond.operator)">
                    <el-tree-select
                      v-if="isTreeSelect(cond.field)"
                      v-model="cond.value"
                      :data="getSelectOptions(cond.field)"
                      multiple
                      check-strictly
                      :render-after-expand="false"
                      size="small"
                      class="value-input"
                      collapse-tags
                      collapse-tags-tooltip
                      placeholder="选择..."
                      @change="onConditionChange"
                    />
                    <el-select
                      v-else
                      v-model="cond.value"
                      multiple
                      size="small"
                      class="value-input"
                      collapse-tags
                      collapse-tags-tooltip
                      placeholder="选择..."
                      @change="onConditionChange"
                    >
                      <el-option
                        v-for="opt in getSelectOptions(cond.field)"
                        :key="opt.value"
                        :label="opt.label"
                        :value="opt.value"
                      />
                    </el-select>
                  </template>
                  <template v-else>
                    <el-tree-select
                      v-if="isTreeSelect(cond.field)"
                      v-model="cond.value"
                      :data="getSelectOptions(cond.field)"
                      check-strictly
                      :render-after-expand="false"
                      size="small"
                      class="value-input"
                      placeholder="选择..."
                      @change="onConditionChange"
                    />
                    <el-select
                      v-else
                      v-model="cond.value"
                      size="small"
                      class="value-input"
                      placeholder="选择..."
                      @change="onConditionChange"
                    >
                      <el-option
                        v-for="opt in getSelectOptions(cond.field)"
                        :key="opt.value"
                        :label="opt.label"
                        :value="opt.value"
                      />
                    </el-select>
                  </template>
                </template>
              </template>

              <div class="cond-actions">
                <button
                  class="icon-btn-mini"
                  @click="duplicateCondition(sub, cond)"
                  title="复制条件"
                >
                  <el-icon><CopyDocument /></el-icon>
                </button>
                <button
                  class="icon-btn-mini icon-btn-danger"
                  @click="removeCondition(sub, cIdx)"
                  title="删除条件"
                  :disabled="sub.conditions.length <= 1"
                >
                  <el-icon><Delete /></el-icon>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <button
        class="add-subgroup-btn"
        @click="addSubGroup(localGroup)"
      >
        <el-icon><FolderAdd /></el-icon>
        添加条件组
      </button>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.adv-filter {
  width: 100%;
}

.group-card {
  background: $color-dark-lighter;
  border: 2px dashed $color-dark-border;
  border-radius: $brick-radius;
  padding: $spacing-md;
  display: flex;
  flex-direction: column;
  gap: $spacing-md;
}

.group-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $spacing-sm;
}

.group-actions {
  display: flex;
  gap: $spacing-xs;
}

.icon-btn-mini {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: $color-dark-border;
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

  &:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    &:hover {
      background: $color-dark-border;
      color: $color-gray-light;
    }
  }
}

.conditions-list {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
}

.condition-row {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  flex-wrap: wrap;

  .field-select {
    width: 140px;
    flex-shrink: 0;
  }

  .op-select {
    width: 120px;
    flex-shrink: 0;
  }

  .value-input {
    flex: 1;
    min-width: 180px;
  }

  .value-input-num {
    width: 140px;
    flex-shrink: 0;
  }

  .cond-actions {
    display: flex;
    gap: $spacing-xs;
    margin-left: auto;
    flex-shrink: 0;
  }
}

.sub-group-wrap {
  position: relative;
  padding-left: $spacing-lg;

  .logic-connector {
    position: absolute;
    left: 0;
    top: 50%;
    transform: translateY(-50%);
    background: $color-primary;
    color: $color-dark;
    font-weight: 700;
    font-size: $font-size-xs;
    padding: 2px 6px;
    border-radius: $brick-radius;
  }
}

.add-subgroup-btn {
  display: inline-flex;
  align-items: center;
  gap: $spacing-xs;
  padding: $spacing-sm $spacing-md;
  background: $color-dark-border;
  border: 2px dashed $color-gray-dark;
  border-radius: $brick-radius;
  color: $color-gray-light;
  cursor: pointer;
  transition: all $transition-fast;
  font-size: $font-size-sm;
  align-self: flex-start;

  &:hover {
    background: rgba(255, 214, 0, 0.1);
    border-color: $color-primary;
    color: $color-primary;
  }
}

:deep(.el-radio-button) {
  .el-radio-button__inner {
    background: $color-dark-border;
    border-color: $color-dark-border;
    color: $color-gray-light;
    box-shadow: none;

    &:hover {
      color: $color-primary;
    }
  }

  &.is-active .el-radio-button__inner {
    background: $color-primary;
    border-color: $color-primary;
    color: $color-dark;
    font-weight: 600;
  }
}
</style>
