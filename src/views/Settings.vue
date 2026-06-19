<script setup lang="ts">
import { ref } from "vue";
import { Refresh, Lock, InfoFilled, Coin, Warning } from "@element-plus/icons-vue";
import { useAppStore } from "@/stores";

const appStore = useAppStore();

const keyDialogVisible = ref(false);
const oldKey = ref("");
const newKey = ref("");
const confirmKey = ref("");

const appInfo = ref({
  version: "0.1.0",
  name: "乐高零件收纳管理器",
  engine: "Tauri 2.x",
  frontend: "Vue 3 + Element Plus",
  database: "SQLite + AES-256-GCM",
});

async function handleChangeKey() {
  if (!oldKey.value) {
    appStore.showError("请输入当前密钥");
    return;
  }
  if (!newKey.value || newKey.value.length < 8) {
    appStore.showError("新密钥长度至少 8 位");
    return;
  }
  if (newKey.value !== confirmKey.value) {
    appStore.showError("两次输入的新密钥不一致");
    return;
  }

  const success = await appStore.changeEncryptionKey(
    oldKey.value,
    newKey.value
  );
  if (success) {
    keyDialogVisible.value = false;
    oldKey.value = "";
    newKey.value = "";
    confirmKey.value = "";
  }
}

function generateRandomKey() {
  const chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
  let result = "";
  for (let i = 0; i < 32; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  newKey.value = result;
  confirmKey.value = result;
}
</script>

<template>
  <div class="page-container">
    <div class="page-header">
      <h1>
        <span class="brick-stud"></span>
        系统设置
      </h1>
    </div>

    <div class="page-content">
      <div class="settings-container">
        <div class="settings-section brick-card">
          <div class="section-header">
            <el-icon><Lock :size="20" /></el-icon>
            <h2>安全设置</h2>
          </div>
          <div class="section-body">
            <div class="setting-item">
            <div class="setting-info">
              <h3>数据加密</h3>
              <p>所有零件名称和描述使用 AES-256-GCM 加密存储</p>
            </div>
            <div class="setting-action">
              <el-tag type="success" effect="dark">已启用</el-tag>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>加密密钥</h3>
              <p>修改加密密钥后，所有数据将使用新密钥重新加密</p>
            </div>
            <div class="setting-action">
              <button
                class="brick-btn brick-btn-sm brick-btn-secondary"
                @click="keyDialogVisible = true"
              >
                <el-icon><Lock /></el-icon>
                修改密钥
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="settings-section brick-card">
        <div class="section-header">
          <el-icon><InfoFilled :size="20" /></el-icon>
          <h2>关于应用信息</h2>
        </div>
        <div class="section-body">
          <div class="info-grid">
            <div class="info-item">
              <span class="info-label">应用名称</span>
              <span class="info-value">{{ appInfo.name }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">版本号</span>
              <span class="info-value">v{{ appInfo.version }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">技术架构</span>
              <span class="info-value">{{ appInfo.engine }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">前端框架</span>
              <span class="info-value">{{ appInfo.frontend }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">数据存储</span>
              <span class="info-value">{{ appInfo.database }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">数据位置</span>
              <span class="info-value">本地存储</span>
            </div>
          </div>
        </div>
      </div>

      <div class="settings-section brick-card">
        <div class="section-header">
          <el-icon><Coin :size="20" /></el-icon>
          <h2>使用说明</h2>
        </div>
        <div class="section-body">
          <div class="help-content">
            <h3>快速上手</h3>
            <ol>
              <li>在「零件管理」页面点击「新增零件」添加你的乐高零件</li>
              <li>填写零件名称、编号、类型、颜色、尺寸和存放位置</li>
              <li>可以为每个零件上传实拍图片，方便识别</li>
              <li>在「MOC 清单」创建你的 MOC 零件清单</li>
              <li>点击「比对库存」查看哪些零件缺少</li>
              <li>使用「导入导出」批量管理零件数据</li>
            </ol>

            <h3>数据安全</h3>
            <ul>
              <li>所有数据存储在本地，不上传云端</li>
              <li>零件名称和描述使用 AES-256-GCM 加密</li>
              <li>请定期导出数据作为备份</li>
              <li>修改加密密钥前请确保记住旧密钥正确</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
    </div>

    <el-dialog
      v-model="keyDialogVisible"
      title="修改加密密钥"
      width="440px"
    >
      <el-form label-width="100px">
        <el-form-item label="当前密钥">
          <el-input
            v-model="oldKey"
            type="password"
            placeholder="请输入当前密钥"
            show-password
          />
        </el-form-item>
        <el-form-item label="新密钥">
          <el-input
          v-model="newKey"
          type="password"
          placeholder="请输入新密钥（至少8位）"
          show-password
          />
          <div class="key-actions">
          <button class="link-btn" @click="generateRandomKey">
            <el-icon><Refresh /></el-icon>
            生成随机密钥
          </button>
        </div>
        </el-form-item>
        <el-form-item label="确认密钥">
          <el-input
            v-model="confirmKey"
            type="password"
            placeholder="请再次输入新密钥"
            show-password
          />
        </el-form-item>
      </el-form>
      <div class="warning-text">
        <el-icon><Warning /></el-icon>
        请妥善保管密钥，丢失密钥将无法恢复数据！
      </div>
      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="keyDialogVisible = false"
        >
          取消
        </button>
        <button class="brick-btn" @click="handleChangeKey">
          确认修改
        </button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
@use "@/styles/variables.scss" as *;

.settings-container {
  display: flex;
  flex-direction: column;
  gap: $spacing-lg;
  max-width: 800px;
  margin: 0 auto;
}

.settings-section {
  padding: 0;
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  padding: $spacing-md $spacing-lg;
  background: $color-dark-lighter;
  border-bottom: 1px solid $color-dark-border;
  color: $color-primary;

  h2 {
    margin: 0;
    font-size: $font-size-base;
    color: $color-white;
    font-weight: 600;
  }
}

.section-body {
  padding: $spacing-lg;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-md 0;
  border-bottom: 1px solid $color-dark-border;

  &:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }

  &:first-child {
    padding-top: 0;
  }
}

.setting-info {
  flex: 1;

  h3 {
    margin: 0 0 $spacing-xs 0;
    font-size: $font-size-base;
    color: $color-white;
    font-weight: 500;
  }

  p {
    margin: 0;
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

.setting-action {
  flex-shrink: 0;
}

.info-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: $spacing-md;
}

.info-item {
  display: flex;
  flex-direction: column;
  gap: $spacing-xs;
  padding: $spacing-md;
  background: $color-dark;
  border-radius: $brick-radius;
  border: 1px solid $color-dark-border;

  .info-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }

  .info-value {
    font-size: $font-size-base;
    color: $color-white;
    font-weight: 500;
  }
}

.help-content {
  h3 {
    color: $color-white;
    margin: $spacing-md 0 $spacing-sm 0;
    font-size: $font-size-base;
    font-weight: 600;

    &:first-child {
      margin-top: 0;
    }
  }

  ol,
  ul {
    margin: 0;
    padding-left: $spacing-lg;
    color: $color-gray-light;
    font-size: $font-size-sm;
    line-height: 1.8;
  }

  li {
    margin-bottom: $spacing-xs;
  }
}

.key-actions {
  margin-top: $spacing-xs;
  text-align: right;
}

.link-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: none;
  border: none;
  color: $color-primary;
  font-size: $font-size-sm;
  cursor: pointer;
  padding: 0;
  cursor: pointer;

  &:hover {
    text-decoration: underline;
  }
}

.warning-text {
  display: flex;
  align-items: center;
  gap: $spacing-xs;
  padding: $spacing-sm $spacing-md;
  background: rgba(255, 152, 0, 0.1);
  color: $color-warning;
  font-size: $font-size-sm;
  border-radius: $brick-radius;
  margin-top: $spacing-md;
}
</style>
