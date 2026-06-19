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
}
