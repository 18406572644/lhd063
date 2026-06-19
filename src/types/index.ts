export interface Part {
  id: string;
  name: string;
  partNumber: string;
  type: string;
  color: string;
  size: string;
  quantity: number;
  location: string;
  description?: string;
  imagePath?: string;
  createdAt: string;
  updatedAt: string;
}

export interface PartType {
  id: string;
  name: string;
  code: string;
  description?: string;
}

export interface PartColor {
  id: string;
  name: string;
  hex: string;
  legoCode?: string;
}

export interface PartSize {
  id: string;
  name: string;
  width: number;
  height: number;
  unit: string;
}

export interface Location {
  id: string;
  name: string;
  code: string;
  description?: string;
  parentId?: string;
}

export interface LocationTreeNode {
  id: string;
  name: string;
  code: string;
  description?: string;
  parentId?: string;
  children?: LocationTreeNode[];
}

export type MocStatus =
  | "planning"
  | "purchasing"
  | "parts_ready"
  | "building"
  | "completed"
  | "archived";

export const MOC_STATUS_OPTIONS: { value: MocStatus; label: string; color: string; type: "info" | "warning" | "primary" | "success" | "danger" }[] = [
  { value: "planning", label: "规划中", color: "#909399", type: "info" },
  { value: "purchasing", label: "采购中", color: "#E6A23C", type: "warning" },
  { value: "parts_ready", label: "零件齐套", color: "#67C23A", type: "success" },
  { value: "building", label: "搭建中", color: "#409EFF", type: "primary" },
  { value: "completed", label: "已完成", color: "#67C23A", type: "success" },
  { value: "archived", label: "已归档", color: "#909399", type: "info" },
];

export interface MocList {
  id: string;
  name: string;
  description?: string;
  coverImagePath?: string;
  status: MocStatus;
  parts: MocPart[];
  createdAt: string;
  updatedAt: string;
}

export interface MocStatusLog {
  id: string;
  mocId: string;
  oldStatus?: string;
  newStatus: string;
  changedAt: string;
  remark?: string;
}

export interface MocStatusChange {
  mocId: string;
  newStatus: MocStatus;
  remark?: string;
}

export interface MocPart {
  partId: string;
  partNumber: string;
  partName: string;
  color: string;
  quantity: number;
  inStock: number;
  isMissing: boolean;
}

export interface PartFilter {
  type?: string;
  color?: string;
  size?: string;
  location?: string;
  keyword?: string;
  quantityMin?: number;
  quantityMax?: number;
  createdAtFrom?: string;
  createdAtTo?: string;
  updatedAtFrom?: string;
  updatedAtTo?: string;
  createdAtDynamic?: DynamicTimeOption;
  updatedAtDynamic?: DynamicTimeOption;
  hasImage?: boolean | null;
  types?: string[];
  colors?: string[];
  sizes?: string[];
  typeLogic?: LogicMode;
  colorLogic?: LogicMode;
  sizeLogic?: LogicMode;
  advancedMode?: boolean;
  advancedConditions?: AdvancedFilterGroup;
}

export type LogicMode = "AND" | "OR";

export type DynamicTimeOption =
  | "today"
  | "yesterday"
  | "last7days"
  | "last30days"
  | "thisWeek"
  | "thisMonth"
  | "thisQuarter"
  | "thisYear"
  | "lastWeek"
  | "lastMonth";

export const DYNAMIC_TIME_OPTIONS: { value: DynamicTimeOption; label: string }[] = [
  { value: "today", label: "今天" },
  { value: "yesterday", label: "昨天" },
  { value: "last7days", label: "最近 7 天" },
  { value: "last30days", label: "最近 30 天" },
  { value: "thisWeek", label: "本周" },
  { value: "thisMonth", label: "本月" },
  { value: "thisQuarter", label: "本季度" },
  { value: "thisYear", label: "本年" },
  { value: "lastWeek", label: "上周" },
  { value: "lastMonth", label: "上月" },
];

export type FilterField =
  | "name"
  | "partNumber"
  | "type"
  | "color"
  | "size"
  | "quantity"
  | "location"
  | "description"
  | "createdAt"
  | "updatedAt"
  | "hasImage";

export type FilterOperator =
  | "eq"
  | "neq"
  | "contains"
  | "notContains"
  | "startsWith"
  | "endsWith"
  | "gt"
  | "gte"
  | "lt"
  | "lte"
  | "between"
  | "in"
  | "notIn"
  | "isNull"
  | "isNotNull"
  | "true"
  | "false";

export interface AdvancedFilterCondition {
  id: string;
  field: FilterField;
  operator: FilterOperator;
  value?: any;
  valueTo?: any;
}

export interface AdvancedFilterGroup {
  id: string;
  logic: LogicMode;
  conditions: AdvancedFilterCondition[];
  groups?: AdvancedFilterGroup[];
}

export const FILTER_FIELD_OPTIONS: { value: FilterField; label: string; type: "text" | "number" | "select" | "date" | "boolean" }[] = [
  { value: "name", label: "零件名称", type: "text" },
  { value: "partNumber", label: "零件编号", type: "text" },
  { value: "type", label: "零件类型", type: "select" },
  { value: "color", label: "颜色", type: "select" },
  { value: "size", label: "尺寸", type: "select" },
  { value: "quantity", label: "库存数量", type: "number" },
  { value: "location", label: "存放位置", type: "select" },
  { value: "description", label: "描述", type: "text" },
  { value: "createdAt", label: "创建时间", type: "date" },
  { value: "updatedAt", label: "更新时间", type: "date" },
  { value: "hasImage", label: "是否有图片", type: "boolean" },
];

export const FILTER_OPERATORS_BY_TYPE: Record<string, { value: FilterOperator; label: string }[]> = {
  text: [
    { value: "contains", label: "包含" },
    { value: "notContains", label: "不包含" },
    { value: "eq", label: "等于" },
    { value: "neq", label: "不等于" },
    { value: "startsWith", label: "开头是" },
    { value: "endsWith", label: "结尾是" },
    { value: "isNull", label: "为空" },
    { value: "isNotNull", label: "不为空" },
  ],
  number: [
    { value: "eq", label: "等于" },
    { value: "neq", label: "不等于" },
    { value: "gt", label: "大于" },
    { value: "gte", label: "大于等于" },
    { value: "lt", label: "小于" },
    { value: "lte", label: "小于等于" },
    { value: "between", label: "在...之间" },
  ],
  select: [
    { value: "eq", label: "等于" },
    { value: "neq", label: "不等于" },
    { value: "in", label: "包含任一" },
    { value: "notIn", label: "不包含" },
  ],
  date: [
    { value: "eq", label: "等于" },
    { value: "gt", label: "晚于" },
    { value: "gte", label: "晚于等于" },
    { value: "lt", label: "早于" },
    { value: "lte", label: "早于等于" },
    { value: "between", label: "在...之间" },
  ],
  boolean: [
    { value: "true", label: "是" },
    { value: "false", label: "否" },
  ],
};

export interface SavedView {
  id: string;
  name: string;
  isPreset?: boolean;
  isDefault?: boolean;
  filter: PartFilter;
  sortField?: string;
  sortOrder?: "asc" | "desc";
  visibleColumns?: string[];
  icon?: string;
  description?: string;
  createdAt?: string;
  updatedAt?: string;
}

export const PRESET_VIEWS: Omit<SavedView, "id" | "createdAt" | "updatedAt">[] = [
  {
    name: "全部零件",
    isPreset: true,
    isDefault: true,
    icon: "Grid",
    description: "显示所有零件",
    filter: {},
  },
  {
    name: "低库存",
    isPreset: true,
    icon: "Warning",
    description: "库存数量 ≤ 5 的零件",
    filter: {
      quantityMin: undefined,
      quantityMax: 5,
    },
  },
  {
    name: "最近添加",
    isPreset: true,
    icon: "Clock",
    description: "最近 7 天新增的零件",
    filter: {
      createdAtDynamic: "last7days",
    },
  },
  {
    name: "无图片",
    isPreset: true,
    icon: "Picture",
    description: "尚未上传图片的零件",
    filter: {
      hasImage: false,
    },
  },
];

export interface MocStatusCount {
  status: string;
  count: number;
}

export interface StatsData {
  totalParts: number;
  totalQuantity: number;
  totalTypes: number;
  totalColors: number;
  totalLocations: number;
  totalMocs: number;
  lowStockParts: number;
  missingPartsInMocs: number;
  partsByType: { name: string; count: number }[];
  partsByColor: { name: string; count: number; hex: string }[];
  partsByLocation: { name: string; count: number }[];
  mocsByStatus: MocStatusCount[];
}

export interface ImportExportPart {
  name: string;
  partNumber: string;
  type: string;
  color: string;
  size: string;
  quantity: number;
  location: string;
  description?: string;
}

export interface TypeCount {
  name: string;
  count: number;
}

export interface ColorCount {
  name: string;
  count: number;
  hex: string;
}

export interface LocationCount {
  name: string;
  count: number;
  code?: string;
  children?: LocationCount[];
}

export type OperationType = "create" | "update" | "delete";

export type ObjectType =
  | "part"
  | "part_type"
  | "part_color"
  | "part_size"
  | "location"
  | "moc_list";

export interface OperationLog {
  id: string;
  operationType: OperationType;
  objectType: ObjectType;
  objectId: string;
  objectName?: string;
  beforeSnapshot?: string;
  afterSnapshot?: string;
  changedAt: string;
}

export interface OperationLogFilter {
  operationType?: string;
  objectType?: string;
  objectId?: string;
}

export const OPERATION_TYPE_OPTIONS: { value: OperationType; label: string; type: "success" | "primary" | "danger" | "warning" }[] = [
  { value: "create", label: "新增", type: "success" },
  { value: "update", label: "修改", type: "primary" },
  { value: "delete", label: "删除", type: "danger" },
];

export const OBJECT_TYPE_OPTIONS: { value: ObjectType; label: string }[] = [
  { value: "part", label: "零件" },
  { value: "part_type", label: "零件类型" },
  { value: "part_color", label: "零件颜色" },
  { value: "part_size", label: "零件尺寸" },
  { value: "location", label: "存放位置" },
  { value: "moc_list", label: "MOC清单" },
];

export interface BackupInfo {
  filename: string;
  fileSize: number;
  createdAt: string;
  encrypted: boolean;
  version: string;
}

export interface BackupConfig {
  enabled: boolean;
  frequency: "daily" | "weekly";
  keepCount: number;
  encrypt: boolean;
}

export interface RestoreResult {
  success: boolean;
  mode: "full" | "merge";
  dbRestored: boolean;
  imagesRestored: number;
  keyRestored: boolean;
  message: string;
}

export interface IntegrityCheckResult {
  ok: boolean;
  errors: string[];
  canAutoRecover: boolean;
  latestBackup: BackupInfo | null;
}
