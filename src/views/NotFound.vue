<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();

function goHome() {
  router.push("/");
}
</script>

<template>
  <div class="not-found">
    <div class="error-code">404</div>
    <div class="error-message">页面未找到</div>
    <div class="error-desc">你访问的页面不存在或已被移除</div>
    <button class="brick-btn" @click="goHome">
      返回首页
    </button>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.not-found {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 60vh;
  text-align: center;
  padding: $spacing-xl;
}

.error-code {
  font-size: 96px;
  font-weight: 800;
  color: $color-primary;
  line-height: 1;
  margin-bottom: $spacing-md;
  text-shadow: 0 4px 24px rgba($color-primary, 0.3);
}

.error-message {
  font-size: 24px;
  font-weight: 600;
  color: $color-white;
  margin-bottom: $spacing-sm;
}

.error-desc {
  font-size: $font-size-base;
  color: $color-gray-dark;
  margin-bottom: $spacing-lg;
}
</style>
