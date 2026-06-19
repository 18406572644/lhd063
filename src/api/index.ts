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

export const api = {
  async initDatabase(): Promise<void> {
    return invoke("init_database");
  },

  async getEncryptionKey(): Promise<string> {
    return invoke("get_encryption_key");
  },

  async changeEncryptionKey(oldKey: string, newKey: string): Promise<void> {
    return invoke("change_encryption_key", { oldKey, newKey });
  },

  async getParts(filter?: PartFilter): Promise<Part[]> {
    return invoke("get_parts", { filter });
  },

  async getPartById(id: string): Promise<Part | null> {
    return invoke("get_part_by_id", { id });
  },

  async createPart(part: Omit<Part, "id" | "createdAt" | "updatedAt">): Promise<Part> {
    return invoke("create_part", { part });
  },

  async updatePart(part: Part): Promise<Part> {
    return invoke("update_part", { part });
  },

  async deletePart(id: string): Promise<void> {
    return invoke("delete_part", { id });
  },

  async getPartTypes(): Promise<PartType[]> {
    return invoke("get_part_types");
  },

  async createPartType(type: Omit<PartType, "id">): Promise<PartType> {
    return invoke("create_part_type", { type });
  },

  async updatePartType(type: PartType): Promise<PartType> {
    return invoke("update_part_type", { type });
  },

  async deletePartType(id: string): Promise<void> {
    return invoke("delete_part_type", { id });
  },

  async getPartColors(): Promise<PartColor[]> {
    return invoke("get_part_colors");
  },

  async createPartColor(color: Omit<PartColor, "id">): Promise<PartColor> {
    return invoke("create_part_color", { color });
  },

  async updatePartColor(color: PartColor): Promise<PartColor> {
    return invoke("update_part_color", { color });
  },

  async deletePartColor(id: string): Promise<void> {
    return invoke("delete_part_color", { id });
  },

  async getPartSizes(): Promise<PartSize[]> {
    return invoke("get_part_sizes");
  },

  async createPartSize(size: Omit<PartSize, "id">): Promise<PartSize> {
    return invoke("create_part_size", { size });
  },

  async updatePartSize(size: PartSize): Promise<PartSize> {
    return invoke("update_part_size", { size });
  },

  async deletePartSize(id: string): Promise<void> {
    return invoke("delete_part_size", { id });
  },

  async getLocations(): Promise<Location[]> {
    return invoke("get_locations");
  },

  async createLocation(location: Omit<Location, "id">): Promise<Location> {
    return invoke("create_location", { location });
  },

  async updateLocation(location: Location): Promise<Location> {
    return invoke("update_location", { location });
  },

  async deleteLocation(id: string): Promise<void> {
    return invoke("delete_location", { id });
  },

  async getMocLists(): Promise<MocList[]> {
    return invoke("get_moc_lists");
  },

  async getMocListById(id: string): Promise<MocList | null> {
    return invoke("get_moc_list_by_id", { id });
  },

  async createMocList(moc: Omit<MocList, "id" | "createdAt" | "updatedAt">): Promise<MocList> {
    return invoke("create_moc_list", { moc });
  },

  async updateMocList(moc: MocList): Promise<MocList> {
    return invoke("update_moc_list", { moc });
  },

  async deleteMocList(id: string): Promise<void> {
    return invoke("delete_moc_list", { id });
  },

  async compareMocInventory(mocId: string): Promise<MocList> {
    return invoke("compare_moc_inventory", { mocId });
  },

  async getStats(): Promise<StatsData> {
    return invoke("get_stats");
  },

  async exportParts(format: "json" | "csv", partIds?: string[]): Promise<string> {
    return invoke("export_parts", { format, partIds });
  },

  async importParts(format: "json" | "csv", data: string): Promise<{ imported: number; errors: string[] }> {
    return invoke("import_parts", { format, data });
  },

  async savePartImage(partId: string, imageData: string): Promise<string> {
    return invoke("save_part_image", { partId, imageData });
  },

  async deletePartImage(partId: string): Promise<void> {
    return invoke("delete_part_image", { partId });
  },

  async getPartImagePath(partId: string): Promise<string | null> {
    return invoke("get_part_image_path", { partId });
  },
};
