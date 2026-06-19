import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { api } from "@/api";
import type {
  Part,
  PartFilter,
  DynamicTimeOption,
  AdvancedFilterCondition,
  AdvancedFilterGroup,
  LogicMode,
} from "@/types";
import { useMasterDataStore } from "./masterData";

function resolveDynamicTimeRange(
  option: DynamicTimeOption
): { from: Date; to: Date } | null {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const endOfToday = new Date(today.getTime() + 86400000 - 1);

  switch (option) {
    case "today":
      return { from: today, to: endOfToday };
    case "yesterday": {
      const yesterday = new Date(today.getTime() - 86400000);
      return {
        from: yesterday,
        to: new Date(yesterday.getTime() + 86400000 - 1),
      };
    }
    case "last7days":
      return { from: new Date(today.getTime() - 6 * 86400000), to: endOfToday };
    case "last30days":
      return { from: new Date(today.getTime() - 29 * 86400000), to: endOfToday };
    case "thisWeek": {
      const day = today.getDay() === 0 ? 6 : today.getDay() - 1;
      const weekStart = new Date(today.getTime() - day * 86400000);
      return {
        from: weekStart,
        to: new Date(weekStart.getTime() + 7 * 86400000 - 1),
      };
    }
    case "thisMonth": {
      const monthStart = new Date(now.getFullYear(), now.getMonth(), 1);
      const monthEnd = new Date(now.getFullYear(), now.getMonth() + 1, 0, 23, 59, 59, 999);
      return { from: monthStart, to: monthEnd };
    }
    case "thisQuarter": {
      const quarter = Math.floor(now.getMonth() / 3);
      const quarterStart = new Date(now.getFullYear(), quarter * 3, 1);
      const quarterEnd = new Date(now.getFullYear(), quarter * 3 + 3, 0, 23, 59, 59, 999);
      return { from: quarterStart, to: quarterEnd };
    }
    case "thisYear": {
      const yearStart = new Date(now.getFullYear(), 0, 1);
      const yearEnd = new Date(now.getFullYear(), 11, 31, 23, 59, 59, 999);
      return { from: yearStart, to: yearEnd };
    }
    case "lastWeek": {
      const day = today.getDay() === 0 ? 6 : today.getDay() - 1;
      const weekStart = new Date(today.getTime() - day * 86400000);
      const lastWeekStart = new Date(weekStart.getTime() - 7 * 86400000);
      return {
        from: lastWeekStart,
        to: new Date(lastWeekStart.getTime() + 7 * 86400000 - 1),
      };
    }
    case "lastMonth": {
      const lastMonthStart = new Date(now.getFullYear(), now.getMonth() - 1, 1);
      const lastMonthEnd = new Date(now.getFullYear(), now.getMonth(), 0, 23, 59, 59, 999);
      return { from: lastMonthStart, to: lastMonthEnd };
    }
    default:
      return null;
  }
}

function parseISODate(s: string): Date {
  return new Date(s);
}

function dateInRange(dateStr: string, from?: string, to?: string): boolean {
  const d = parseISODate(dateStr).getTime();
  if (from && d < parseISODate(from).getTime()) return false;
  if (to && d > parseISODate(to).getTime()) return false;
  return true;
}

function evalCondition(part: Part, cond: AdvancedFilterCondition): boolean {
  const masterDataStore = useMasterDataStore();

  let fieldValue: any;
  switch (cond.field) {
    case "location": {
      let locVal: any = part[cond.field];
      if (cond.operator === "in" || cond.operator === "notIn") {
        const codes = new Set<string>();
        (Array.isArray(cond.value) ? cond.value : []).forEach((c: string) => {
          masterDataStore.getAllChildLocationCodesByCode(c).forEach((x) => codes.add(x));
        });
        locVal = codes.has(part.location) ? cond.value?.[0] : undefined;
      } else if (typeof cond.value === "string") {
        const codes = new Set(
          masterDataStore.getAllChildLocationCodesByCode(cond.value as string)
        );
        fieldValue = codes.has(part.location) ? cond.value : part.location;
        break;
      }
      fieldValue = locVal;
      break;
    }
    case "hasImage":
      fieldValue = !!part.imagePath;
      break;
    default:
      fieldValue = (part as any)[cond.field];
  }

  switch (cond.operator) {
    case "eq":
      return fieldValue === cond.value;
    case "neq":
      return fieldValue !== cond.value;
    case "contains":
      return typeof fieldValue === "string" && fieldValue.toLowerCase().includes(String(cond.value || "").toLowerCase());
    case "notContains":
      return typeof fieldValue === "string" && !fieldValue.toLowerCase().includes(String(cond.value || "").toLowerCase());
    case "startsWith":
      return typeof fieldValue === "string" && fieldValue.toLowerCase().startsWith(String(cond.value || "").toLowerCase());
    case "endsWith":
      return typeof fieldValue === "string" && fieldValue.toLowerCase().endsWith(String(cond.value || "").toLowerCase());
    case "gt":
      return Number(fieldValue) > Number(cond.value);
    case "gte":
      return Number(fieldValue) >= Number(cond.value);
    case "lt":
      return Number(fieldValue) < Number(cond.value);
    case "lte":
      return Number(fieldValue) <= Number(cond.value);
    case "between": {
      const n = Number(fieldValue);
      const v1 = Number(cond.value);
      const v2 = Number(cond.valueTo);
      if (isNaN(v1) || isNaN(v2)) return true;
      return n >= Math.min(v1, v2) && n <= Math.max(v1, v2);
    }
    case "in": {
      const arr = Array.isArray(cond.value) ? cond.value : [];
      return arr.includes(fieldValue);
    }
    case "notIn": {
      const arr = Array.isArray(cond.value) ? cond.value : [];
      return !arr.includes(fieldValue);
    }
    case "isNull":
      return fieldValue === null || fieldValue === undefined || fieldValue === "";
    case "isNotNull":
      return fieldValue !== null && fieldValue !== undefined && fieldValue !== "";
    case "true":
      return !!fieldValue;
    case "false":
      return !fieldValue;
    default:
      return true;
  }
}

function evalGroup(part: Part, group: AdvancedFilterGroup): boolean {
  const results: boolean[] = [];
  for (const cond of group.conditions) {
    results.push(evalCondition(part, cond));
  }
  if (group.groups) {
    for (const sub of group.groups) {
      results.push(evalGroup(part, sub));
    }
  }
  if (results.length === 0) return true;
  return group.logic === "AND"
    ? results.every((r) => r)
    : results.some((r) => r);
}

function applyBasicFilter(parts: Part[], filter: PartFilter): Part[] {
  const masterDataStore = useMasterDataStore();
  let result = parts;

  if (filter.keyword) {
    const keyword = filter.keyword.toLowerCase();
    result = result.filter(
      (p) =>
        p.name.toLowerCase().includes(keyword) ||
        p.partNumber.toLowerCase().includes(keyword) ||
        (p.description?.toLowerCase().includes(keyword) ?? false)
    );
  }

  if (filter.types && filter.types.length > 0) {
    const logic: LogicMode = filter.typeLogic || "OR";
    result = result.filter((p) =>
      logic === "OR"
        ? filter.types!.includes(p.type)
        : filter.types!.every((t) => p.type === t)
    );
  } else if (filter.type) {
    result = result.filter((p) => p.type === filter.type);
  }

  if (filter.colors && filter.colors.length > 0) {
    const logic: LogicMode = filter.colorLogic || "OR";
    result = result.filter((p) =>
      logic === "OR"
        ? filter.colors!.includes(p.color)
        : filter.colors!.every((c) => p.color === c)
    );
  } else if (filter.color) {
    result = result.filter((p) => p.color === filter.color);
  }

  if (filter.sizes && filter.sizes.length > 0) {
    const logic: LogicMode = filter.sizeLogic || "OR";
    result = result.filter((p) =>
      logic === "OR"
        ? filter.sizes!.includes(p.size)
        : filter.sizes!.every((s) => p.size === s)
    );
  } else if (filter.size) {
    result = result.filter((p) => p.size === filter.size);
  }

  if (filter.location) {
    const locationCodes = new Set(
      masterDataStore.getAllChildLocationCodesByCode(filter.location)
    );
    result = result.filter((p) => locationCodes.has(p.location));
  }

  if (filter.quantityMin !== undefined) {
    result = result.filter((p) => p.quantity >= filter.quantityMin!);
  }
  if (filter.quantityMax !== undefined) {
    result = result.filter((p) => p.quantity <= filter.quantityMax!);
  }

  if (filter.hasImage === true) {
    result = result.filter((p) => !!p.imagePath);
  } else if (filter.hasImage === false) {
    result = result.filter((p) => !p.imagePath);
  }

  if (filter.createdAtDynamic) {
    const range = resolveDynamicTimeRange(filter.createdAtDynamic);
    if (range) {
      result = result.filter(
        (p) =>
          parseISODate(p.createdAt).getTime() >= range.from.getTime() &&
          parseISODate(p.createdAt).getTime() <= range.to.getTime()
      );
    }
  } else if (filter.createdAtFrom || filter.createdAtTo) {
    result = result.filter((p) =>
      dateInRange(p.createdAt, filter.createdAtFrom, filter.createdAtTo)
    );
  }

  if (filter.updatedAtDynamic) {
    const range = resolveDynamicTimeRange(filter.updatedAtDynamic);
    if (range) {
      result = result.filter(
        (p) =>
          parseISODate(p.updatedAt).getTime() >= range.from.getTime() &&
          parseISODate(p.updatedAt).getTime() <= range.to.getTime()
      );
    }
  } else if (filter.updatedAtFrom || filter.updatedAtTo) {
    result = result.filter((p) =>
      dateInRange(p.updatedAt, filter.updatedAtFrom, filter.updatedAtTo)
    );
  }

  return result;
}

export const usePartsStore = defineStore("parts", () => {
  const parts = ref<Part[]>([]);
  const loading = ref(false);
  const filter = ref<PartFilter>({});
  const sortField = ref<string>("");
  const sortOrder = ref<"asc" | "desc">("desc");

  const filteredParts = computed(() => {
    let result: Part[] = [...parts.value];

    if (filter.value.advancedMode && filter.value.advancedConditions) {
      result = result.filter((p) =>
        evalGroup(p, filter.value.advancedConditions!)
      );
    } else {
      result = applyBasicFilter(result, filter.value);
    }

    if (sortField.value) {
      result.sort((a, b) => {
        let va: any = (a as any)[sortField.value];
        let vb: any = (b as any)[sortField.value];
        if (typeof va === "string") va = va.toLowerCase();
        if (typeof vb === "string") vb = vb.toLowerCase();
        if (va < vb) return sortOrder.value === "asc" ? -1 : 1;
        if (va > vb) return sortOrder.value === "asc" ? 1 : -1;
        return 0;
      });
    }

    return result;
  });

  const totalQuantity = computed(() => {
    return filteredParts.value.reduce((sum, p) => sum + p.quantity, 0);
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

  function replaceFilter(newFilter: PartFilter) {
    filter.value = JSON.parse(JSON.stringify(newFilter || {}));
  }

  function clearFilter() {
    filter.value = {};
    sortField.value = "";
  }

  function setSort(field: string, order: "asc" | "desc") {
    sortField.value = field;
    sortOrder.value = order;
  }

  function applyView(view: {
    filter: PartFilter;
    sortField?: string;
    sortOrder?: "asc" | "desc";
  }) {
    replaceFilter(view.filter);
    if (view.sortField) {
      sortField.value = view.sortField;
      sortOrder.value = view.sortOrder || "desc";
    } else {
      sortField.value = "";
    }
  }

  return {
    parts,
    loading,
    filter,
    sortField,
    sortOrder,
    filteredParts,
    totalQuantity,
    lowStockParts,
    loadParts,
    addPart,
    updatePart,
    deletePart,
    getPartById,
    setFilter,
    replaceFilter,
    clearFilter,
    setSort,
    applyView,
  };
});
