import { defineStore } from "pinia";
import { ref, computed, watch } from "vue";
import type { SavedView, PartFilter } from "@/types";
import { PRESET_VIEWS } from "@/types";

const STORAGE_KEY = "lego-saved-views";
const ACTIVE_VIEW_KEY = "lego-active-view";

function generateId(): string {
  return "view_" + Date.now().toString(36) + "_" + Math.random().toString(36).slice(2, 8);
}

function loadFromStorage(): SavedView[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      return JSON.parse(raw);
    }
  } catch {
    // ignore
  }
  return [];
}

function saveToStorage(views: SavedView[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(views));
  } catch {
    // ignore
  }
}

function loadActiveViewId(): string | null {
  try {
    return localStorage.getItem(ACTIVE_VIEW_KEY);
  } catch {
    return null;
  }
}

function saveActiveViewId(id: string | null) {
  try {
    if (id) {
      localStorage.setItem(ACTIVE_VIEW_KEY, id);
    } else {
      localStorage.removeItem(ACTIVE_VIEW_KEY);
    }
  } catch {
    // ignore
  }
}

export const useViewsStore = defineStore("views", () => {
  const customViews = ref<SavedView[]>(loadFromStorage());
  const activeViewId = ref<string | null>(loadActiveViewId());

  const presetViews = computed<SavedView[]>(() =>
    PRESET_VIEWS.map((v, idx) => ({
      ...v,
      id: "preset_" + idx,
      createdAt: new Date(0).toISOString(),
      updatedAt: new Date(0).toISOString(),
    }))
  );

  const allViews = computed<SavedView[]>(() => [...presetViews.value, ...customViews.value]);

  const defaultView = computed(() => {
    const fromCustom = customViews.value.find((v) => v.isDefault);
    if (fromCustom) return fromCustom;
    return presetViews.value.find((v) => v.isDefault) || presetViews.value[0];
  });

  const activeView = computed(() => {
    if (activeViewId.value) {
      const found = allViews.value.find((v) => v.id === activeViewId.value);
      if (found) return found;
    }
    return defaultView.value;
  });

  watch(
    customViews,
    (val) => {
      saveToStorage(val);
    },
    { deep: true }
  );

  watch(activeViewId, (val) => {
    saveActiveViewId(val);
  });

  function setActiveView(id: string) {
    activeViewId.value = id;
  }

  function resetActiveViewToDefault() {
    activeViewId.value = defaultView.value.id;
  }

  function saveView(params: {
    name: string;
    filter: PartFilter;
    sortField?: string;
    sortOrder?: "asc" | "desc";
    visibleColumns?: string[];
    icon?: string;
    description?: string;
  }): SavedView {
    const now = new Date().toISOString();
    const view: SavedView = {
      id: generateId(),
      name: params.name,
      filter: params.filter,
      sortField: params.sortField,
      sortOrder: params.sortOrder,
      visibleColumns: params.visibleColumns,
      icon: params.icon,
      description: params.description,
      createdAt: now,
      updatedAt: now,
    };
    customViews.value.push(view);
    return view;
  }

  function updateView(
    id: string,
    updates: Partial<Omit<SavedView, "id" | "createdAt" | "isPreset">>
  ): boolean {
    const idx = customViews.value.findIndex((v) => v.id === id);
    if (idx === -1) return false;
    customViews.value[idx] = {
      ...customViews.value[idx],
      ...updates,
      updatedAt: new Date().toISOString(),
    };
    return true;
  }

  function overwriteCurrentView(
    filter: PartFilter,
    sortField?: string,
    sortOrder?: "asc" | "desc",
    visibleColumns?: string[]
  ): boolean {
    if (!activeView.value || activeView.value.isPreset) return false;
    return updateView(activeView.value.id, {
      filter,
      sortField,
      sortOrder,
      visibleColumns,
    });
  }

  function deleteView(id: string): boolean {
    const idx = customViews.value.findIndex((v) => v.id === id);
    if (idx === -1) return false;
    customViews.value.splice(idx, 1);
    if (activeViewId.value === id) {
      activeViewId.value = defaultView.value.id;
    }
    return true;
  }

  function setAsDefault(id: string): boolean {
    const idx = customViews.value.findIndex((v) => v.id === id);
    if (idx === -1) {
      const pIdx = presetViews.value.findIndex((v) => v.id === id);
      if (pIdx === -1) return false;
      customViews.value.forEach((v) => (v.isDefault = false));
      return true;
    }
    customViews.value.forEach((v) => (v.isDefault = false));
    customViews.value[idx].isDefault = true;
    return true;
  }

  function renameView(id: string, newName: string): boolean {
    return updateView(id, { name: newName });
  }

  function duplicateView(id: string): SavedView | null {
    const source = allViews.value.find((v) => v.id === id);
    if (!source) return null;
    const now = new Date().toISOString();
    const copy: SavedView = {
      id: generateId(),
      name: source.name + " (副本)",
      filter: JSON.parse(JSON.stringify(source.filter || {})),
      sortField: source.sortField,
      sortOrder: source.sortOrder,
      visibleColumns: source.visibleColumns ? [...source.visibleColumns] : undefined,
      icon: source.icon,
      description: source.description,
      createdAt: now,
      updatedAt: now,
    };
    customViews.value.push(copy);
    return copy;
  }

  return {
    customViews,
    presetViews,
    allViews,
    activeViewId,
    activeView,
    defaultView,
    setActiveView,
    resetActiveViewToDefault,
    saveView,
    updateView,
    overwriteCurrentView,
    deleteView,
    setAsDefault,
    renameView,
    duplicateView,
  };
});
