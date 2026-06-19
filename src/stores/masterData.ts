import { defineStore } from "pinia";
import { ref } from "vue";
import { api } from "@/api";
import type { PartType, PartColor, PartSize, Location, LocationTreeNode } from "@/types";

export const useMasterDataStore = defineStore("masterData", () => {
  const partTypes = ref<PartType[]>([]);
  const partColors = ref<PartColor[]>([]);
  const partSizes = ref<PartSize[]>([]);
  const locations = ref<Location[]>([]);
  const loading = ref(false);

  async function loadAll() {
    loading.value = true;
    try {
      await Promise.all([
        loadPartTypes(),
        loadPartColors(),
        loadPartSizes(),
        loadLocations(),
      ]);
    } finally {
      loading.value = false;
    }
  }

  async function loadPartTypes() {
    const response = await api.getPartTypes();
    if (response.success) {
      partTypes.value = response.data;
    }
  }

  async function addPartType(type: Omit<PartType, "id">): Promise<PartType | undefined> {
    const response = await api.createPartType(type);
    if (response.success) {
      partTypes.value.push(response.data);
      return response.data;
    }
    return undefined;
  }

  async function updatePartType(type: PartType): Promise<PartType | undefined> {
    const response = await api.updatePartType(type);
    if (response.success) {
      const index = partTypes.value.findIndex((t) => t.id === type.id);
      if (index !== -1) {
        partTypes.value[index] = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function deletePartType(id: string) {
    const response = await api.deletePartType(id);
    if (response.success) {
      partTypes.value = partTypes.value.filter((t) => t.id !== id);
    }
  }

  async function loadPartColors() {
    const response = await api.getPartColors();
    if (response.success) {
      partColors.value = response.data;
    }
  }

  async function addPartColor(color: Omit<PartColor, "id">): Promise<PartColor | undefined> {
    const response = await api.createPartColor(color);
    if (response.success) {
      partColors.value.push(response.data);
      return response.data;
    }
    return undefined;
  }

  async function updatePartColor(color: PartColor): Promise<PartColor | undefined> {
    const response = await api.updatePartColor(color);
    if (response.success) {
      const index = partColors.value.findIndex((c) => c.id === color.id);
      if (index !== -1) {
        partColors.value[index] = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function deletePartColor(id: string) {
    const response = await api.deletePartColor(id);
    if (response.success) {
      partColors.value = partColors.value.filter((c) => c.id !== id);
    }
  }

  async function loadPartSizes() {
    const response = await api.getPartSizes();
    if (response.success) {
      partSizes.value = response.data;
    }
  }

  async function addPartSize(size: Omit<PartSize, "id">): Promise<PartSize | undefined> {
    const response = await api.createPartSize(size);
    if (response.success) {
      partSizes.value.push(response.data);
      return response.data;
    }
    return undefined;
  }

  async function updatePartSize(size: PartSize): Promise<PartSize | undefined> {
    const response = await api.updatePartSize(size);
    if (response.success) {
      const index = partSizes.value.findIndex((s) => s.id === size.id);
      if (index !== -1) {
        partSizes.value[index] = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function deletePartSize(id: string) {
    const response = await api.deletePartSize(id);
    if (response.success) {
      partSizes.value = partSizes.value.filter((s) => s.id !== id);
    }
  }

  async function loadLocations() {
    const response = await api.getLocations();
    if (response.success) {
      locations.value = response.data;
    }
  }

  async function addLocation(location: Omit<Location, "id">): Promise<Location | undefined> {
    const response = await api.createLocation(location);
    if (response.success) {
      locations.value.push(response.data);
      return response.data;
    }
    return undefined;
  }

  async function updateLocation(location: Location): Promise<Location | undefined> {
    const response = await api.updateLocation(location);
    if (response.success) {
      const index = locations.value.findIndex((l) => l.id === location.id);
      if (index !== -1) {
        locations.value[index] = response.data;
      }
      return response.data;
    }
    return undefined;
  }

  async function deleteLocation(id: string) {
    const response = await api.deleteLocation(id);
    if (response.success) {
      locations.value = locations.value.filter((l) => l.id !== id);
    }
  }

  function getPartTypeName(code: string) {
    return partTypes.value.find((t) => t.code === code)?.name || code;
  }

  function getPartColorName(name: string) {
    return partColors.value.find((c) => c.name === name)?.name || name;
  }

  function getPartColorHex(name: string) {
    return partColors.value.find((c) => c.name === name)?.hex || "#888888";
  }

  function getPartSizeName(name: string) {
    return partSizes.value.find((s) => s.name === name)?.name || name;
  }

  function getLocationName(code: string) {
    return locations.value.find((l) => l.code === code)?.name || code;
  }

  function buildLocationTree(): LocationTreeNode[] {
    const list = locations.value;
    const map = new Map<string, LocationTreeNode>();
    const roots: LocationTreeNode[] = [];

    for (const loc of list) {
      map.set(loc.id, { ...loc, children: [] });
    }

    for (const loc of list) {
      const node = map.get(loc.id)!;
      if (loc.parentId && map.has(loc.parentId)) {
        map.get(loc.parentId)!.children!.push(node);
      } else {
        roots.push(node);
      }
    }

    return roots;
  }

  function getAllChildLocationCodes(parentId: string): string[] {
    const parent = locations.value.find((l) => l.id === parentId);
    if (!parent) return [];
    const codes = [parent.code];
    const children = locations.value.filter((l) => l.parentId === parentId);
    for (const child of children) {
      codes.push(...getAllChildLocationCodes(child.id));
    }
    return codes;
  }

  function getAllChildLocationCodesByCode(code: string): string[] {
    const loc = locations.value.find((l) => l.code === code);
    if (!loc) return [code];
    return getAllChildLocationCodes(loc.id);
  }

  function getLocationAncestors(locationId: string): Location[] {
    const result: Location[] = [];
    let current = locations.value.find((l) => l.id === locationId);
    while (current) {
      result.unshift(current);
      current = current.parentId
        ? locations.value.find((l) => l.id === current!.parentId)
        : undefined;
    }
    return result;
  }

  return {
    partTypes,
    partColors,
    partSizes,
    locations,
    loading,
    loadAll,
    loadPartTypes,
    loadPartColors,
    loadPartSizes,
    loadLocations,
    addPartType,
    updatePartType,
    deletePartType,
    addPartColor,
    updatePartColor,
    deletePartColor,
    addPartSize,
    updatePartSize,
    deletePartSize,
    addLocation,
    updateLocation,
    deleteLocation,
    getPartTypeName,
    getPartColorName,
    getPartColorHex,
    getPartSizeName,
    getLocationName,
    buildLocationTree,
    getAllChildLocationCodes,
    getAllChildLocationCodesByCode,
    getLocationAncestors,
  };
});
