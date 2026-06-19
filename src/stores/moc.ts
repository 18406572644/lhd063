import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "@/api";
import type { MocList, MocStatus, MocStatusLog } from "@/types";

export const useMocStore = defineStore("moc", () => {
  const mocLists = ref<MocList[]>([]);
  const loading = ref(false);
  const currentMoc = ref<MocList | null>(null);
  const statusLogs = ref<MocStatusLog[]>([]);
  const statusLogsLoading = ref(false);

  const mocListOptions = computed(() => {
    return mocLists.value.map((m) => ({
      label: m.name,
      value: m.id,
    }));
  });

  const totalMissingParts = computed(() => {
    return mocLists.value.reduce((sum, moc) => {
      const parts = moc.parts ?? [];
      return sum + parts.filter((p) => p.isMissing).length;
    }, 0);
  });

  const mocsByStatus = computed(() => {
    const map = new Map<MocStatus, MocList[]>();
    for (const moc of mocLists.value) {
      if (!map.has(moc.status)) {
        map.set(moc.status, []);
      }
      map.get(moc.status)!.push(moc);
    }
    return map;
  });

  const getStatusCount = (status: MocStatus) => {
    return mocsByStatus.value.get(status)?.length ?? 0;
  };

  async function loadMocLists() {
    loading.value = true;
    try {
      const response = await api.getMocLists();
      if (response.success) {
        mocLists.value = response.data;
      }
    } finally {
      loading.value = false;
    }
  }

  async function loadMocById(id: string) {
    const response = await api.getMocListById(id);
    if (response.success) {
      currentMoc.value = response.data;
    }
    return currentMoc.value;
  }

  async function compareInventory(id: string) {
    const response = await api.compareMocInventory(id);
    if (response.success) {
      currentMoc.value = response.data;
    }
    const index = mocLists.value.findIndex((m) => m.id === id);
    if (response.success && index !== -1) {
      mocLists.value[index] = response.data;
    }
    return currentMoc.value;
  }

  async function addMocList(moc: Omit<MocList, "id" | "createdAt" | "updatedAt">) {
    const response = await api.createMocList(moc);
    if (response.success) {
      mocLists.value.push(response.data);
      return response.data;
    }
    return undefined;
  }

  async function updateMocList(moc: MocList) {
    const response = await api.updateMocList(moc);
    if (response.success) {
      const index = mocLists.value.findIndex((m) => m.id === moc.id);
      if (index !== -1) {
        mocLists.value[index] = response.data;
      }
      if (currentMoc.value?.id === moc.id) {
        currentMoc.value = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function deleteMocList(id: string) {
    const response = await api.deleteMocList(id);
    if (response.success) {
      mocLists.value = mocLists.value.filter((m) => m.id !== id);
      if (currentMoc.value?.id === id) {
        currentMoc.value = null;
      }
    }
  }

  async function changeMocStatus(mocId: string, newStatus: MocStatus, remark?: string) {
    const response = await api.changeMocStatus({ mocId, newStatus, remark });
    if (response.success) {
      const index = mocLists.value.findIndex((m) => m.id === mocId);
      if (index !== -1) {
        mocLists.value[index] = response.data;
      }
      if (currentMoc.value?.id === mocId) {
        currentMoc.value = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function loadStatusLogs(mocId: string) {
    statusLogsLoading.value = true;
    try {
      const response = await api.getMocStatusLogs(mocId);
      if (response.success) {
        statusLogs.value = response.data;
      }
    } finally {
      statusLogsLoading.value = false;
    }
  }

  function clearStatusLogs() {
    statusLogs.value = [];
  }

  async function saveMocCoverImage(mocId: string, imageData: string) {
    const response = await api.saveMocCoverImage(mocId, imageData);
    if (response.success) {
      const path = response.data;
      const index = mocLists.value.findIndex((m) => m.id === mocId);
      if (index !== -1) {
        mocLists.value[index] = { ...mocLists.value[index], coverImagePath: path };
      }
      if (currentMoc.value?.id === mocId) {
        currentMoc.value = { ...currentMoc.value, coverImagePath: path };
      }
      return path;
    }
    return undefined;
  }

  async function deleteMocCoverImage(mocId: string) {
    const response = await api.deleteMocCoverImage(mocId);
    if (response.success) {
      const index = mocLists.value.findIndex((m) => m.id === mocId);
      if (index !== -1) {
        mocLists.value[index] = { ...mocLists.value[index], coverImagePath: undefined };
      }
      if (currentMoc.value?.id === mocId) {
        currentMoc.value = { ...currentMoc.value, coverImagePath: undefined };
      }
    }
  }

  function clearCurrentMoc() {
    currentMoc.value = null;
  }

  return {
    mocLists,
    loading,
    currentMoc,
    statusLogs,
    statusLogsLoading,
    mocListOptions,
    totalMissingParts,
    mocsByStatus,
    getStatusCount,
    loadMocLists,
    loadMocById,
    compareInventory,
    addMocList,
    updateMocList,
    deleteMocList,
    changeMocStatus,
    loadStatusLogs,
    clearStatusLogs,
    saveMocCoverImage,
    deleteMocCoverImage,
    clearCurrentMoc,
  };
});
