import { defineStore } from "pinia";
import { ref } from "vue";
import { ElMessage, ElMessageBox } from "element-plus";
import { api } from "@/api";

export const useAppStore = defineStore("app", () => {
  const loading = ref(false);
  const loadingText = ref("加载中...");
  const initialized = ref(false);
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

    startLoading("正在初始化应用...");
    try {
      await api.initDatabase();
      encryptionKey.value = await api.getEncryptionKey();
      initialized.value = true;
    } catch (error) {
      console.error("初始化失败:", error);
      showError("应用初始化失败，请重启应用");
    } finally {
      stopLoading();
    }
  }

  async function changeEncryptionKey(oldKey: string, newKey: string) {
    startLoading("正在修改加密密钥...");
    try {
      await api.changeEncryptionKey(oldKey, newKey);
      encryptionKey.value = newKey;
      showSuccess("加密密钥修改成功");
      return true;
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
