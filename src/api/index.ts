import { invoke } from "@tauri-apps/api/core";
import type {
  Part,
  PartType,
  PartColor,
  PartSize,
  Location,
  MocList,
  PartFilter,
  StatsData,
} from "@/types";

function isTauriAvailable(): boolean {
  return (
    typeof window !== "undefined" &&
    (window as any).__TAURI_INTERNALS__ !== undefined
  );
}

function wrapInvoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauriAvailable()) {
    console.warn(`[mock] Called ${command} - returning mock data`);
    return getMockData(command, args) as Promise<T>;
  }
  return invoke(command, args);
}

function getMockData(command: string, args?: Record<string, unknown>): unknown {
  const now = new Date().toISOString();
  const mockStats: StatsData = {
    totalParts: 127,
    totalQuantity: 3456,
    totalTypes: 12,
    totalColors: 15,
    totalLocations: 5,
    totalMocs: 3,
    lowStockParts: 8,
    missingPartsInMocs: 12,
    partsByType: [
      { name: "砖类", count: 42 },
      { name: "板类", count: 35 },
      { name: "杆类", count: 18 },
      { name: "齿轮", count: 12 },
      { name: "连接件", count: 10 },
      { name: "其他", count: 10 },
    ],
    partsByColor: [
      { name: "红色", count: 52, hex: "#E53935" },
      { name: "蓝色", count: 48, hex: "#1E88E5" },
      { name: "黄色", count: 42, hex: "#FDD835" },
      { name: "绿色", count: 38, hex: "#43A047" },
      { name: "白色", count: 32, hex: "#FFFFFF" },
      { name: "黑色", count: 28, hex: "#212121" },
      { name: "灰色", count: 22, hex: "#9E9E9E" },
      { name: "橙色", count: 18, hex: "#FB8C00" },
    ],
    partsByLocation: [
      { name: "收纳盒 A", count: 68 },
      { name: "收纳盒 B", count: 42 },
      { name: "抽屉 1", count: 12 },
      { name: "抽屉 2", count: 3 },
      { name: "展示架", count: 2 },
    ],
  };

  const mockParts: Part[] = [
    {
      id: "1",
      name: "2x4 基础砖",
      partNumber: "3001",
      type: "砖类",
      color: "红色",
      size: "标准",
      quantity: 150,
      location: "收纳盒 A",
      description: "经典红色 2x4 基础砖",
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "2",
      name: "1x2 板",
      partNumber: "3023",
      type: "板类",
      color: "蓝色",
      size: "薄款",
      quantity: 200,
      location: "收纳盒 A",
      description: "蓝色 1x2 薄板",
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "3",
      name: "十字轴",
      partNumber: "3705",
      type: "杆类",
      color: "黑色",
      size: "4M",
      quantity: 50,
      location: "收纳盒 B",
      description: "4单位长十字轴",
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "4",
      name: "8齿齿轮",
      partNumber: "3647",
      type: "齿轮",
      color: "浅灰",
      size: "标准",
      quantity: 25,
      location: "抽屉 1",
      description: "8齿小齿轮",
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "5",
      name: "2x2 斜面砖",
      partNumber: "3039",
      type: "砖类",
      color: "黄色",
      size: "标准",
      quantity: 80,
      location: "收纳盒 A",
      description: "2x2 45度斜面砖",
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "6",
      name: "1x1 圆砖",
      partNumber: "4073",
      type: "砖类",
      color: "绿色",
      size: "标准",
      quantity: 120,
      location: "收纳盒 B",
      createdAt: now,
      updatedAt: now,
    },
  ];

  const mockTypes: PartType[] = [
    { id: "1", name: "砖类", code: "BRICK", description: "各种规格的基础砖" },
    { id: "2", name: "板类", code: "PLATE", description: "薄板类零件" },
    { id: "3", name: "杆类", code: "ROD", description: "轴、杆类零件" },
    { id: "4", name: "齿轮", code: "GEAR", description: "各种齿轮" },
    { id: "5", name: "连接件", code: "CONNECTOR", description: "连接销、卡扣等" },
    { id: "6", name: "其他", code: "OTHER", description: "其他类型零件" },
  ];

  const mockColors: PartColor[] = [
    { id: "1", name: "红色", hex: "#E53935", legoCode: "21" },
    { id: "2", name: "蓝色", hex: "#1E88E5", legoCode: "23" },
    { id: "3", name: "黄色", hex: "#FDD835", legoCode: "24" },
    { id: "4", name: "绿色", hex: "#43A047", legoCode: "28" },
    { id: "5", name: "白色", hex: "#FFFFFF", legoCode: "1" },
    { id: "6", name: "黑色", hex: "#212121", legoCode: "26" },
    { id: "7", name: "浅灰", hex: "#9E9E9E", legoCode: "199" },
    { id: "8", name: "深灰", hex: "#616161", legoCode: "194" },
    { id: "9", name: "橙色", hex: "#FB8C00", legoCode: "106" },
  ];

  const mockSizes: PartSize[] = [
    { id: "1", name: "标准", width: 1.0, height: 1.0, unit: "砖" },
    { id: "2", name: "薄款", width: 1.0, height: 0.33, unit: "砖" },
    { id: "3", name: "2M", width: 2.0, height: 1.0, unit: "单位" },
    { id: "4", name: "4M", width: 4.0, height: 1.0, unit: "单位" },
  ];

  const mockLocations: Location[] = [
    { id: "1", name: "收纳盒 A", code: "BOX_A", description: "主收纳盒 - 常用零件" },
    { id: "2", name: "收纳盒 B", code: "BOX_B", description: "主收纳盒 - 备用零件" },
    { id: "3", name: "抽屉 1", code: "DRAWER_1", description: "工作台抽屉1" },
    { id: "4", name: "抽屉 2", code: "DRAWER_2", description: "工作台抽屉2" },
    { id: "5", name: "展示架", code: "DISPLAY", description: "展示架上的模型" },
  ];

  const mockMocs: MocList[] = [
    {
      id: "1",
      name: "迷你机器人",
      description: "一个可爱的小型机器人 MOC",
      parts: [
        { partId: "1", partNumber: "3001", partName: "2x4 基础砖", color: "红色", quantity: 10, inStock: 150, isMissing: false },
        { partId: "2", partNumber: "3023", partName: "1x2 板", color: "蓝色", quantity: 20, inStock: 200, isMissing: false },
        { partId: "100", partNumber: "9999", partName: "特殊零件", color: "紫色", quantity: 5, inStock: 0, isMissing: true },
      ],
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "2",
      name: "小汽车",
      description: "经典 LEGO 小汽车 MOC",
      parts: [
        { partId: "3", partNumber: "3705", partName: "十字轴", color: "黑色", quantity: 4, inStock: 50, isMissing: false },
        { partId: "4", partNumber: "3647", partName: "8齿齿轮", color: "浅灰", quantity: 8, inStock: 25, isMissing: false },
      ],
      createdAt: now,
      updatedAt: now,
    },
    {
      id: "3",
      name: "小房子",
      description: "温馨的小房子 MOC",
      parts: [
        { partId: "1", partNumber: "3001", partName: "2x4 基础砖", color: "红色", quantity: 50, inStock: 150, isMissing: false },
        { partId: "5", partNumber: "3039", partName: "2x2 斜面砖", color: "黄色", quantity: 30, inStock: 80, isMissing: false },
        { partId: "200", partNumber: "8888", partName: "屋顶零件", color: "棕色", quantity: 12, inStock: 0, isMissing: true },
      ],
      createdAt: now,
      updatedAt: now,
    },
  ];

  switch (command) {
    case "init_database":
      return Promise.resolve();
    case "get_encryption_key":
      return Promise.resolve("mock-encryption-key");
    case "get_stats":
      return Promise.resolve(mockStats);
    case "get_parts": {
      const filter = args?.filter as PartFilter | undefined;
      let result = [...mockParts];
      if (filter?.keyword) {
        result = result.filter(p =>
          p.name.toLowerCase().includes(filter.keyword!.toLowerCase()) ||
          p.partNumber.toLowerCase().includes(filter.keyword!.toLowerCase())
        );
      }
      if (filter?.type) {
        result = result.filter(p => p.type === filter.type);
      }
      return Promise.resolve(result);
    }
    case "get_part_by_id":
      return Promise.resolve(mockParts[0]);
    case "get_part_types":
      return Promise.resolve(mockTypes);
    case "get_part_colors":
      return Promise.resolve(mockColors);
    case "get_part_sizes":
      return Promise.resolve(mockSizes);
    case "get_locations":
      return Promise.resolve(mockLocations);
    case "get_moc_lists":
      return Promise.resolve(mockMocs);
    case "get_moc_list_by_id":
      return Promise.resolve(mockMocs[0]);
    case "compare_moc_inventory":
      return Promise.resolve(mockMocs[0]);
    case "export_parts":
      return Promise.resolve("mock-export-data");
    case "import_parts":
      return Promise.resolve({ imported: 10, errors: [] });
    case "save_part_image":
      return Promise.resolve("mock-image-path");
    case "get_part_image_path":
      return Promise.resolve(null);
    default:
      return Promise.resolve({});
  }
}

export const api = {
  async initDatabase(): Promise<void> {
    return wrapInvoke("init_database");
  },

  async getEncryptionKey(): Promise<string> {
    return wrapInvoke("get_encryption_key");
  },

  async changeEncryptionKey(oldKey: string, newKey: string): Promise<void> {
    return wrapInvoke("change_encryption_key", { oldKey, newKey });
  },

  async getParts(filter?: PartFilter): Promise<Part[]> {
    return wrapInvoke("get_parts", { filter });
  },

  async getPartById(id: string): Promise<Part | null> {
    return wrapInvoke("get_part_by_id", { id });
  },

  async createPart(part: Omit<Part, "id" | "createdAt" | "updatedAt">): Promise<Part> {
    return wrapInvoke("create_part", { part });
  },

  async updatePart(part: Part): Promise<Part> {
    return wrapInvoke("update_part", { part });
  },

  async deletePart(id: string): Promise<void> {
    return wrapInvoke("delete_part", { id });
  },

  async getPartTypes(): Promise<PartType[]> {
    return wrapInvoke("get_part_types");
  },

  async createPartType(type: Omit<PartType, "id">): Promise<PartType> {
    return wrapInvoke("create_part_type", { type });
  },

  async updatePartType(type: PartType): Promise<PartType> {
    return wrapInvoke("update_part_type", { type });
  },

  async deletePartType(id: string): Promise<void> {
    return wrapInvoke("delete_part_type", { id });
  },

  async getPartColors(): Promise<PartColor[]> {
    return wrapInvoke("get_part_colors");
  },

  async createPartColor(color: Omit<PartColor, "id">): Promise<PartColor> {
    return wrapInvoke("create_part_color", { color });
  },

  async updatePartColor(color: PartColor): Promise<PartColor> {
    return wrapInvoke("update_part_color", { color });
  },

  async deletePartColor(id: string): Promise<void> {
    return wrapInvoke("delete_part_color", { id });
  },

  async getPartSizes(): Promise<PartSize[]> {
    return wrapInvoke("get_part_sizes");
  },

  async createPartSize(size: Omit<PartSize, "id">): Promise<PartSize> {
    return wrapInvoke("create_part_size", { size });
  },

  async updatePartSize(size: PartSize): Promise<PartSize> {
    return wrapInvoke("update_part_size", { size });
  },

  async deletePartSize(id: string): Promise<void> {
    return wrapInvoke("delete_part_size", { id });
  },

  async getLocations(): Promise<Location[]> {
    return wrapInvoke("get_locations");
  },

  async createLocation(location: Omit<Location, "id">): Promise<Location> {
    return wrapInvoke("create_location", { location });
  },

  async updateLocation(location: Location): Promise<Location> {
    return wrapInvoke("update_location", { location });
  },

  async deleteLocation(id: string): Promise<void> {
    return wrapInvoke("delete_location", { id });
  },

  async getMocLists(): Promise<MocList[]> {
    return wrapInvoke("get_moc_lists");
  },

  async getMocListById(id: string): Promise<MocList | null> {
    return wrapInvoke("get_moc_list_by_id", { id });
  },

  async createMocList(moc: Omit<MocList, "id" | "createdAt" | "updatedAt">): Promise<MocList> {
    return wrapInvoke("create_moc_list", { moc });
  },

  async updateMocList(moc: MocList): Promise<MocList> {
    return wrapInvoke("update_moc_list", { moc });
  },

  async deleteMocList(id: string): Promise<void> {
    return wrapInvoke("delete_moc_list", { id });
  },

  async compareMocInventory(mocId: string): Promise<MocList> {
    return wrapInvoke("compare_moc_inventory", { mocId });
  },

  async getStats(): Promise<StatsData> {
    return wrapInvoke("get_stats");
  },

  async exportParts(format: "json" | "csv", partIds?: string[]): Promise<string> {
    return wrapInvoke("export_parts", { format, partIds });
  },

  async importParts(format: "json" | "csv", data: string): Promise<{ imported: number; errors: string[] }> {
    return wrapInvoke("import_parts", { format, data });
  },

  async savePartImage(partId: string, imageData: string): Promise<string> {
    return wrapInvoke("save_part_image", { partId, imageData });
  },

  async deletePartImage(partId: string): Promise<void> {
    return wrapInvoke("delete_part_image", { partId });
  },

  async getPartImagePath(partId: string): Promise<string | null> {
    return wrapInvoke("get_part_image_path", { partId });
  },
};
