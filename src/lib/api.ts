import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import type { SaveData, CellEdit, EditResponse, BackupInfo, SaveFile, TableSummary } from "./types";
import { save, isLoading, loadStatus, shortlist, notes, compareIds, lastSavePath, toast, selectedId } from "./stores";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** In a plain browser (vite dev without Tauri) the app reads a JSON dump so the UI can be previewed. */
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) return tauriInvoke<T>(cmd, args);
  return browserMock<T>(cmd, args);
}

async function browserMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const mockUrl = import.meta.env.VITE_MOCK_SAVE as string | undefined;
  switch (cmd) {
    case "load_save": {
      if (!mockUrl) throw new Error("Preview mode: set VITE_MOCK_SAVE to a JSON dump");
      const data = await (await fetch(mockUrl)).json();
      return { ...data, modified_ms: Date.now() } as T;
    }
    case "find_saves":
      return (mockUrl ? [{ path: "preview.cdb", name: "Career_2.cdb", game: "PCM 2026", modified_ms: Date.now() - 3600e3, size: 5_700_000 }] : []) as T;
    case "load_workspace": {
      try { return JSON.parse(localStorage.getItem("pcmrecon-ws") ?? "{}") as T; } catch { return {} as T; }
    }
    case "save_workspace":
      try { localStorage.setItem("pcmrecon-ws", JSON.stringify(args?.data)); } catch { /* preview only */ }
      return undefined as T;
    case "apply_edits": {
      const cur = get(save)!;
      const edits = args!.edits as CellEdit[];
      const next = structuredClone(cur);
      const outcomes = edits.map((e) => {
        let old = 0;
        if (e.table === "GAM_career_data" && next.finance) { old = next.finance.balance; next.finance.balance = e.value; }
        if (e.table === "DYN_team") {
          const t = next.teams.find((t) => t.id === e.key);
          if (t) { old = t.budget; t.budget = e.value; }
          if (next.finance && e.key === next.finance.team_id) next.finance.season_budget = e.value;
        }
        if (e.table === "DYN_team_sponsor" && next.finance) {
          if (e.column === "value_i_budget") { old = next.finance.sponsor_budget; next.finance.sponsor_budget = e.value; }
          else { old = next.finance.sponsor_budget_next; next.finance.sponsor_budget_next = e.value; }
        }
        return { table: e.table, column: e.column, key: String(e.key), old, new: e.value };
      });
      return { ...next, outcomes, backup: "preview-backup.cdb" } as T;
    }
    case "list_backups":
      return [] as T;
    case "save_modified":
      return (get(save)?.modified_ms ?? 0) as T;
    default:
      throw new Error(`${cmd} needs the desktop app`);
  }
}

export const isDesktop = inTauri;

// ─── Save loading ─────────────────────────────────────────────────────────────

export async function loadSave(path: string, quiet = false): Promise<boolean> {
  isLoading.set(true);
  loadStatus.set(`Reading ${path.split(/[\\/]/).pop()}…`);
  try {
    const data = await invoke<SaveData>("load_save", { path });
    const prevPath = get(save)?.meta.path;
    save.set(data);
    lastSavePath.set(data.meta.path);
    if (prevPath !== data.meta.path) selectedId.set(null);
    await importLegacyNotes(data.meta.path);
    persistWorkspace();
    if (!quiet) toast(`Loaded ${data.meta.file_name} · ${data.cyclists.length.toLocaleString()} riders`, "ok", 2800);
    return true;
  } catch (e) {
    toast(`Could not open the save: ${e}`, "error", 8000);
    return false;
  } finally {
    isLoading.set(false);
    loadStatus.set("");
  }
}

export async function reloadSave(): Promise<void> {
  const p = get(save)?.meta.path;
  if (p) await loadSave(p, true).then((ok) => ok && toast("Save reloaded from disk", "info", 2200));
}

export async function pickSave(): Promise<void> {
  if (!inTauri) return void toast("Opening files needs the desktop app", "info");
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = (await open({ filters: [{ name: "PCM career save", extensions: ["cdb"] }] }).catch(() => null)) as string | null;
  if (path) await loadSave(path);
}

export const findSaves = () => invoke<SaveFile[]>("find_saves");
export const saveModified = (path: string) => invoke<number>("save_modified", { path });

// ─── Editing ──────────────────────────────────────────────────────────────────

export async function applyEdits(edits: CellEdit[]): Promise<EditResponse> {
  const path = get(save)!.meta.path;
  const res = await invoke<EditResponse>("apply_edits", { path, edits });
  save.set(res);
  return res;
}

export const listBackups = () => invoke<BackupInfo[]>("list_backups", { path: get(save)!.meta.path });

export async function restoreBackup(backup: string): Promise<void> {
  const res = await invoke<SaveData>("restore_backup", { path: get(save)!.meta.path, backup });
  save.set(res);
}

export const openBackupFolder = () => invoke<void>("open_backup_folder", { path: get(save)!.meta.path });
export const dbTables = () => invoke<TableSummary[]>("db_tables", { path: get(save)!.meta.path });
export const dbPage = (table: string, offset: number, limit: number) =>
  invoke<(number | string | null)[][]>("db_page", { path: get(save)!.meta.path, table, offset, limit });

export async function exportCsv(rows: object[], fields: string[], defaultName: string): Promise<void> {
  if (!inTauri) return void toast("CSV export needs the desktop app", "info");
  const { save: saveDialog } = await import("@tauri-apps/plugin-dialog");
  const path = await saveDialog({ filters: [{ name: "CSV", extensions: ["csv"] }], defaultPath: defaultName }).catch(() => null);
  if (!path) return;
  await invoke("export_csv", { path, data: rows, fields });
  toast(`Exported ${rows.length} rows`);
}

export const openExternal = (url: string) => invoke<void>("open_external", { url });

// ─── Workspace persistence ────────────────────────────────────────────────────

interface Workspace { shortlist?: number[]; notes?: Record<string, string>; compare?: number[]; lastSave?: string; }

let ready = false;
let timer: ReturnType<typeof setTimeout> | undefined;

export async function loadWorkspace(): Promise<Workspace> {
  const ws = await invoke<Workspace>("load_workspace").catch(() => ({} as Workspace));
  shortlist.set(new Set(ws.shortlist ?? []));
  notes.set(ws.notes ?? {});
  compareIds.set(ws.compare ?? []);
  lastSavePath.set(ws.lastSave ?? "");
  ready = true;
  return ws;
}

export function persistWorkspace() {
  if (!ready) return;
  clearTimeout(timer);
  timer = setTimeout(() => {
    const ws: Workspace = {
      shortlist: [...get(shortlist)],
      notes: get(notes),
      compare: get(compareIds),
      lastSave: get(lastSavePath),
    };
    invoke("save_workspace", { data: ws }).catch(() => {});
  }, 400);
}

shortlist.subscribe(persistWorkspace);
notes.subscribe(persistWorkspace);
compareIds.subscribe(persistWorkspace);

/** v2.0 wrote notes beside the save; fold them in once. */
async function importLegacyNotes(savePath: string) {
  if (!inTauri) return;
  const dir = savePath.replace(/[^/\\]+$/, "");
  const legacy = await invoke<Record<string, string>>("load_notes", { path: dir + "pcm_recon_notes.json" }).catch(() => ({} as Record<string, string>));
  const keys = Object.keys(legacy);
  if (!keys.length) return;
  notes.update((n) => {
    const merged = { ...n };
    for (const k of keys) if (!merged[k] && legacy[k]) merged[k] = legacy[k];
    return merged;
  });
}

// ─── Name packs ───────────────────────────────────────────────────────────────

export interface PackInfo { key: string; label: string; first: string[]; last: string[]; }
export interface NameDb { game: string; path: string; current_first: string[]; current_last: string[]; }
export interface NamePacksInfo { packs: PackInfo[]; databases: NameDb[]; copies_folder: string; country_id: number; }
export interface ListReport { removed_first: string[]; removed_last: string[]; added_first: number; added_last: number; }
export interface Rename { id: number; from: string; to: string; }

export const namePacks = (pack: string) => invoke<NamePacksInfo>("name_packs", { pack, savePath: get(save)!.meta.path });
export const applyPackToGame = (pack: string, path: string, countryId: number) =>
  invoke<{ report: ListReport; backup: string; copy: string }>("apply_name_pack_to_game", { pack, path, countryId });

export async function applyPackToSave(pack: string): Promise<Rename[]> {
  const res = await invoke<SaveData & { renames: Rename[]; backup: string }>("apply_name_pack_to_save", { pack, path: get(save)!.meta.path });
  save.set(res);
  return res.renames;
}
