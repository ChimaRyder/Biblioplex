import { invoke } from "@tauri-apps/api/core";

export const backend = {
  status: () => invoke<string>("app_status"),
};
