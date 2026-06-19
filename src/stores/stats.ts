import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/api";
import type { StatsData } from "@/types";

export const useStatsStore = defineStore("stats", () => {
  const stats = ref<StatsData | null>(null);
  const loading = ref(false);

  async function loadStats() {
    loading.value = true;
    try {
      const response = await api.getStats();
      if (response.success) {
        stats.value = response.data;
      }
    } finally {
      loading.value = false;
    }
  }

  function clearStats() {
    stats.value = null;
  }

  return {
    stats,
    loading,
    loadStats,
    clearStats,
  };
});
