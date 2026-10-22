import { invoke } from "@tauri-apps/api/core";

export type Group = "privacy" | "explorer" | "system";

export interface Loc {
  ru: string;
  en: string;
}

export interface TweakDto {
  id: string;
  group: Group;
  title: Loc;
  description: Loc;
  details: string[];
  requires_admin: boolean;
  win11_only: boolean;
}

export interface SystemInfo {
  is_admin: boolean;
  windows_product: string;
  display_version: string;
  build_number: string;
  arch: string;
}

export interface TweakResult {
  id: string;
  ok: boolean;
  error: string | null;
}

export type SnapshotMeta = [id: string, createdUnix: number, label: string, points: number];

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke("get_system_info");
}

export async function listTweaks(): Promise<TweakDto[]> {
  return invoke("list_tweaks");
}

export async function getTweaksState(ids: string[]): Promise<Record<string, boolean>> {
  return invoke("get_tweaks_state", { ids });
}

export async function applyTweaks(ids: string[]): Promise<TweakResult[]> {
  return invoke("apply_tweaks", { ids });
}

export async function revertTweaks(ids: string[]): Promise<TweakResult[]> {
  return invoke("revert_tweaks", { ids });
}

export async function listSnapshots(): Promise<SnapshotMeta[]> {
  return invoke("list_snapshots");
}

export async function restoreSnapshot(id?: string): Promise<number> {
  return invoke("restore_snapshot", { id: id ?? null });
}

export async function restartExplorer(): Promise<void> {
  return invoke("restart_explorer");
}

export async function createRestorePoint(): Promise<string> {
  return invoke("create_restore_point");
}

export async function emptyRecycleBin(): Promise<string> {
  return invoke("empty_recycle_bin");
}

export async function clearTemp(): Promise<number> {
  return invoke("clear_temp");
}

export async function runSfc(): Promise<number> {
  return invoke("run_sfc");
}

export async function runDism(): Promise<number> {
  return invoke("run_dism");
}

export interface UwpApp {
  name: string;
  full_name: string;
  version: string;
}

export interface UwpResult {
  full_name: string;
  ok: boolean;
  error: string | null;
}

export async function listUwp(): Promise<UwpApp[]> {
  return invoke("list_uwp");
}

export async function removeUwp(fullNames: string[]): Promise<UwpResult[]> {
  return invoke("remove_uwp", { fullNames });
}

export async function getMaxPowerState(): Promise<boolean> {
  return invoke("get_max_power_state");
}

export async function setMaxPower(enabled: boolean): Promise<void> {
  return invoke("set_max_power", { enabled });
}

export async function openUrl(url: string): Promise<void> {
  return invoke("open_url", { url });
}
