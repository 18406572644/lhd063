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
      stats.value = await api.getStats();
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
