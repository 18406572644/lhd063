import { defineStore } from "pinia";
import { ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "@/api";
import type { ApiResponse } from "@/types";

export const useAppStore = defineStore("app", () => {
  const loading = ref(false);
  const loadingText = ref("加载中...");
  const initialized = ref(false);
  const initError = ref(false);
  const sidebarCollapsed = ref(false);
  const encryptionKey = ref("");

  function startLoading(text = "加载中...") {
    loading.value = true;
    loadingText.value = text;
  }

  function stopLoading() {
    loading.value = false;
    loadingText.value = "加载中...";
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value;
  }

  function showSuccess(message: string) {
    ElMessage.success({
      message,
      duration: 3000,
      showClose: true,
    });
  }

  function showError(message: string) {
    ElMessage.error({
      message,
      duration: 5000,
      showClose: true,
    });
  }

  function showWarning(message: string) {
    ElMessage.warning({
      message,
      duration: 4000,
      showClose: true,
    });
  }

  function showInfo(message: string) {
    ElMessage.info({
      message,
      duration: 3000,
      showClose: true,
    });
  }

  async function showConfirm(message: string, title = "确认操作") {
    try {
      await ElMessageBox.confirm(message, title, {
        confirmButtonText: "确定",
        cancelButtonText: "取消",
        type: "warning",
        confirmButtonClass: "brick-btn",
        cancelButtonClass: "brick-btn brick-btn-secondary",
      });
      return true;
    } catch {
      return false;
    }
  }

  async function initializeApp() {
    if (initialized.value) return;

    initError.value = false;
    startLoading("正在初始化应用...");
    try {
      const initPromise = api.initDatabase();
      const timeoutPromise = new Promise<ApiResponse<void>>((resolve) =>
        setTimeout(() => resolve({ success: false, data: undefined as unknown as void, error: { code: "NETWORK_ERROR", message: "初始化超时" } }), 15000)
      );
      const initResponse = await Promise.race([initPromise, timeoutPromise]);
      if (!initResponse.success) {
        throw new Error(initResponse.error?.message || "初始化超时");
      }

      const keyResponse = await api.getEncryptionKey();
      if (keyResponse.success) {
        encryptionKey.value = keyResponse.data;
      }
      initialized.value = true;

      checkIntegrityOnStartup();
      performAutoBackupIfNeeded();
    } catch (error) {
      console.error("初始化失败:", error);
      initError.value = true;
      showError("应用初始化失败，请点击重试");
    } finally {
      stopLoading();
    }
  }

  async function checkIntegrityOnStartup() {
    try {
      const response = await api.checkDatabaseIntegrity();
      if (response.success && !response.data.ok) {
        showWarning("数据库完整性异常，请在设置中进行完整性检查并恢复备份");
      }
    } catch {
      // silently ignore
    }
  }

  async function performAutoBackupIfNeeded() {
    try {
      const neededResponse = await api.shouldAutoBackup();
      if (neededResponse.success && neededResponse.data) {
        const configResponse = await api.getBackupConfig();
        if (configResponse.success) {
          const password = configResponse.data.encrypt ? undefined : undefined;
          await api.createBackup(password);
        }
      }
    } catch {
      // silently ignore
    }
  }

  async function changeEncryptionKey(oldKey: string, newKey: string) {
    startLoading("正在修改加密密钥...");
    try {
      const response = await api.changeEncryptionKey(oldKey, newKey);
      if (response.success) {
        encryptionKey.value = newKey;
        showSuccess("加密密钥修改成功");
        return true;
      } else {
        showError(response.error?.message || "修改加密密钥失败");
        return false;
      }
    } catch (error) {
      showError("修改加密密钥失败");
      return false;
    } finally {
      stopLoading();
    }
  }

  return {
    loading,
    loadingText,
    initialized,
    initError,
    sidebarCollapsed,
    encryptionKey,
    startLoading,
    stopLoading,
    toggleSidebar,
    showSuccess,
    showError,
    showWarning,
    showInfo,
    showConfirm,
    initializeApp,
    changeEncryptionKey,
  };
});
