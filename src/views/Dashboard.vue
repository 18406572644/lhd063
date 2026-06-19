<script setup lang="ts">
import { ref, onMounted, computed, watch, nextTick } from "vue";
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
import { useApiRequest } from "@/composables";
import { useRouter } from "vue-router";
import type { TypeCount, ColorCount, LocationCount, MocStatusCount, LocationTreeNode } from "@/types";
import { MOC_STATUS_OPTIONS } from "@/types";

const statsStore = useStatsStore();
const partsStore = usePartsStore();
const mocStore = useMocStore();
const masterDataStore = useMasterDataStore();
const router = useRouter();

const { execute } = useApiRequest();
const sunburstCanvas = ref<HTMLCanvasElement | null>(null);
const sunburstTooltip = ref({ visible: false, x: 0, y: 0, name: "", count: 0, total: 0 });

async function loadData() {
  await execute(() =>
    Promise.all([
      statsStore.loadStats(),
      partsStore.loadParts(),
      mocStore.loadMocLists(),
      masterDataStore.loadAll(),
    ]).then(() => ({ success: true, data: undefined as void }))
  );
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
  return statsStore.stats.partsByLocation.filter((l: LocationCount) => l.count > 0);
});

const mocsByStatusData = computed(() => {
  if (!statsStore.stats) return [] as MocStatusCount[];
  return statsStore.stats.mocsByStatus.filter((m: MocStatusCount) => m.count > 0);
});

interface SunburstSegment {
  name: string;
  code: string;
  count: number;
  totalWithChildren: number;
  startAngle: number;
  endAngle: number;
  innerRadius: number;
  outerRadius: number;
  depth: number;
  color: string;
  children: SunburstSegment[];
}

const locationSunburstData = computed(() => {
  const locationTree = masterDataStore.buildLocationTree();
  const flatStats = partsByLocationData.value;
  const locationMap = new Map(flatStats.map((l) => [l.name, l.count]));

  function getSelfCount(node: LocationTreeNode): number {
    return locationMap.get(node.name) || 0;
  }

  function getTotalCount(node: LocationTreeNode): number {
    let total = getSelfCount(node);
    if (node.children) {
      for (const child of node.children) {
        total += getTotalCount(child);
      }
    }
    return total;
  }

  const palette = [
    "#FFD600", "#FF9800", "#2196F3", "#4CAF50", "#E91E63",
    "#9C27B0", "#00BCD4", "#FF5722", "#607D8B", "#8BC34A",
    "#FFEB3B", "#03A9F4", "#CDDC39", "#FFC107", "#795548",
  ];

  function buildSegments(
    nodes: LocationTreeNode[],
    startAngle: number,
    endAngle: number,
    innerRadius: number,
    outerRadius: number,
    depth: number,
    colorIndex: number
  ): SunburstSegment[] {
    const totalAll = nodes.reduce((sum, n) => sum + getTotalCount(n), 0);
    if (totalAll === 0) return [];

    let currentAngle = startAngle;
    const segments: SunburstSegment[] = [];

    for (const node of nodes) {
      const total = getTotalCount(node);
      if (total === 0) continue;
      const sweep = ((endAngle - startAngle) * total) / totalAll;
      const segColor = depth === 0
        ? palette[colorIndex % palette.length]
        : adjustColor(palette[colorIndex % palette.length], depth);

      const segment: SunburstSegment = {
        name: node.name,
        code: node.code,
        count: getSelfCount(node),
        totalWithChildren: total,
        startAngle: currentAngle,
        endAngle: currentAngle + sweep,
        innerRadius,
        outerRadius,
        depth,
        color: segColor,
        children: [],
      };

      if (node.children?.length) {
        segment.children = buildSegments(
          node.children,
          currentAngle,
          currentAngle + sweep,
          outerRadius,
          outerRadius + 40,
          depth + 1,
          colorIndex
        );
      }

      segments.push(segment);
      currentAngle += sweep;
      colorIndex++;
    }

    return segments;
  }

  const r0 = 50;
  return buildSegments(locationTree, -Math.PI / 2, Math.PI * 1.5, r0, r0 + 55, 0, 0);
});

function adjustColor(hex: string, depth: number): string {
  const factor = 1 - depth * 0.15;
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  const nr = Math.round(Math.min(255, r * factor));
  const ng = Math.round(Math.min(255, g * factor));
  const nb = Math.round(Math.min(255, b * factor));
  return `rgb(${nr},${ng},${nb})`;
}

function flattenSegments(segments: SunburstSegment[]): SunburstSegment[] {
  const result: SunburstSegment[] = [];
  for (const seg of segments) {
    result.push(seg);
    if (seg.children.length) {
      result.push(...flattenSegments(seg.children));
    }
  }
  return result;
}

function drawSunburst() {
  const canvas = sunburstCanvas.value;
  if (!canvas) return;
  const allSegments = flattenSegments(locationSunburstData.value);
  if (allSegments.length === 0) return;

  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const displayWidth = 400;
  const displayHeight = 400;
  canvas.width = displayWidth * dpr;
  canvas.height = displayHeight * dpr;
  canvas.style.width = displayWidth + "px";
  canvas.style.height = displayHeight + "px";
  ctx.scale(dpr, dpr);

  ctx.clearRect(0, 0, displayWidth, displayHeight);

  const cx = displayWidth / 2;
  const cy = displayHeight / 2;

  for (const seg of allSegments) {
    ctx.beginPath();
    ctx.arc(cx, cy, seg.outerRadius, seg.startAngle, seg.endAngle);
    ctx.arc(cx, cy, seg.innerRadius, seg.endAngle, seg.startAngle, true);
    ctx.closePath();
    ctx.fillStyle = seg.color;
    ctx.fill();
    ctx.strokeStyle = "#2A2A2A";
    ctx.lineWidth = 2;
    ctx.stroke();

    if (seg.endAngle - seg.startAngle > 0.2) {
      const midAngle = (seg.startAngle + seg.endAngle) / 2;
      const labelRadius = (seg.innerRadius + seg.outerRadius) / 2;
      const lx = cx + Math.cos(midAngle) * labelRadius;
      const ly = cy + Math.sin(midAngle) * labelRadius;

      ctx.save();
      ctx.translate(lx, ly);
      let angle = midAngle;
      if (angle > Math.PI / 2 && angle < Math.PI * 1.5) {
        angle += Math.PI;
      }
      ctx.rotate(angle);
      ctx.fillStyle = "#2A2A2A";
      ctx.font = "11px 'Segoe UI', 'Microsoft YaHei', sans-serif";
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      const labelText =
        seg.name.length > 5 ? seg.name.slice(0, 4) + "…" : seg.name;
      ctx.fillText(labelText, 0, 0);
      ctx.restore();
    }
  }

  ctx.beginPath();
  ctx.arc(cx, cy, 48, 0, Math.PI * 2);
  ctx.fillStyle = "#2A2A2A";
  ctx.fill();
  ctx.strokeStyle = "#5A5A5A";
  ctx.lineWidth = 2;
  ctx.stroke();

  ctx.fillStyle = "#FFD600";
  ctx.font = "bold 20px 'Segoe UI', 'Microsoft YaHei', sans-serif";
  ctx.textAlign = "center";
  ctx.textBaseline = "middle";
  const totalCount = allSegments
    .filter((s) => s.depth === 0)
    .reduce((sum, s) => sum + s.totalWithChildren, 0);
  ctx.fillText(String(totalCount), cx, cy - 6);
  ctx.fillStyle = "#9E9E9E";
  ctx.font = "10px 'Segoe UI', 'Microsoft YaHei', sans-serif";
  ctx.fillText("零件种类", cx, cy + 12);
}

function handleSunburstMove(e: MouseEvent) {
  const canvas = sunburstCanvas.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const cx = 200;
  const cy = 200;
  const dx = x - cx;
  const dy = y - cy;
  const dist = Math.sqrt(dx * dx + dy * dy);
  let angle = Math.atan2(dy, dx);

  const allSegments = flattenSegments(locationSunburstData.value);
  let found: SunburstSegment | null = null;
  for (const seg of allSegments) {
    if (dist >= seg.innerRadius && dist <= seg.outerRadius) {
      let a = angle;
      if (a < seg.startAngle) a += Math.PI * 2;
      if (a >= seg.startAngle && a <= seg.endAngle) {
        found = seg;
        break;
      }
    }
  }

  if (found) {
    sunburstTooltip.value = {
      visible: true,
      x: e.clientX - rect.left,
      y: e.clientY - rect.top - 40,
      name: found.name,
      count: found.count,
      total: found.totalWithChildren,
    };
    canvas.style.cursor = "pointer";
  } else {
    sunburstTooltip.value.visible = false;
    canvas.style.cursor = "default";
  }
}

function handleSunburstClick(e: MouseEvent) {
  const canvas = sunburstCanvas.value;
  if (!canvas) return;
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  const cx = 200;
  const cy = 200;
  const dx = x - cx;
  const dy = y - cy;
  const dist = Math.sqrt(dx * dx + dy * dy);
  let angle = Math.atan2(dy, dx);

  const allSegments = flattenSegments(locationSunburstData.value);
  for (const seg of allSegments) {
    if (dist >= seg.innerRadius && dist <= seg.outerRadius) {
      let a = angle;
      if (a < seg.startAngle) a += Math.PI * 2;
      if (a >= seg.startAngle && a <= seg.endAngle) {
        goToLocationParts(seg.code);
        return;
      }
    }
  }
}

function handleSunburstLeave() {
  sunburstTooltip.value.visible = false;
}

function goToLocationParts(code: string) {
  partsStore.setFilter({ location: code });
  router.push("/parts");
}

function getMaxTypeCount() {
  if (!partsByTypeData.value.length) return 1;
  return Math.max(...partsByTypeData.value.map((t: TypeCount) => t.count));
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

watch(locationSunburstData, () => {
  nextTick(() => drawSunburst());
});

onMounted(() => {
  loadData().then(() => {
    nextTick(() => drawSunburst());
  });
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

        <div class="chart-card brick-card sunburst-card">
          <div class="chart-header">
            <h3>位置层级分布</h3>
            <span class="chart-subtitle">旭日图展示层级库存占比（点击可跳转）</span>
          </div>
          <div class="chart-body sunburst-body">
            <div v-if="partsByLocationData.length === 0" class="empty-chart">
              暂无数据
            </div>
            <div v-else class="sunburst-wrapper">
              <canvas
                ref="sunburstCanvas"
                @mousemove="handleSunburstMove"
                @click="handleSunburstClick"
                @mouseleave="handleSunburstLeave"
              ></canvas>
              <div
                v-if="sunburstTooltip.visible"
                class="sunburst-tooltip"
                :style="{
                  left: sunburstTooltip.x + 'px',
                  top: sunburstTooltip.y + 'px',
                }"
              >
                <div class="tooltip-name">{{ sunburstTooltip.name }}</div>
                <div class="tooltip-detail">
                  直接: {{ sunburstTooltip.count }} | 含子级: {{ sunburstTooltip.total }}
                </div>
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

      <div class="location-cards-section" v-if="partsByLocationData.length > 0">
        <div class="section-card brick-card">
          <div class="chart-header">
            <h3>位置快捷导航</h3>
            <span class="chart-subtitle">点击位置卡片查看该位置（含子位置）的零件</span>
          </div>
          <div class="location-cards-grid">
            <div
              v-for="loc in masterDataStore.buildLocationTree()"
              :key="loc.code"
              class="location-nav-card"
              @click="goToLocationParts(loc.code)"
            >
              <div class="location-nav-icon">
                <el-icon :size="24"><Location /></el-icon>
              </div>
              <div class="location-nav-info">
                <div class="location-nav-name">{{ loc.name }}</div>
                <div class="location-nav-code">{{ loc.code }}</div>
              </div>
              <div v-if="loc.children?.length" class="location-nav-children">
                <div
                  v-for="child in loc.children"
                  :key="child.code"
                  class="location-nav-child"
                  @click.stop="goToLocationParts(child.code)"
                >
                  <el-icon :size="14"><Location /></el-icon>
                  {{ child.name }}
                </div>
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

.sunburst-body {
  display: flex;
  justify-content: center;
  align-items: center;
  position: relative;
}

.sunburst-wrapper {
  position: relative;
  display: inline-block;
}

.sunburst-tooltip {
  position: absolute;
  background: $color-dark-lighter;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  padding: $spacing-sm $spacing-md;
  pointer-events: none;
  z-index: 100;

  .tooltip-name {
    font-weight: 600;
    color: $color-white;
    font-size: $font-size-sm;
  }

  .tooltip-detail {
    font-size: $font-size-xs;
    color: $color-gray-dark;
    margin-top: 2px;
  }
}

.location-cards-section {
  margin-bottom: $spacing-lg;
}

.section-card {
  padding: 0;
}

.location-cards-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: $spacing-md;
  padding: $spacing-lg;
}

.location-nav-card {
  display: flex;
  flex-direction: column;
  gap: $spacing-sm;
  padding: $spacing-md;
  background: $color-dark;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    border-color: $color-primary;
    transform: translateY(-2px);
  }
}

.location-nav-icon {
  width: 36px;
  height: 36px;
  border-radius: $brick-radius;
  background: rgba($color-warning, 0.2);
  color: $color-warning;
  display: flex;
  align-items: center;
  justify-content: center;
}

.location-nav-info {
  .location-nav-name {
    font-weight: 600;
    color: $color-white;
    font-size: $font-size-base;
  }

  .location-nav-code {
    font-size: $font-size-sm;
    color: $color-primary;
    font-family: monospace;
  }
}

.location-nav-children {
  display: flex;
  flex-wrap: wrap;
  gap: $spacing-xs;
  margin-top: $spacing-xs;
  padding-top: $spacing-xs;
  border-top: 1px solid $color-dark-border;
}

.location-nav-child {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: $font-size-xs;
  color: $color-gray-light;
  background: $color-dark-lighter;
  padding: 2px 8px;
  border-radius: 10px;
  cursor: pointer;
  transition: all $transition-fast;

  &:hover {
    background: $color-primary;
    color: $color-dark;
  }
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

  .location-cards-grid {
    grid-template-columns: 1fr;
  }
}
</style>
