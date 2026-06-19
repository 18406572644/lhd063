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
    partTypes.value = await api.getPartTypes();
  }

  async function addPartType(type: Omit<PartType, "id">) {
    const newType = await api.createPartType(type);
    partTypes.value.push(newType);
    return newType;
  }

  async function updatePartType(type: PartType) {
    const updated = await api.updatePartType(type);
    const index = partTypes.value.findIndex((t) => t.id === type.id);
    if (index !== -1) {
      partTypes.value[index] = updated;
    }
    return updated;
  }

  async function deletePartType(id: string) {
    await api.deletePartType(id);
    partTypes.value = partTypes.value.filter((t) => t.id !== id);
  }

  async function loadPartColors() {
    partColors.value = await api.getPartColors();
  }

  async function addPartColor(color: Omit<PartColor, "id">) {
    const newColor = await api.createPartColor(color);
    partColors.value.push(newColor);
    return newColor;
  }

  async function updatePartColor(color: PartColor) {
    const updated = await api.updatePartColor(color);
    const index = partColors.value.findIndex((c) => c.id === color.id);
    if (index !== -1) {
      partColors.value[index] = updated;
    }
    return updated;
  }

  async function deletePartColor(id: string) {
    await api.deletePartColor(id);
    partColors.value = partColors.value.filter((c) => c.id !== id);
  }

  async function loadPartSizes() {
    partSizes.value = await api.getPartSizes();
  }

  async function addPartSize(size: Omit<PartSize, "id">) {
    const newSize = await api.createPartSize(size);
    partSizes.value.push(newSize);
    return newSize;
  }

  async function updatePartSize(size: PartSize) {
    const updated = await api.updatePartSize(size);
    const index = partSizes.value.findIndex((s) => s.id === size.id);
    if (index !== -1) {
      partSizes.value[index] = updated;
    }
    return updated;
  }

  async function deletePartSize(id: string) {
    await api.deletePartSize(id);
    partSizes.value = partSizes.value.filter((s) => s.id !== id);
  }

  async function loadLocations() {
    locations.value = await api.getLocations();
  }

  async function addLocation(location: Omit<Location, "id">) {
    const newLocation = await api.createLocation(location);
    locations.value.push(newLocation);
    return newLocation;
  }

  async function updateLocation(location: Location) {
    const updated = await api.updateLocation(location);
    const index = locations.value.findIndex((l) => l.id === location.id);
    if (index !== -1) {
      locations.value[index] = updated;
    }
    return updated;
  }

  async function deleteLocation(id: string) {
    await api.deleteLocation(id);
    locations.value = locations.value.filter((l) => l.id !== id);
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
