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
}

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
