<script setup lang="ts">
import { RouterLink, RouterView, useRoute } from "vue-router";
import { computed, onMounted } from "vue";
import { useAppStore } from "@/stores";
import {
  Menu,
  Grid,
  List,
  Box,
  DataLine,
  Setting,
  Upload,
} from "@element-plus/icons-vue";

const appStore = useAppStore();
const route = useRoute();

const menuItems = [
  { path: "/", icon: Grid, label: "仪表板" },
  { path: "/parts", icon: Box, label: "零件管理" },
  { path: "/moc", icon: List, label: "MOC 清单" },
  { path: "/master", icon: Setting, label: "基础数据" },
  { path: "/import-export", icon: Upload, label: "导入导出" },
  { path: "/settings", icon: DataLine, label: "系统设置" },
];

const activeMenu = computed(() => route.path);

onMounted(async () => {
  await appStore.initializeApp();
});
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar" :class="{ collapsed: appStore.sidebarCollapsed }">
      <div class="logo">
        <div class="logo-studs">
          <div class="brick-stud-row">
            <span class="brick-stud"></span>
            <span class="brick-stud"></span>
          </div>
          <div class="brick-stud-row">
            <span class="brick-stud"></span>
            <span class="brick-stud"></span>
          </div>
        </div>
        <span v-if="!appStore.sidebarCollapsed" class="logo-text">
          乐高收纳
        </span>
      </div>

      <nav class="nav-menu">
        <RouterLink
          v-for="item in menuItems"
          :key="item.path"
          :to="item.path"
          class="nav-item"
          :class="{ active: activeMenu === item.path }"
        >
          <el-icon :size="20">
            <component :is="item.icon" />
          </el-icon>
          <span v-if="!appStore.sidebarCollapsed" class="nav-label">
            {{ item.label }}
          </span>
        </RouterLink>
      </nav>

      <div class="sidebar-footer">
        <button
          class="collapse-btn"
          @click="appStore.toggleSidebar()"
          title="收起/展开菜单"
        >
          <el-icon :size="18">
            <Menu />
          </el-icon>
        </button>
      </div>
    </aside>

    <main class="main-content">
      <RouterView v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </RouterView>
    </main>

    <transition name="fade">
      <div v-if="appStore.loading" class="loading-overlay">
        <div class="loading-content">
          <div class="loading-spinner"></div>
          <div class="loading-text">{{ appStore.loadingText }}</div>
        </div>
      </div>
    </transition>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.app-layout {
  display: flex;
  width: 100%;
  height: 100vh;
  background: $color-dark;
}

.sidebar {
  width: 220px;
  background: $color-dark-light;
  border-right: $brick-border solid $color-dark-border;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
  transition: width $transition-normal;

  &.collapsed {
    width: 64px;

    .logo-text,
    .nav-label {
      opacity: 0;
      width: 0;
      overflow: hidden;
    }

    .nav-item {
      justify-content: center;
      padding: $spacing-sm;
    }

    .logo {
      justify-content: center;
      padding: $spacing-md;
    }
  }
}

.logo {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-lg $spacing-md;
  border-bottom: $brick-border solid $color-dark-border;

  .logo-studs {
    flex-shrink: 0;
  }

  .logo-text {
    font-size: $font-size-lg;
    font-weight: 700;
    color: $color-white;
    white-space: nowrap;
    transition: opacity $transition-fast;
  }
}

.nav-menu {
  flex: 1;
  padding: $spacing-sm 0;
  overflow-y: auto;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-sm $spacing-md;
  margin: 2px $spacing-sm;
  color: $color-gray-light;
  text-decoration: none;
  border-radius: $brick-radius;
  transition: all $transition-fast;
  white-space: nowrap;

  &:hover {
    background: $color-dark-lighter;
    color: $color-primary;
  }

  &.active {
    background: $color-primary;
    color: $color-dark;
    font-weight: 600;
  }

  .nav-label {
    transition: opacity $transition-fast;
  }
}

.sidebar-footer {
  padding: $spacing-sm;
  border-top: $brick-border solid $color-dark-border;

  .collapse-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: $spacing-sm;
    background: $color-dark-lighter;
    border: none;
    border-radius: $brick-radius;
    color: $color-gray-light;
    cursor: pointer;
    transition: all $transition-fast;

    &:hover {
      background: $color-dark-border;
      color: $color-primary;
    }
  }
}

.main-content {
  flex: 1;
  overflow: hidden;
  background: $color-dark;
}
</style>
