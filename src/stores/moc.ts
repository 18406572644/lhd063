import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "@/api";
import type { MocList } from "@/types";

export const useMocStore = defineStore("moc", () => {
  const mocLists = ref<MocList[]>([]);
  const loading = ref(false);
  const currentMoc = ref<MocList | null>(null);

  const mocListOptions = computed(() => {
    return mocLists.value.map((m) => ({
      label: m.name,
      value: m.id,
    }));
  });

  const totalMissingParts = computed(() => {
    return mocLists.value.reduce((sum, moc) => {
      return sum + moc.parts.filter((p) => p.isMissing).length;
    }, 0);
  });

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

  function clearCurrentMoc() {
    currentMoc.value = null;
  }

  return {
    mocLists,
    loading,
    currentMoc,
    mocListOptions,
    totalMissingParts,
    loadMocLists,
    loadMocById,
    compareInventory,
    addMocList,
    updateMocList,
    deleteMocList,
    clearCurrentMoc,
  };
});
