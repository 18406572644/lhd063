import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "@/api";
import type { Part, PartFilter } from "@/types";

export const usePartsStore = defineStore("parts", () => {
  const parts = ref<Part[]>([]);
  const loading = ref(false);
  const filter = ref<PartFilter>({});

  const filteredParts = computed(() => {
    let result = [...parts.value];

    if (filter.value.type) {
      result = result.filter((p) => p.type === filter.value.type);
    }
    if (filter.value.color) {
      result = result.filter((p) => p.color === filter.value.color);
    }
    if (filter.value.size) {
      result = result.filter((p) => p.size === filter.value.size);
    }
    if (filter.value.location) {
      result = result.filter((p) => p.location === filter.value.location);
    }
    if (filter.value.keyword) {
      const keyword = filter.value.keyword.toLowerCase();
      result = result.filter(
        (p) =>
          p.name.toLowerCase().includes(keyword) ||
          p.partNumber.toLowerCase().includes(keyword) ||
          p.description?.toLowerCase().includes(keyword)
      );
    }

    return result;
  });

  const totalQuantity = computed(() => {
    return parts.value.reduce((sum, p) => sum + p.quantity, 0);
  });

  const lowStockParts = computed(() => {
    return parts.value.filter((p) => p.quantity <= 5);
  });

  async function loadParts() {
    loading.value = true;
    try {
      parts.value = await api.getParts();
    } finally {
      loading.value = false;
    }
  }

  async function addPart(part: Omit<Part, "id" | "createdAt" | "updatedAt">) {
    const newPart = await api.createPart(part);
    parts.value.push(newPart);
    return newPart;
  }

  async function updatePart(part: Part) {
    const updated = await api.updatePart(part);
    const index = parts.value.findIndex((p) => p.id === part.id);
    if (index !== -1) {
      parts.value[index] = updated;
    }
    return updated;
  }

  async function deletePart(id: string) {
    await api.deletePart(id);
    parts.value = parts.value.filter((p) => p.id !== id);
  }

  function getPartById(id: string) {
    return parts.value.find((p) => p.id === id);
  }

  function setFilter(newFilter: Partial<PartFilter>) {
    filter.value = { ...filter.value, ...newFilter };
  }

  function clearFilter() {
    filter.value = {};
  }

  return {
    parts,
    loading,
    filter,
    filteredParts,
    totalQuantity,
    lowStockParts,
    loadParts,
    addPart,
    updatePart,
    deletePart,
    getPartById,
    setFilter,
    clearFilter,
  };
});
