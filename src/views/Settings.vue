<script setup lang="ts">
import { ref, onMounted } from "vue";
import { Refresh, Lock, InfoFilled, Coin, Warning, List, View, Download, Upload, Delete, CircleCheck, CircleClose } from "@element-plus/icons-vue";
import { useAppStore } from "@/stores";
import type { OperationLog, OperationLogFilter, BackupInfo, BackupConfig, IntegrityCheckResult } from "@/types";
import { OPERATION_TYPE_OPTIONS, OBJECT_TYPE_OPTIONS } from "@/types";
import { api } from "@/api";

const appStore = useAppStore();

const keyDialogVisible = ref(false);
const oldKey = ref("");
const newKey = ref("");
const confirmKey = ref("");

const logsDialogVisible = ref(false);
const operationLogs = ref<OperationLog[]>([]);
const logsLoading = ref(false);
const filterOperationType = ref("");
const filterObjectType = ref("");
const logDetailDialogVisible = ref(false);
const currentLog = ref<OperationLog | null>(null);

const appInfo = ref({
  version: "0.1.0",
  name: "乐高零件收纳管理器",
  engine: "Tauri 2.x",
  frontend: "Vue 3 + Element Plus",
  database: "SQLite + AES-256-GCM",
});

const backups = ref<BackupInfo[]>([]);
const backupsLoading = ref(false);
const backupConfig = ref<BackupConfig>({ enabled: false, frequency: "daily", keepCount: 5, encrypt: false });
const createBackupLoading = ref(false);
const backupPassword = ref("");
const backupEncryptEnabled = ref(false);

const restoreDialogVisible = ref(false);
const restoreTarget = ref<BackupInfo | null>(null);
const restoreMode = ref<"full" | "merge">("full");
const restorePassword = ref("");
const restoreLoading = ref(false);

const integrityResult = ref<IntegrityCheckResult | null>(null);
const integrityLoading = ref(false);
const integrityDialogVisible = ref(false);

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

async function openLogsDialog() {
  logsDialogVisible.value = true;
  await loadOperationLogs();
}

async function loadOperationLogs() {
  logsLoading.value = true;
  try {
    const filter: OperationLogFilter = {};
    if (filterOperationType.value) filter.operationType = filterOperationType.value;
    if (filterObjectType.value) filter.objectType = filterObjectType.value;
    operationLogs.value = await api.getOperationLogs(filter);
  } finally {
    logsLoading.value = false;
  }
}

function handleFilterReset() {
  filterOperationType.value = "";
  filterObjectType.value = "";
  loadOperationLogs();
}

function getOperationTypeLabel(type: string) {
  const opt = OPERATION_TYPE_OPTIONS.find((o) => o.value === type);
  return opt?.label || type;
}

function getOperationTypeTagType(type: string) {
  const opt = OPERATION_TYPE_OPTIONS.find((o) => o.value === type);
  return opt?.type || "info";
}

function getObjectTypeLabel(type: string) {
  const opt = OBJECT_TYPE_OPTIONS.find((o) => o.value === type);
  return opt?.label || type;
}

function formatDate(dateStr: string) {
  try {
    return new Date(dateStr).toLocaleString("zh-CN");
  } catch {
    return dateStr;
  }
}

function parseSnapshot(snapshot?: string) {
  if (!snapshot) return null;
  try {
    return JSON.parse(snapshot);
  } catch {
    return null;
  }
}

function viewLogDetail(log: OperationLog) {
  currentLog.value = log;
  logDetailDialogVisible.value = true;
}

function formatSnapshot(snapshot?: string) {
  const data = parseSnapshot(snapshot);
  if (!data) return "-";
  return JSON.stringify(data, null, 2);
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
}

async function loadBackups() {
  backupsLoading.value = true;
  try {
    backups.value = await api.listBackups();
  } finally {
    backupsLoading.value = false;
  }
}

async function loadBackupConfig() {
  try {
    backupConfig.value = await api.getBackupConfig();
    backupEncryptEnabled.value = backupConfig.value.encrypt;
  } catch {
    // keep defaults
  }
}

async function handleCreateBackup() {
  createBackupLoading.value = true;
  try {
    const password = backupEncryptEnabled.value && backupPassword.value ? backupPassword.value : undefined;
    const info = await api.createBackup(password);
    appStore.showSuccess(`备份创建成功：${info.filename}`);
    backupPassword.value = "";
    await loadBackups();
  } catch (e: any) {
    appStore.showError(`备份创建失败：${e?.toString() || "未知错误"}`);
  } finally {
    createBackupLoading.value = false;
  }
}

async function handleDeleteBackup(backup: BackupInfo) {
  const confirmed = await appStore.showConfirm(
    `确定要删除备份「${backup.filename}」吗？此操作不可恢复。`
  );
  if (!confirmed) return;
  try {
    await api.deleteBackup(backup.filename);
    appStore.showSuccess("备份已删除");
    await loadBackups();
  } catch (e: any) {
    appStore.showError(`删除失败：${e?.toString() || "未知错误"}`);
  }
}

function openRestoreDialog(backup: BackupInfo) {
  restoreTarget.value = backup;
  restoreMode.value = "full";
  restorePassword.value = "";
  restoreDialogVisible.value = true;
}

async function handleRestore() {
  if (!restoreTarget.value) return;

  if (restoreMode.value === "full") {
    const confirmed = await appStore.showConfirm(
      "全量覆盖将替换当前所有数据（数据库、图片、密钥），此操作不可恢复！确定继续吗？"
    );
    if (!confirmed) return;
  }

  restoreLoading.value = true;
  try {
    const password = restoreTarget.value.encrypted && restorePassword.value ? restorePassword.value : undefined;
    const result = await api.restoreBackup(
      restoreTarget.value.filename,
      password,
      restoreMode.value
    );
    if (result.success) {
      appStore.showSuccess(result.message);
      restoreDialogVisible.value = false;
    } else {
      appStore.showError(result.message);
    }
  } catch (e: any) {
    appStore.showError(`恢复失败：${e?.toString() || "未知错误"}`);
  } finally {
    restoreLoading.value = false;
  }
}

async function handleBackupConfigChange() {
  try {
    const config: BackupConfig = {
      ...backupConfig.value,
      encrypt: backupEncryptEnabled.value,
    };
    await api.updateBackupConfig(config);
    backupConfig.value = config;
    appStore.showSuccess("备份配置已保存");
  } catch (e: any) {
    appStore.showError(`配置保存失败：${e?.toString() || "未知错误"}`);
  }
}

async function handleCheckIntegrity() {
  integrityLoading.value = true;
  try {
    integrityResult.value = await api.checkDatabaseIntegrity();
    integrityDialogVisible.value = true;
  } catch (e: any) {
    appStore.showError(`完整性检查失败：${e?.toString() || "未知错误"}`);
  } finally {
    integrityLoading.value = false;
  }
}

async function handleAutoRecover() {
  if (!integrityResult.value?.latestBackup) return;
  const confirmed = await appStore.showConfirm(
    "将使用最近的备份进行全量恢复，当前数据将被覆盖。确定继续吗？"
  );
  if (!confirmed) return;

  try {
    const result = await api.restoreBackup(
      integrityResult.value.latestBackup.filename,
      undefined,
      "full"
    );
    if (result.success) {
      appStore.showSuccess("数据已从备份恢复");
      integrityDialogVisible.value = false;
    } else {
      appStore.showError(result.message);
    }
  } catch (e: any) {
    appStore.showError(`恢复失败：${e?.toString() || "未知错误"}`);
  }
}

onMounted(async () => {
  await Promise.all([loadBackups(), loadBackupConfig()]);
});
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
          <el-icon><Download :size="20" /></el-icon>
          <h2>备份中心</h2>
        </div>
        <div class="section-body">
          <div class="setting-item">
            <div class="setting-info">
              <h3>手动备份</h3>
              <p>一键导出完整备份包（lego-backup-YYYYMMDD.lpk），包含数据库 + 图片目录 + 密钥</p>
            </div>
            <div class="setting-action backup-action-row">
              <label class="backup-encrypt-toggle">
                <el-switch v-model="backupEncryptEnabled" active-text="加密" inactive-text="" size="small" />
              </label>
              <el-input
                v-if="backupEncryptEnabled"
                v-model="backupPassword"
                type="password"
                placeholder="设置密码"
                show-password
                size="small"
                style="width: 140px"
              />
              <button
                class="brick-btn brick-btn-sm"
                :disabled="createBackupLoading || (backupEncryptEnabled && !backupPassword)"
                @click="handleCreateBackup"
              >
                <el-icon><Download /></el-icon>
                {{ createBackupLoading ? "备份中..." : "立即备份" }}
              </button>
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>自动备份</h3>
              <p>按计划自动创建备份，保留最近 N 个版本</p>
            </div>
            <div class="setting-action">
              <el-switch v-model="backupConfig.enabled" size="small" @change="handleBackupConfigChange" />
            </div>
          </div>

          <div v-if="backupConfig.enabled" class="backup-config-row">
            <div class="config-field">
              <span class="config-label">频率</span>
              <el-select v-model="backupConfig.frequency" size="small" style="width: 120px" @change="handleBackupConfigChange">
                <el-option label="每日" value="daily" />
                <el-option label="每周" value="weekly" />
              </el-select>
            </div>
            <div class="config-field">
              <span class="config-label">保留数量</span>
              <el-input-number v-model="backupConfig.keepCount" :min="1" :max="30" size="small" style="width: 110px" @change="handleBackupConfigChange" />
            </div>
            <div class="config-field">
              <span class="config-label">自动加密</span>
              <el-switch v-model="backupEncryptEnabled" size="small" @change="handleBackupConfigChange" />
            </div>
          </div>

          <div class="setting-item">
            <div class="setting-info">
              <h3>异常检测</h3>
              <p>校验数据库完整性，发现损坏自动提示从最近备份恢复</p>
            </div>
            <div class="setting-action">
              <button
                class="brick-btn brick-btn-sm brick-btn-secondary"
                :loading="integrityLoading"
                @click="handleCheckIntegrity"
              >
                <el-icon><CircleCheck /></el-icon>
                完整性检查
              </button>
            </div>
          </div>

          <div class="backups-list" v-loading="backupsLoading">
            <div v-if="backups.length === 0" class="empty-backups">
              <p>暂无备份</p>
            </div>
            <div v-for="backup in backups" :key="backup.filename" class="backup-item">
              <div class="backup-info">
                <div class="backup-name">
                  <el-icon><Download :size="14" /></el-icon>
                  <span class="backup-filename">{{ backup.filename }}</span>
                  <el-tag v-if="backup.encrypted" type="warning" effect="dark" size="small">加密</el-tag>
                </div>
                <div class="backup-meta">
                  <span>{{ formatDate(backup.createdAt) }}</span>
                  <span>{{ formatFileSize(backup.fileSize) }}</span>
                  <span>v{{ backup.version }}</span>
                </div>
              </div>
              <div class="backup-actions">
                <button class="brick-btn brick-btn-sm" @click="openRestoreDialog(backup)">
                  <el-icon><Upload /></el-icon>
                  恢复
                </button>
                <button class="brick-btn brick-btn-sm brick-btn-secondary" @click="handleDeleteBackup(backup)">
                  <el-icon><Delete /></el-icon>
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="settings-section brick-card">
        <div class="section-header">
          <el-icon><List :size="20" /></el-icon>
          <h2>操作日志</h2>
        </div>
        <div class="section-body">
          <div class="setting-item">
            <div class="setting-info">
              <h3>全局操作日志</h3>
              <p>查看所有数据的增删改操作记录，支持按操作类型和对象类型筛选</p>
            </div>
            <div class="setting-action">
              <button
                class="brick-btn brick-btn-sm"
                @click="openLogsDialog"
              >
                <el-icon><List /></el-icon>
                查看日志
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

    <el-dialog
      v-model="restoreDialogVisible"
      title="备份恢复"
      width="520px"
      :close-on-click-modal="false"
    >
      <div v-if="restoreTarget" class="restore-dialog-content">
        <div class="restore-target-info">
          <p><strong>备份文件：</strong>{{ restoreTarget.filename }}</p>
          <p><strong>创建时间：</strong>{{ formatDate(restoreTarget.createdAt) }}</p>
          <p><strong>文件大小：</strong>{{ formatFileSize(restoreTarget.fileSize) }}</p>
          <p v-if="restoreTarget.encrypted"><el-tag type="warning" effect="dark" size="small">加密备份</el-tag></p>
        </div>

        <el-form label-width="100px" style="margin-top: 16px">
          <el-form-item label="恢复模式">
            <el-radio-group v-model="restoreMode">
              <el-radio value="full">全量覆盖</el-radio>
              <el-radio value="merge">合并增量</el-radio>
            </el-radio-group>
          </el-form-item>

          <el-form-item v-if="restoreTarget.encrypted" label="解密密码">
            <el-input
              v-model="restorePassword"
              type="password"
              placeholder="请输入备份密码"
              show-password
            />
          </el-form-item>
        </el-form>

        <div class="warning-text">
          <el-icon><Warning /></el-icon>
          <span v-if="restoreMode === 'full'">全量覆盖将替换当前所有数据（数据库 + 图片 + 密钥），此操作不可恢复！</span>
          <span v-else>合并增量仅恢复不存在的图片文件，数据库保持不变。</span>
        </div>
      </div>

      <template #footer>
        <button
          class="brick-btn brick-btn-secondary"
          @click="restoreDialogVisible = false"
        >
          取消
        </button>
        <button
          class="brick-btn"
          :disabled="restoreTarget?.encrypted && !restorePassword"
          :loading="restoreLoading"
          @click="handleRestore"
        >
          确认恢复
        </button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="integrityDialogVisible"
      title="数据库完整性检查"
      width="560px"
    >
      <div v-if="integrityResult" class="integrity-dialog-content">
        <div v-if="integrityResult.ok" class="integrity-ok">
          <el-icon :size="48" color="#4CAF50"><CircleCheck /></el-icon>
          <p>数据库完整性检查通过，未发现异常</p>
        </div>
        <div v-else class="integrity-fail">
          <el-icon :size="48" color="#F44336"><CircleClose /></el-icon>
          <p class="fail-title">数据库完整性检查失败</p>
          <div class="error-list">
            <div v-for="(err, idx) in integrityResult.errors" :key="idx" class="error-item">
              {{ err }}
            </div>
          </div>
          <div v-if="integrityResult.canAutoRecover && integrityResult.latestBackup" class="recover-section">
            <div class="warning-text">
              <el-icon><Warning /></el-icon>
              检测到可用的备份，可以从最近备份自动恢复
            </div>
            <div class="recover-backup-info">
              <p><strong>备份文件：</strong>{{ integrityResult.latestBackup.filename }}</p>
              <p><strong>创建时间：</strong>{{ formatDate(integrityResult.latestBackup.createdAt) }}</p>
            </div>
            <button class="brick-btn" @click="handleAutoRecover">
              <el-icon><Upload /></el-icon>
              从此备份恢复
            </button>
          </div>
        </div>
      </div>

      <template #footer>
        <button class="brick-btn brick-btn-secondary" @click="integrityDialogVisible = false">
          关闭
        </button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="logsDialogVisible"
      title="全局操作日志"
      width="900px"
      :close-on-click-modal="false"
    >
      <div class="logs-filter">
        <div class="filter-item">
          <el-select
            v-model="filterOperationType"
            placeholder="操作类型"
            clearable
            style="width: 160px"
          >
            <el-option
              v-for="opt in OPERATION_TYPE_OPTIONS"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </div>
        <div class="filter-item">
          <el-select
            v-model="filterObjectType"
            placeholder="对象类型"
            clearable
            style="width: 160px"
          >
            <el-option
              v-for="opt in OBJECT_TYPE_OPTIONS"
              :key="opt.value"
              :label="opt.label"
              :value="opt.value"
            />
          </el-select>
        </div>
        <div class="filter-item filter-actions">
          <button class="brick-btn brick-btn-sm" @click="loadOperationLogs">
            <el-icon><Refresh /></el-icon>
            查询
          </button>
          <button
            class="brick-btn brick-btn-sm brick-btn-secondary"
            @click="handleFilterReset"
          >
            重置
          </button>
        </div>
      </div>

      <div v-loading="logsLoading" class="logs-table-wrap">
        <el-table
          :data="operationLogs"
          border
          stripe
          style="width: 100%"
          max-height="500"
          empty-text="暂无操作日志"
        >
          <el-table-column label="操作类型" width="100">
            <template #default="{ row }">
              <el-tag :type="getOperationTypeTagType(row.operationType)" effect="dark" size="small">
                {{ getOperationTypeLabel(row.operationType) }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="对象类型" width="110">
            <template #default="{ row }">
              {{ getObjectTypeLabel(row.objectType) }}
            </template>
          </el-table-column>
          <el-table-column label="对象名称" min-width="160">
            <template #default="{ row }">
              {{ row.objectName || "-" }}
            </template>
          </el-table-column>
          <el-table-column label="操作时间" width="180">
            <template #default="{ row }">
              {{ formatDate(row.changedAt) }}
            </template>
          </el-table-column>
          <el-table-column label="操作" width="100">
            <template #default="{ row }">
              <button class="link-btn" @click="viewLogDetail(row)">
                <el-icon><View /></el-icon>
                查看详情
              </button>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <template #footer>
        <button class="brick-btn brick-btn-secondary" @click="logsDialogVisible = false">
          关闭
        </button>
      </template>
    </el-dialog>

    <el-dialog
      v-model="logDetailDialogVisible"
      title="操作日志详情"
      width="680px"
    >
      <div v-if="currentLog" class="log-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="操作类型">
            <el-tag :type="getOperationTypeTagType(currentLog.operationType)" effect="dark" size="small">
              {{ getOperationTypeLabel(currentLog.operationType) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="对象类型">
            {{ getObjectTypeLabel(currentLog.objectType) }}
          </el-descriptions-item>
          <el-descriptions-item label="对象名称" :span="2">
            {{ currentLog.objectName || "-" }}
          </el-descriptions-item>
          <el-descriptions-item label="对象 ID" :span="2">
            <span class="mono-text">{{ currentLog.objectId }}</span>
          </el-descriptions-item>
          <el-descriptions-item label="操作时间" :span="2">
            {{ formatDate(currentLog.changedAt) }}
          </el-descriptions-item>
        </el-descriptions>

        <div class="snapshot-section">
          <h4>变更前</h4>
          <pre class="snapshot-content">{{ formatSnapshot(currentLog.beforeSnapshot) }}</pre>
        </div>

        <div class="snapshot-section">
          <h4>变更后</h4>
          <pre class="snapshot-content">{{ formatSnapshot(currentLog.afterSnapshot) }}</pre>
        </div>
      </div>
      <template #footer>
        <button class="brick-btn brick-btn-secondary" @click="logDetailDialogVisible = false">
          关闭
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

.backup-action-row {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
}

.backup-encrypt-toggle {
  display: flex;
  align-items: center;
  gap: $spacing-xs;
  font-size: $font-size-sm;
  color: $color-gray-light;
}

.backup-config-row {
  display: flex;
  gap: $spacing-lg;
  padding: $spacing-md $spacing-md $spacing-lg;
  background: $color-dark;
  border-radius: $brick-radius;
  margin-bottom: $spacing-md;
  border: 1px solid $color-dark-border;

  .config-field {
    display: flex;
    flex-direction: column;
    gap: $spacing-xs;
  }

  .config-label {
    font-size: $font-size-sm;
    color: $color-gray-dark;
  }
}

.backups-list {
  margin-top: $spacing-md;
  min-height: 60px;
}

.empty-backups {
  text-align: center;
  padding: $spacing-xl;
  color: $color-gray-dark;
  font-size: $font-size-sm;
}

.backup-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: $spacing-md;
  border: 1px solid $color-dark-border;
  border-radius: $brick-radius;
  margin-bottom: $spacing-sm;
  background: $color-dark;

  &:last-child {
    margin-bottom: 0;
  }
}

.backup-info {
  flex: 1;
}

.backup-name {
  display: flex;
  align-items: center;
  gap: $spacing-sm;
  margin-bottom: $spacing-xs;
}

.backup-filename {
  font-size: $font-size-base;
  color: $color-white;
  font-weight: 500;
}

.backup-meta {
  display: flex;
  gap: $spacing-md;
  font-size: $font-size-sm;
  color: $color-gray-dark;
}

.backup-actions {
  display: flex;
  gap: $spacing-xs;
  flex-shrink: 0;
}

.restore-dialog-content {
  .restore-target-info {
    padding: $spacing-md;
    background: $color-dark;
    border-radius: $brick-radius;
    border: 1px solid $color-dark-border;

    p {
      margin: $spacing-xs 0;
      font-size: $font-size-sm;
      color: $color-gray-light;

      strong {
        color: $color-white;
      }
    }
  }
}

.integrity-dialog-content {
  text-align: center;
}

.integrity-ok {
  p {
    color: $color-success;
    font-size: $font-size-lg;
    font-weight: 500;
    margin-top: $spacing-md;
  }
}

.integrity-fail {
  .fail-title {
    color: $color-danger;
    font-size: $font-size-lg;
    font-weight: 500;
    margin-top: $spacing-md;
  }
}

.error-list {
  text-align: left;
  margin: $spacing-md 0;
}

.error-item {
  padding: $spacing-xs $spacing-md;
  background: rgba(244, 67, 54, 0.1);
  border-radius: $brick-radius;
  margin-bottom: $spacing-xs;
  font-size: $font-size-sm;
  color: $color-danger;
}

.recover-section {
  text-align: left;
  margin-top: $spacing-lg;

  .recover-backup-info {
    padding: $spacing-md;
    background: $color-dark;
    border-radius: $brick-radius;
    border: 1px solid $color-dark-border;
    margin-bottom: $spacing-md;

    p {
      margin: $spacing-xs 0;
      font-size: $font-size-sm;
      color: $color-gray-light;

      strong {
        color: $color-white;
      }
    }
  }
}

.logs-filter {
  display: flex;
  gap: $spacing-sm;
  margin-bottom: $spacing-md;
  align-items: center;

  .filter-actions {
    display: flex;
    gap: $spacing-xs;
    margin-left: auto;
  }
}

.logs-table-wrap {
  min-height: 200px;
}

.log-detail {
  .snapshot-section {
    margin-top: $spacing-md;

    h4 {
      margin: 0 0 $spacing-xs 0;
      font-size: $font-size-sm;
      color: $color-gray-light;
      font-weight: 500;
    }
  }

  .snapshot-content {
    background: $color-dark;
    border: 1px solid $color-dark-border;
    border-radius: $brick-radius;
    padding: $spacing-md;
    max-height: 240px;
    overflow-y: auto;
    color: $color-gray-light;
    font-size: $font-size-sm;
    margin: 0;
    white-space: pre-wrap;
    word-break: break-all;
  }
}

.mono-text {
  font-family: monospace;
}

:deep(.el-descriptions) {
  --el-descriptions-table-border: 1px solid $color-dark-border;
  --el-descriptions-item-label-bg: $color-dark-lighter;
  --el-descriptions-item-content-bg: $color-dark;
  --el-text-color-primary: $color-white;
  --el-text-color-regular: $color-gray-light;
}

:deep(.el-radio) {
  color: $color-gray-light;
}

:deep(.el-radio__input.is-checked .el-radio__inner) {
  border-color: $color-primary;
  background: $color-primary;
}
</style>
