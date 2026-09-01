import { writable } from "svelte/store";
export const toast = writable<{ message: string; error?: boolean } | null>(null);
let timer: ReturnType<typeof setTimeout>;
export function showToast(message: string, error = false) { toast.set({ message, error }); clearTimeout(timer); timer = setTimeout(() => toast.set(null), 4500); }
