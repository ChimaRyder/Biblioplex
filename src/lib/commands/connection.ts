import { invoke } from "@tauri-apps/api/core";

export type ConnectionState = "unknown" | "checking" | "stable" | "unavailable";

export async function checkImageProvider(): Promise<ConnectionState> {
  try {
    return await invoke<ConnectionState>("check_image_provider");
  } catch {
    return "unavailable";
  }
}
