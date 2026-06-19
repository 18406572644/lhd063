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

export interface MocList {
  id: string;
  name: string;
  description?: string;
  parts: MocPart[];
  createdAt: string;
  updatedAt: string;
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
