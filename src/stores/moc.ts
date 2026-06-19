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
      mocLists.value = await api.getMocLists();
    } finally {
      loading.value = false;
    }
  }

  async function loadMocById(id: string) {
    currentMoc.value = await api.getMocListById(id);
    return currentMoc.value;
  }

  async function compareInventory(id: string) {
    currentMoc.value = await api.compareMocInventory(id);
    const index = mocLists.value.findIndex((m) => m.id === id);
    if (index !== -1) {
      mocLists.value[index] = currentMoc.value;
    }
    return currentMoc.value;
  }

  async function addMocList(moc: Omit<MocList, "id" | "createdAt" | "updatedAt">) {
    const newMoc = await api.createMocList(moc);
    mocLists.value.push(newMoc);
    return newMoc;
  }

  async function updateMocList(moc: MocList) {
    const updated = await api.updateMocList(moc);
    const index = mocLists.value.findIndex((m) => m.id === moc.id);
    if (index !== -1) {
      mocLists.value[index] = updated;
    }
    if (currentMoc.value?.id === moc.id) {
      currentMoc.value = updated;
    }
    return updated;
  }

  async function deleteMocList(id: string) {
    await api.deleteMocList(id);
    mocLists.value = mocLists.value.filter((m) => m.id !== id);
    if (currentMoc.value?.id === id) {
      currentMoc.value = null;
    }
  }

  async function changeMocStatus(mocId: string, newStatus: MocStatus, remark?: string) {
    const updated = await api.changeMocStatus({ mocId, newStatus, remark });
    const index = mocLists.value.findIndex((m) => m.id === mocId);
    if (index !== -1) {
      mocLists.value[index] = updated;
    }
    if (currentMoc.value?.id === mocId) {
      currentMoc.value = updated;
    }
    return updated;
  }

  async function loadStatusLogs(mocId: string) {
    statusLogsLoading.value = true;
    try {
      statusLogs.value = await api.getMocStatusLogs(mocId);
    } finally {
      statusLogsLoading.value = false;
    }
  }

  function clearStatusLogs() {
    statusLogs.value = [];
  }

  async function saveMocCoverImage(mocId: string, imageData: string) {
    const path = await api.saveMocCoverImage(mocId, imageData);
    const index = mocLists.value.findIndex((m) => m.id === mocId);
    if (index !== -1) {
      mocLists.value[index] = { ...mocLists.value[index], coverImagePath: path };
    }
    if (currentMoc.value?.id === mocId) {
      currentMoc.value = { ...currentMoc.value, coverImagePath: path };
    }
    return path;
  }

  async function deleteMocCoverImage(mocId: string) {
    await api.deleteMocCoverImage(mocId);
    const index = mocLists.value.findIndex((m) => m.id === mocId);
    if (index !== -1) {
      mocLists.value[index] = { ...mocLists.value[index], coverImagePath: undefined };
    }
    if (currentMoc.value?.id === mocId) {
      currentMoc.value = { ...currentMoc.value, coverImagePath: undefined };
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
