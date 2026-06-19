<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  Box,
  List,
  Location,
  DataLine,
  Warning,
  TrendCharts,
  Setting,
  Calendar,
} from "@element-plus/icons-vue";
import { useStatsStore, usePartsStore, useMocStore, useMasterDataStore } from "@/stores";
import { useRouter } from "vue-router";
import type { TypeCount, ColorCount, LocationCount, MocStatusCount } from "@/types";
import { MOC_STATUS_OPTIONS } from "@/types";

const statsStore = useStatsStore();
const partsStore = usePartsStore();
const mocStore = useMocStore();
const masterDataStore = useMasterDataStore();
const router = useRouter();

const loading = ref(false);

async function loadData() {
  loading.value = true;
  try {
    await Promise.all([
      statsStore.loadStats(),
      partsStore.loadParts(),
      mocStore.loadMocLists(),
      masterDataStore.loadAll(),
    ]);
  } finally {
    loading.value = false;
  }
}

const statCards = computed(() => {
  if (!statsStore.stats) return [];
  return [
    {
      title: "零件种类",
      value: statsStore.stats.totalParts,
      icon: Box,
      color: "primary",
      path: "/parts",
    },
    {
      title: "零件总数",
      value: statsStore.stats.totalQuantity,
      icon: TrendCharts,
      color: "success",
      path: "/parts",
    },
    {
      title: "MOC 清单",
      value: statsStore.stats.totalMocs,
      icon: List,
      color: "info",
      path: "/moc",
    },
    {
      title: "存放位置",
      value: statsStore.stats.totalLocations,
      icon: Location,
      color: "warning",
      path: "/master",
    },
    {
      title: "低库存零件",
      value: statsStore.stats.lowStockParts,
      icon: Warning,
      color: "danger",
      path: "/parts",
    },
    {
      title: "MOC 缺件",
      value: statsStore.stats.missingPartsInMocs,
      icon: Setting,
      color: "danger",
      path: "/moc",
    },
    {
      title: "MOC 状态分布",
      value: `${statsStore.stats.mocsByStatus.length} 状态`,
      icon: Calendar,
      color: "success",
      path: "/moc",
    },
  ];
});

const partsByTypeData = computed(() => {
  if (!statsStore.stats) return [] as TypeCount[];
  return statsStore.stats.partsByType.slice(0, 6).filter((t: TypeCount) => t.count > 0);
});

const partsByColorData = computed(() => {
  if (!statsStore.stats) return [] as ColorCount[];
  return statsStore.stats.partsByColor.slice(0, 8).filter((c: ColorCount) => c.count > 0);
});

const partsByLocationData = computed(() => {
  if (!statsStore.stats) return [] as LocationCount[];
  return statsStore.stats.partsByLocation.slice(0, 5).filter((l: LocationCount) => l.count > 0);
});

const mocsByStatusData = computed(() => {
  if (!statsStore.stats) return [] as MocStatusCount[];
  return statsStore.stats.mocsByStatus.filter((m: MocStatusCount) => m.count > 0);
});

function getMaxTypeCount() {
  if (!partsByTypeData.value.length) return 1;
  return Math.max(...partsByTypeData.value.map((t: TypeCount) => t.count));
}

function getMaxLocationCount() {
  if (!partsByLocationData.value.length) return 1;
  return Math.max(...partsByLocationData.value.map((l: LocationCount) => l.count));
}

function getMaxMocStatusCount() {
  if (!mocsByStatusData.value.length) return 1;
  return Math.max(...mocsByStatusData.value.map((m: MocStatusCount) => m.count));
}

function getMocStatusLabel(status: string) {
  const option = MOC_STATUS_OPTIONS.find((opt) => opt.value === status);
  return option ? option.label : status;
}

function getMocStatusColor(status: string) {
  const option = MOC_STATUS_OPTIONS.find((opt) => opt.value === status);
  return option ? option.color : "#909399";
}

function goTo(path: string) {
  router.push(path);
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
        仪表板
      </h1>
      <div class="header-actions">
        <button class="brick-btn brick-btn-sm" @click="loadData">
          <el-icon><DataLine /></el-icon>
          刷新数据
        </button>
      </div>
    </div>

    <div class="page-content">
      <div class="stats-grid">
        <div
          v-for="card in statCards"
          :key="card.title"
          class="stat-card brick-card"
          @click="goTo(card.path)"
        >
          <div class="stat-icon" :class="card.color">
            <el-icon :size="28">
              <component :is="card.icon" />
            </el-icon>
          </div>
          <div class="stat-content">
            <div class="stat-value">{{ card.value }}</div>
            <div class="stat-label">{{ card.title }}</div>
          </div>
        </div>
      </div>

      <div class="charts-section">
        <div class="chart-card brick-card">
          <div class="chart-header">
            <h3>零件类型分布</h3>
            <span class="chart-subtitle">按类型统计零件种类数量</span>
          </div>
          <div class="chart-body">
            <div v-if="partsByTypeData.length === 0" class="empty-chart">
              暂无数据
            </div>
            <div v-else class="bar-chart">
              <div
                v-for="item in partsByTypeData"
                :key="item.name"
                class="bar-item"
              >
                <span class="bar-label">{{ item.name }}</span>
                <div class="bar-track">
                  <div
                    class="bar-fill"
                    :style="{
                      width: `${(item.count / getMaxTypeCount()) * 100}%`,
                    }"
                  ></div>
                </div>
                <span class="bar-value">{{ item.count }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="chart-card brick-card">
          <div class="chart-header">
            <h3>零件颜色分布</h3>
            <span class="chart-subtitle">按颜色统计零件种类数量</span>
          </div>
          <div class="chart-body">
            <div v-if="partsByColorData.length === 0" class="empty-chart">
              暂无数据
            </div>
            <div v-else class="color-chart">
              <div
                v-for="item in partsByColorData"
                :key="item.name"
                class="color-item"
              >
                <span
                  class="color-block"
                  :style="{ backgroundColor: item.hex }"
                ></span>
                <span class="color-name">{{ item.name }}</span>
                <span class="color-count">{{ item.count }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="chart-card brick-card">
          <div class="chart-header">
            <h3>存放位置分布</h3>
            <span class="chart-subtitle">按位置统计零件种类数量</span>
          </div>
          <div class="chart-body">
            <div v-if="partsByLocationData.length === 0" class="empty-chart">
              暂无数据
            </div>
            <div v-else class="bar-chart horizontal">
              <div
                v-for="item in partsByLocationData"
                :key="item.name"
                class="bar-item-h"
              >
                <span class="bar-label-h">{{ item.name }}</span>
                <div class="bar-track-h">
                  <div
                    class="bar-fill-h"
                    :style="{
                      width: `${(item.count / getMaxLocationCount()) * 100}%`,
                    }"
                  ></div>
                </div>
                <span class="bar-value-h">{{ item.count }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="chart-card brick-card">
          <div class="chart-header">
            <h3>MOC 状态分布</h3>
            <span class="chart-subtitle">按状态统计 MOC 清单数量</span>
          </div>
          <div class="chart-body">
            <div v-if="mocsByStatusData.length === 0" class="empty-chart">
              暂无数据
            </div>
            <div v-else class="bar-chart horizontal moc-status">
              <div
                v-for="item in mocsByStatusData"
                :key="item.status"
                class="bar-item-h"
              >
                <span class="bar-label-h">{{ getMocStatusLabel(item.status) }}</span>
                <div class="bar-track-h">
                  <div
                    class="bar-fill-h"
                    :style="{
                      width: `${(item.count / getMaxMocStatusCount()) * 100}%`,
                      background: `linear-gradient(90deg, ${getMocStatusColor(item.status)}, ${getMocStatusColor(item.status)}cc)`,
                    }"
                  ></div>
                </div>
                <span
                  class="bar-value-h"
                  :style="{ color: getMocStatusColor(item.status) }"
                >{{ item.count }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="quick-section">
        <div class="quick-card brick-card">
          <div class="quick-header">
            <h3>快速操作</h3>
          </div>
          <div class="quick-body">
            <button class="quick-btn" @click="goTo('/parts')">
              <el-icon><Box /></el-icon>
              <span>添加新零件</span>
            </button>
            <button class="quick-btn" @click="goTo('/moc')">
              <el-icon><List /></el-icon>
              <span>创建 MOC 清单</span>
            </button>
            <button class="quick-btn" @click="goTo('/import-export')">
              <el-icon><DataLine /></el-icon>
              <span>导入导出</span>
            </button>
            <button class="quick-btn" @click="goTo('/settings')">
              <el-icon><Setting /></el-icon>
              <span>系统设置</span>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;
@use "sass:color";

.stats-grid {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  gap: $spacing-lg;
  margin-bottom: $spacing-lg;
}

.stat-card {
  display: flex;
  align-items: center;
  gap: $spacing-md;
  padding: $spacing-lg;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    transform: translateY(-4px);
  }
}

.stat-icon {
  width: 56px;
  height: 56px;
  border-radius: $brick-radius;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;

  &.primary {
    background: rgba(255, 214, 0, 0.2);
    color: $color-primary;
  }

  &.success {
    background: rgba(76, 175, 80, 0.2);
    color: $color-success;
  }

  &.warning {
    background: rgba(255, 152, 0, 0.2);
    color: $color-warning;
  }

  &.danger {
    background: rgba(244, 67, 54, 0.2);
    color: $color-danger;
  }

  &.info {
    background: rgba(33, 150, 243, 0.2);
    color: $color-info;
  }
}

.stat-content {
  flex: 1;
  min-width: 0;

  .stat-value {
    font-size: 28px;
    font-weight: 700;
    color: $color-white;
    line-height: 1.2;
  }

  .stat-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-top: $spacing-xs;
  }
}

.charts-section {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: $spacing-lg;
  margin-bottom: $spacing-lg;
}

.chart-card {
  display: flex;
  flex-direction: column;
}

.chart-header {
  padding: $spacing-md $spacing-lg;
  border-bottom: 1px solid $color-dark-border;

  h3 {
    font-size: $font-size-base;
    font-weight: 600;
    color: $color-white;
    margin: 0;
  }

  .chart-subtitle {
    font-size: $font-size-sm;
    color: $color-gray-dark;
    margin-top: $spacing-xs;
  }
}

.chart-body {
  padding: $spacing-lg;
  flex: 1;
}

.empty-chart {
  text-align: center;
  color: $color-gray-dark;
  padding: $spacing-xl 0;
}

.bar-chart {
  display: flex;
  flex-direction: column;
  gap: $spacing-md;
}

.bar-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
}

.bar-label {
  width: 80px;
  font-size: $font-size-sm;
  color: $color-gray-light;
  flex-shrink: 0;
}

.bar-track {
  flex: 1;
  height: 20px;
  background: $color-dark;
  border-radius: $brick-radius;
  overflow: hidden;
}

.bar-fill {
  height: 100%;
  background: linear-gradient(90deg, $color-primary, $color-primary-dark);
  border-radius: $brick-radius;
  transition: width $transition-normal;
  min-width: 4px;
}

.bar-value {
  width: 40px;
  text-align: right;
  font-size: $font-size-sm;
  color: $color-primary;
  font-weight: 600;
  flex-shrink: 0;
}

.color-chart {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: $spacing-sm;
}

.color-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm;
  background: $color-dark;
  border-radius: $brick-radius;
}

.color-block {
  width: 20px;
  height: 20px;
  border-radius: $brick-radius;
  border: 2px solid $color-dark-border;
  flex-shrink: 0;
}

.color-name {
  flex: 1;
  font-size: $font-size-sm;
  color: $color-gray-light;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.color-count {
  font-size: $font-size-sm;
  color: $color-primary;
  font-weight: 600;
  flex-shrink: 0;
}

.bar-chart.horizontal {
  .bar-item-h {
    display: flex;
    align-items: center;
    gap: $spacing-sm;
  }

  .bar-label-h {
    width: 80px;
    font-size: $font-size-sm;
    color: $color-gray-light;
    flex-shrink: 0;
  }

  .bar-track-h {
    flex: 1;
    height: 24px;
    background: $color-dark;
    border-radius: $brick-radius;
    overflow: hidden;
  }

  .bar-fill-h {
    height: 100%;
    background: linear-gradient(90deg, $color-info, color.adjust($color-info, $lightness: -15%));
    border-radius: $brick-radius;
    transition: width $transition-normal;
    min-width: 4px;
  }

  .bar-value-h {
    width: 40px;
    text-align: right;
    font-size: $font-size-sm;
    color: $color-info;
    font-weight: 600;
    flex-shrink: 0;
  }
}

.quick-section {
  .quick-card {
    padding: 0;
  }
}

.quick-header {
  padding: $spacing-md $spacing-lg;
  border-bottom: 1px solid $color-dark-border;

  h3 {
    font-size: $font-size-base;
    font-weight: 600;
    color: $color-white;
    margin: 0;
  }
}

.quick-body {
  padding: $spacing-lg;
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: $spacing-md;
}

.quick-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-lg $spacing-md;
  background: $color-dark;
  border: 2px solid $color-dark-border;
  border-radius: $brick-radius;
  color: $color-gray-light;
  cursor: pointer;
  transition: all $transition-fast;

  :deep(.el-icon) {
    font-size: 32px;
    color: $color-primary;
  }

  span {
    font-size: $font-size-sm;
    font-weight: 500;
  }

  &:hover {
    background: $color-dark-lighter;
    border-color: $color-primary;
    transform: translateY(-2px);
    color: $color-white;
  }
}

@media (max-width: 1400px) {
  .stats-grid {
    grid-template-columns: repeat(4, 1fr);
  }

  .charts-section {
    grid-template-columns: 1fr 1fr;
  }
}

@media (max-width: 900px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .charts-section {
    grid-template-columns: 1fr;
  }

  .quick-body {
    grid-template-columns: repeat(2, 1fr);
  }
}
</style>
