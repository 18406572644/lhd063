<script setup lang="ts">
import { ref, watch, computed } from "vue";
import {
  Search,
  Refresh,
  Picture,
  PictureFilled,
  MagicStick,
} from "@element-plus/icons-vue";
import { useMasterDataStore } from "@/stores";
import type {
  PartFilter,
  LocationTreeNode,
  DynamicTimeOption,
  AdvancedFilterGroup,
} from "@/types";
import { DYNAMIC_TIME_OPTIONS } from "@/types";
import AdvancedFilterBuilder from "./AdvancedFilterBuilder.vue";

const props = defineProps<{
  modelValue: PartFilter;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", val: PartFilter): void;
  (e: "search"): void;
  (e: "reset"): void;
}>();

const masterDataStore = useMasterDataStore();

const local = ref<PartFilter>(cloneFilter(props.modelValue));
const expandedSection = ref<Set<string>>(new Set());
const advancedMode = ref(!!props.modelValue.advancedMode);

function cloneFilter(f: PartFilter): PartFilter {
  return JSON.parse(JSON.stringify(f || {}));
}

watch(
  () => props.modelValue,
  (val) => {
    local.value = cloneFilter(val);
    advancedMode.value = !!val.advancedMode;
  },
  { deep: true }
);

function emitChange() {
  const copy = cloneFilter(local.value);
  copy.advancedMode = advancedMode.value;
  emit("update:modelValue", copy);
}

function toggleSection(section: string) {
  if (expandedSection.value.has(section)) {
    expandedSection.value.delete(section);
  } else {
    expandedSection.value.add(section);
  }
}

const typeOptions = computed(() =>
  masterDataStore.partTypes.map((t) => ({ label: t.name, value: t.code }))
);
const colorOptions = computed(() =>
  masterDataStore.partColors.map((c) => ({ label: c.name, value: c.name, hex: c.hex }))
);
const sizeOptions = computed(() =>
  masterDataStore.partSizes.map((s) => ({ label: s.name, value: s.name }))
);

const locationTreeOptions = computed(() => {
  const tree = masterDataStore.buildLocationTree();
  function toOptions(
    nodes: LocationTreeNode[]
  ): { value: string; label: string; children?: any[] }[] {
    return nodes.map((n) => ({
      value: n.code,
      label: n.name,
      children: n.children?.length ? toOptions(n.children) : undefined,
    }));
  }
  return toOptions(tree);
});

function getColorHex(colorName: string) {
  return masterDataStore.getPartColorHex(colorName);
}

function useMultiType(use: boolean) {
  if (use) {
    if (local.value.type) {
      local.value.types = [local.value.type];
      local.value.type = undefined;
    } else if (!local.value.types) {
      local.value.types = [];
    }
  } else {
    local.value.types = undefined;
    local.value.typeLogic = undefined;
  }
  emitChange();
}

function useMultiColor(use: boolean) {
  if (use) {
    if (local.value.color) {
      local.value.colors = [local.value.color];
      local.value.color = undefined;
    } else if (!local.value.colors) {
      local.value.colors = [];
    }
  } else {
    local.value.colors = undefined;
    local.value.colorLogic = undefined;
  }
  emitChange();
}

function useMultiSize(use: boolean) {
  if (use) {
    if (local.value.size) {
      local.value.sizes = [local.value.size];
      local.value.size = undefined;
    } else if (!local.value.sizes) {
      local.value.sizes = [];
    }
  } else {
    local.value.sizes = undefined;
    local.value.sizeLogic = undefined;
  }
  emitChange();
}

function toggleAdvanced() {
  advancedMode.value = !advancedMode.value;
  if (advancedMode.value && !local.value.advancedConditions) {
    local.value.advancedConditions = {
      id: "root_" + Math.random().toString(36).slice(2, 10),
      logic: "AND",
      conditions: [
        {
          id: "c_" + Math.random().toString(36).slice(2, 10),
          field: "name",
          operator: "contains",
        },
      ],
      groups: [],
    };
  }
  emitChange();
}

function onAdvancedGroupChange(g: AdvancedFilterGroup) {
  local.value.advancedConditions = g;
  emitChange();
}

function onReset() {
  local.value = {};
  advancedMode.value = false;
  emitChange();
  emit("reset");
}

function onSearch() {
  emitChange();
  emit("search");
}

const dynamicTimeOptions = DYNAMIC_TIME_OPTIONS;

const activeChips = computed(() => {
  const chips: { label: string; clear: () => void }[] = [];
  const f = local.value;

  if (f.keyword) {
    chips.push({
      label: `关键词: ${f.keyword}`,
      clear: () => {
        f.keyword = undefined;
        emitChange();
      },
    });
  }

  const addChipForField = (
    label: string,
    vals: string[] | undefined,
    singleVal: string | undefined,
    clearMulti: () => void,
    clearSingle: () => void
  ) => {
    if (vals && vals.length) {
      chips.push({
        label: `${label}: ${vals.join(", ")}`,
        clear: () => {
          clearMulti();
        },
      });
    } else if (singleVal) {
      chips.push({
        label: `${label}: ${singleVal}`,
        clear: () => {
          clearSingle();
        },
      });
    }
  };

  addChipForField(
    "类型",
    f.types,
    f.type,
    () => {
      f.types = undefined;
      f.typeLogic = undefined;
      emitChange();
    },
    () => {
      f.type = undefined;
      emitChange();
    }
  );

  addChipForField(
    "颜色",
    f.colors,
    f.color,
    () => {
      f.colors = undefined;
      f.colorLogic = undefined;
      emitChange();
    },
    () => {
      f.color = undefined;
      emitChange();
    }
  );

  addChipForField(
    "尺寸",
    f.sizes,
    f.size,
    () => {
      f.sizes = undefined;
      f.sizeLogic = undefined;
      emitChange();
    },
    () => {
      f.size = undefined;
      emitChange();
    }
  );

  if (f.location) {
    chips.push({
      label: `位置: ${f.location}`,
      clear: () => {
        f.location = undefined;
        emitChange();
      },
    });
  }

  if (f.quantityMin !== undefined || f.quantityMax !== undefined) {
    const label =
      f.quantityMin !== undefined && f.quantityMax !== undefined
        ? `数量: ${f.quantityMin} ~ ${f.quantityMax}`
        : f.quantityMin !== undefined
          ? `数量 ≥ ${f.quantityMin}`
          : `数量 ≤ ${f.quantityMax}`;
    chips.push({
      label,
      clear: () => {
        f.quantityMin = undefined;
        f.quantityMax = undefined;
        emitChange();
      },
    });
  }

  if (f.hasImage === true) {
    chips.push({
      label: "有图片",
      clear: () => {
        f.hasImage = null;
        emitChange();
      },
    });
  } else if (f.hasImage === false) {
    chips.push({
      label: "无图片",
      clear: () => {
        f.hasImage = null;
        emitChange();
      },
    });
  }

  const addTimeChip = (
    label: string,
    dyn: DynamicTimeOption | undefined,
    from: string | undefined,
    to: string | undefined,
    clear: () => void
  ) => {
    if (dyn) {
      const opt = dynamicTimeOptions.find((o) => o.value === dyn);
      chips.push({
        label: `${label}: ${opt?.label || dyn}`,
        clear,
      });
    } else if (from || to) {
      const range =
        from && to
          ? `${from.slice(0, 10)} ~ ${to.slice(0, 10)}`
          : from
            ? `≥ ${from.slice(0, 10)}`
            : `≤ ${to!.slice(0, 10)}`;
      chips.push({
        label: `${label}: ${range}`,
        clear,
      });
    }
  };

  addTimeChip(
    "创建时间",
    f.createdAtDynamic as DynamicTimeOption,
    f.createdAtFrom,
    f.createdAtTo,
    () => {
      f.createdAtDynamic = undefined;
      f.createdAtFrom = undefined;
      f.createdAtTo = undefined;
      emitChange();
    }
  );

  addTimeChip(
    "更新时间",
    f.updatedAtDynamic as DynamicTimeOption,
    f.updatedAtFrom,
    f.updatedAtTo,
    () => {
      f.updatedAtDynamic = undefined;
      f.updatedAtFrom = undefined;
      f.updatedAtTo = undefined;
      emitChange();
    }
  );

  if (f.advancedMode) {
    chips.push({
      label: "高级筛选模式",
      clear: () => {
        advancedMode.value = false;
        f.advancedConditions = undefined;
        emitChange();
      },
    });
  }

  return chips;
});

watch(
  [
    () => local.value.type,
    () => local.value.color,
    () => local.value.size,
    () => local.value.location,
    () => local.value.types,
    () => local.value.colors,
    () => local.value.sizes,
    () => local.value.quantityMin,
    () => local.value.quantityMax,
    () => local.value.hasImage,
    () => local.value.createdAtFrom,
    () => local.value.createdAtTo,
    () => local.value.createdAtDynamic,
    () => local.value.updatedAtFrom,
    () => local.value.updatedAtTo,
    () => local.value.updatedAtDynamic,
    () => local.value.typeLogic,
    () => local.value.colorLogic,
    () => local.value.sizeLogic,
  ],
  () => {
    emitChange();
  },
  { deep: true }
);
</script>

<template>
  <div class="filter-panel brick-card">
    <div class="quick-filter-row">
      <div class="filter-item flex-2">
        <el-input
          v-model="local.keyword"
          placeholder="搜索零件名称/编号..."
          class="brick-input"
          clearable
          @keyup.enter="onSearch"
        >
          <template #prefix>
            <el-icon><Search /></el-icon>
          </template>
        </el-input>
      </div>

      <div class="filter-item">
        <el-select
          v-if="!local.types"
          v-model="local.type"
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
        <el-select
          v-else
          v-model="local.types"
          placeholder="零件类型 (多选)"
          multiple
          collapse-tags
          collapse-tags-tooltip
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
        <div class="multi-toggle">
          <button
            class="link-btn"
            @click="useMultiType(!local.types)"
          >
            {{ local.types ? "单选模式" : "多选模式" }}
          </button>
          <el-select
            v-if="local.types"
            v-model="local.typeLogic"
            size="small"
            class="logic-select"
            placeholder="逻辑"
          >
            <el-option value="OR" label="或(OR)" />
            <el-option value="AND" label="且(AND)" />
          </el-select>
        </div>
      </div>

      <div class="filter-item">
        <el-select
          v-if="!local.colors"
          v-model="local.color"
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
        <el-select
          v-else
          v-model="local.colors"
          placeholder="颜色 (多选)"
          multiple
          collapse-tags
          collapse-tags-tooltip
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
        <div class="multi-toggle">
          <button
            class="link-btn"
            @click="useMultiColor(!local.colors)"
          >
            {{ local.colors ? "单选模式" : "多选模式" }}
          </button>
          <el-select
            v-if="local.colors"
            v-model="local.colorLogic"
            size="small"
            class="logic-select"
            placeholder="逻辑"
          >
            <el-option value="OR" label="或(OR)" />
            <el-option value="AND" label="且(AND)" />
          </el-select>
        </div>
      </div>

      <div class="filter-item">
        <el-select
          v-if="!local.sizes"
          v-model="local.size"
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
        <el-select
          v-else
          v-model="local.sizes"
          placeholder="尺寸 (多选)"
          multiple
          collapse-tags
          collapse-tags-tooltip
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
        <div class="multi-toggle">
          <button
            class="link-btn"
            @click="useMultiSize(!local.sizes)"
          >
            {{ local.sizes ? "单选模式" : "多选模式" }}
          </button>
          <el-select
            v-if="local.sizes"
            v-model="local.sizeLogic"
            size="small"
            class="logic-select"
            placeholder="逻辑"
          >
            <el-option value="OR" label="或(OR)" />
            <el-option value="AND" label="且(AND)" />
          </el-select>
        </div>
      </div>

      <div class="filter-item">
        <el-tree-select
          v-model="local.location"
          :data="locationTreeOptions"
          placeholder="存放位置"
          clearable
          check-strictly
          :render-after-expand="false"
          class="w-full"
        />
      </div>

      <div class="filter-item filter-actions">
        <button class="brick-btn brick-btn-sm" @click="onSearch">
          搜索
        </button>
        <button
          class="brick-btn brick-btn-sm brick-btn-secondary"
          @click="onReset"
        >
          <el-icon><Refresh /></el-icon>
          重置
        </button>
        <button
          class="brick-btn brick-btn-sm"
          :class="{ 'brick-btn-active': advancedMode }"
          @click="toggleAdvanced"
          title="高级筛选模式"
        >
          <el-icon><MagicStick /></el-icon>
          高级
        </button>
      </div>
    </div>

    <div class="expandable-sections">
      <div
        class="section-card"
        :class="{ expanded: expandedSection.has('quantity') }"
      >
        <button
          class="section-header"
          @click="toggleSection('quantity')"
        >
          <span class="section-title">
            📦 数量区间
            <span v-if="local.quantityMin !== undefined || local.quantityMax !== undefined" class="active-dot"></span>
          </span>
          <span class="expand-indicator">{{ expandedSection.has('quantity') ? '−' : '+' }}</span>
        </button>
        <div v-if="expandedSection.has('quantity')" class="section-body">
          <div class="range-inputs">
            <el-input-number
              v-model="local.quantityMin"
              :min="0"
              placeholder="最小值"
              controls-position="right"
              class="range-field"
            />
            <span class="range-separator">~</span>
            <el-input-number
              v-model="local.quantityMax"
              :min="0"
              placeholder="最大值"
              controls-position="right"
              class="range-field"
            />
          </div>
        </div>
      </div>

      <div
        class="section-card"
        :class="{ expanded: expandedSection.has('time') }"
      >
        <button
          class="section-header"
          @click="toggleSection('time')"
        >
          <span class="section-title">
            🕐 时间范围
            <span v-if="local.createdAtFrom || local.createdAtTo || local.createdAtDynamic || local.updatedAtFrom || local.updatedAtTo || local.updatedAtDynamic" class="active-dot"></span>
          </span>
          <span class="expand-indicator">{{ expandedSection.has('time') ? '−' : '+' }}</span>
        </button>
        <div v-if="expandedSection.has('time')" class="section-body">
          <div class="time-block">
            <label>创建时间</label>
            <div class="time-row">
              <el-select
                v-model="local.createdAtDynamic"
                placeholder="相对时间"
                clearable
                class="dyn-field"
              >
                <el-option
                  v-for="opt in dynamicTimeOptions"
                  :key="opt.value"
                  :label="opt.label"
                  :value="opt.value"
                />
              </el-select>
              <span class="time-or">或</span>
              <el-date-picker
                v-model="local.createdAtFrom"
                type="date"
                placeholder="开始日期"
                value-format="YYYY-MM-DDTHH:mm:ss"
                class="date-field"
                :disabled="!!local.createdAtDynamic"
              />
              <span class="range-separator">~</span>
              <el-date-picker
                v-model="local.createdAtTo"
                type="date"
                placeholder="结束日期"
                value-format="YYYY-MM-DDTHH:mm:ss"
                class="date-field"
                :disabled="!!local.createdAtDynamic"
              />
            </div>
          </div>
          <div class="time-block">
            <label>更新时间</label>
            <div class="time-row">
              <el-select
                v-model="local.updatedAtDynamic"
                placeholder="相对时间"
                clearable
                class="dyn-field"
              >
                <el-option
                  v-for="opt in dynamicTimeOptions"
                  :key="opt.value"
                  :label="opt.label"
                  :value="opt.value"
                />
              </el-select>
              <span class="time-or">或</span>
              <el-date-picker
                v-model="local.updatedAtFrom"
                type="date"
                placeholder="开始日期"
                value-format="YYYY-MM-DDTHH:mm:ss"
                class="date-field"
                :disabled="!!local.updatedAtDynamic"
              />
              <span class="range-separator">~</span>
              <el-date-picker
                v-model="local.updatedAtTo"
                type="date"
                placeholder="结束日期"
                value-format="YYYY-MM-DDTHH:mm:ss"
                class="date-field"
                :disabled="!!local.updatedAtDynamic"
              />
            </div>
          </div>
        </div>
      </div>

      <div
        class="section-card"
        :class="{ expanded: expandedSection.has('image') }"
      >
        <button
          class="section-header"
          @click="toggleSection('image')"
        >
          <span class="section-title">
            🖼️ 图片筛选
            <span v-if="local.hasImage === true || local.hasImage === false" class="active-dot"></span>
          </span>
          <span class="expand-indicator">{{ expandedSection.has('image') ? '−' : '+' }}</span>
        </button>
        <div v-if="expandedSection.has('image')" class="section-body">
          <div class="image-options">
            <button
              class="option-chip"
              :class="{ active: local.hasImage === true }"
              @click="local.hasImage = local.hasImage === true ? null : true"
            >
              <el-icon><PictureFilled /></el-icon>
              有图片
            </button>
            <button
              class="option-chip"
              :class="{ active: local.hasImage === false }"
              @click="local.hasImage = local.hasImage === false ? null : false"
            >
              <el-icon><Picture /></el-icon>
              无图片
            </button>
          </div>
        </div>
      </div>

      <div
        v-if="advancedMode"
        class="section-card expanded"
        style="border-color: $color-primary;"
      >
        <div class="section-header" style="cursor: default;">
          <span class="section-title" style="color: $color-primary;">
            ✨ 高级筛选
          </span>
        </div>
        <div class="section-body">
          <AdvancedFilterBuilder
            v-if="local.advancedConditions"
            v-model="local.advancedConditions"
            @update:modelValue="onAdvancedGroupChange"
          />
        </div>
      </div>
    </div>

    <div v-if="activeChips.length > 0" class="active-filters">
      <span class="chips-label">已应用筛选:</span>
      <div class="chips-wrap">
        <el-tag
          v-for="(chip, idx) in activeChips"
          :key="idx"
          closable
          size="small"
          effect="dark"
          type="warning"
          @close="chip.clear"
          class="filter-chip"
        >
          {{ chip.label }}
        </el-tag>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.filter-panel {
  padding: $spacing-lg;
  margin-bottom: $spacing-lg;
  display: flex;
  flex-direction: column;
  gap: $spacing-md;
}

.quick-filter-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr auto;
  gap: $spacing-md;
  align-items: start;
}

.filter-item {
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: $spacing-xs;

  &.flex-2 {
    min-width: 0;
  }
}

.w-full {
  width: 100%;
}

.multi-toggle {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: $spacing-sm;

  .link-btn {
    background: none;
    border: none;
    color: $color-primary;
    font-size: $font-size-xs;
    cursor: pointer;
    padding: 0;

    &:hover {
      text-decoration: underline;
    }
  }

  .logic-select {
    width: 90px;
  }
}

.filter-actions {
  display: flex;
  gap: $spacing-sm;
  flex-direction: row !important;
  align-items: center;
  padding-top: 2px;
}

.color-option {
  display: inline-flex;
  align-items: center;
  gap: $spacing-xs;
}

.color-dot {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 1px solid rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
}

.expandable-sections {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
}

.section-card {
  background: $color-dark-lighter;
  border: 2px solid $color-dark-border;
  border-radius: $brick-radius;
  overflow: hidden;
  transition: border-color $transition-fast;

  &.expanded {
    border-color: $color-dark-border;
  }
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-sm $spacing-md;
  background: transparent;
  border: none;
  width: 100%;
  color: $color-gray-light;
  cursor: pointer;
  transition: background $transition-fast;

  &:hover {
    background: rgba(255, 255, 255, 0.03);
  }
}

.section-title {
  font-size: $font-size-sm;
  font-weight: 500;
  display: inline-flex;
  align-items: center;
  gap: $spacing-xs;

  .active-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: $color-primary;
    display: inline-block;
  }
}

.expand-indicator {
  font-weight: 700;
  color: $color-gray-dark;
  font-size: $font-size-lg;
  line-height: 1;
}

.section-body {
  padding: $spacing-md;
  border-top: 1px solid $color-dark-border;
}

.range-inputs {
  display: flex;
  align-items: center;
  gap: $spacing-md;

  .range-field {
    flex: 1;
    max-width: 220px;
  }

  .range-separator {
    color: $color-gray-dark;
    font-weight: 600;
  }
}

.time-block {
  margin-bottom: $spacing-md;

  &:last-child {
    margin-bottom: 0;
  }

  > label {
    display: block;
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-bottom: $spacing-xs;
  }
}

.time-row {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  flex-wrap: wrap;

  .dyn-field {
    flex: 0 0 160px;
  }

  .date-field {
    flex: 1;
    min-width: 160px;
  }

  .time-or {
    color: $color-gray-dark;
    font-size: $font-size-sm;
  }

  .range-separator {
    color: $color-gray-dark;
    font-weight: 600;
  }
}

.image-options {
  display: flex;
  gap: $spacing-md;
}

.option-chip {
  display: inline-flex;
  align-items: center;
  gap: $spacing-xs;
  padding: $spacing-sm $spacing-lg;
  background: $color-dark-border;
  border: 2px solid $color-dark-border;
  border-radius: $brick-radius;
  color: $color-gray-light;
  font-size: $font-size-sm;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-gray-dark;
  }

  &.active {
    background: rgba(255, 214, 0, 0.15);
    border-color: $color-primary;
    color: $color-primary;
    font-weight: 500;
  }
}

.active-filters {
  display: flex;
  align-items: flex-start;
  gap: $spacing-sm;
  padding-top: $spacing-md;
  border-top: 1px solid $color-dark-border;
  flex-wrap: wrap;

  .chips-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    padding-top: 2px;
    flex-shrink: 0;
  }

  .chips-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: $spacing-xs;
  }
}

.filter-chip {
  :deep(.el-tag__content) {
    max-width: 300px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    display: inline-block;
  }
}

.brick-btn-active {
  background: $color-primary !important;
  color: $color-dark !important;
}

@media (max-width: 1200px) {
  .quick-filter-row {
    grid-template-columns: 1fr 1fr;
  }

  .filter-actions {
    grid-column: span 2;
  }
}
</style>
