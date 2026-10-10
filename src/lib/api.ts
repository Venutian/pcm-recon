import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
import type { SaveData, CellEdit, EditResponse, BackupInfo, SaveFile, TableSummary } from "./types";
import { save, isLoading, loadStatus, shortlist, notes, compareIds, lastSavePath, toast, selectedId } from "./stores";
import type { Recipe } from "./kits/render";

const inTauri = typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

/** In a plain browser (vite dev without Tauri) the app reads a JSON dump so the UI can be previewed. */
async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (inTauri) return tauriInvoke<T>(cmd, args);
  return browserMock<T>(cmd, args);
}

/** Commands that send raw bytes (images) instead of JSON arguments. */
async function invokeRaw<T>(cmd: string, body: Uint8Array, headers: Record<string, string>): Promise<T> {
  if (inTauri) return tauriInvoke<T>(cmd, body, { headers });
  return browserMock<T>(cmd, { body, headers });
}

// Preview-only kit backend: serves the sample kit files in `.kit-samples/` and keeps edits in memory.
const previewKitEdits = new Map<string, { recipe: Recipe; img: ImageData; modified_ms: number }>();
const previewMeta = new Map<string, string>();
async function previewKitImage(kit: string, part: string): Promise<ArrayBuffer> {
  const res = await fetch(`/.kit-samples/${kit}_${part}.uexp`);
  if (!res.ok) throw new Error(`Preview has no sample for ${kit} ${part}`);
  const u = new Uint8Array(await res.arrayBuffer());
  const dv = new DataView(u.buffer);
  const w = dv.getUint32(0x50, true), h = dv.getUint32(0x54, true);
  const out = new Uint8Array(8 + w * h * 4);
  new DataView(out.buffer).setUint32(0, w, true);
  new DataView(out.buffer).setUint32(4, h, true);
  for (let i = 0; i < w * h * 4; i += 4) {
    out[8 + i] = u[0x78 + i + 2]; out[9 + i] = u[0x78 + i + 1]; out[10 + i] = u[0x78 + i]; out[11 + i] = u[0x78 + i + 3];
  }
  return out.buffer;
}

async function browserMock<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const mockUrl = import.meta.env.VITE_MOCK_SAVE as string | undefined;
  switch (cmd) {
    case "kit_status": {
      const jersey = get(save)?.teams.find((t) => t.is_mine)?.jersey ?? "";
      const parts = ["maillot", "maillot_tour", "minimaillot", "corsair", "shoes"];
      return { game: "PCM 2026", mod_label: "WorldDB 2026 (preview)", oodle_ready: true, override_path: "preview", override_installed: false, override_modified_ms: 0, kits: [{ abbr: jersey, from_mod: true, parts }] } as T;
    }
    case "kit_image": {
      const { kit, part, edited } = args as { kit: string; part: string; edited: boolean };
      const e = previewKitEdits.get(`${kit}|${part}`);
      if (edited && e) {
        const out = new Uint8Array(8 + e.img.data.length);
        new DataView(out.buffer).setUint32(0, e.img.width, true);
        new DataView(out.buffer).setUint32(4, e.img.height, true);
        out.set(e.img.data, 8);
        return out.buffer as T;
      }
      return (await previewKitImage(kit, part)) as T;
    }
    case "kit_edits":
      return [...previewKitEdits.entries()].map(([k, e]) => ({ kit: k.split("|")[0], part: k.split("|")[1], recipe: e.recipe, modified_ms: e.modified_ms })) as T;
    case "kit_save_edit": {
      const { body, headers } = args as { body: Uint8Array; headers: Record<string, string> };
      const len = new DataView(body.buffer, body.byteOffset).getUint32(0, true);
      const recipe = JSON.parse(new TextDecoder().decode(body.subarray(4, 4 + len)));
      const img = new ImageData(new Uint8ClampedArray(body.subarray(4 + len)), +headers.width, +headers.height);
      previewKitEdits.set(`${headers.kit}|${headers.part}`, { recipe, img, modified_ms: Date.now() });
      return undefined as T;
    }
    case "kit_delete_edit":
      previewKitEdits.delete(`${args!.kit}|${args!.part}`);
      return undefined as T;
    case "kit_meta_get":
      return (previewMeta.get(String(args!.key)) ?? null) as T;
    case "kit_meta_set":
      if (args!.value == null) previewMeta.delete(String(args!.key)); else previewMeta.set(String(args!.key), String(args!.value));
      return undefined as T;
    case "kit_apply":
      return { parts: previewKitEdits.size, path: "preview", skipped: [] } as T;
    case "kit_squad_nations": {
      const counts = new Map<string, { code: string; name: string; flag: string; riders: number }>();
      for (const c of get(save)?.cyclists ?? []) {
        if (c.team_id !== args!.teamId || !c.flag) continue;
        const n = counts.get(c.flag) ?? { code: c.iso, name: c.nationality, flag: c.flag, riders: 0 };
        n.riders++;
        counts.set(c.flag, n);
      }
      return [...counts.values()].sort((a, b) => b.riders - a.riders) as T;
    }
    case "load_save": {
      if (!mockUrl) throw new Error("Preview mode: set VITE_MOCK_SAVE to a JSON dump");
      const data = await (await fetch(mockUrl)).json();
      return { ...data, modified_ms: Date.now() } as T;
    }
    case "find_saves":
      return (mockUrl ? [{ path: "preview.cdb", name: "Career_1.cdb", game: "PCM 2026", modified_ms: Date.now() - 3600e3, size: 5_700_000 }] : []) as T;
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

// ─── Kits ─────────────────────────────────────────────────────────────────────

export interface KitSummary { abbr: string; from_mod: boolean; parts: string[]; }
export interface KitStatus { game: string; mod_label: string | null; oodle_ready: boolean; override_path: string; override_installed: boolean; override_modified_ms: number; kits: KitSummary[]; }
export interface KitEdit { kit: string; part: string; recipe: Recipe; modified_ms: number; }
export interface Nation { code: string; name: string; flag: string; riders: number; }
export interface ApplyReport { parts: number; path: string; skipped: string[]; }

const savePath = () => get(save)!.meta.path;

export const kitStatus = () => invoke<KitStatus>("kit_status", { savePath: savePath() });
export const kitOodleDownload = () => invoke<void>("kit_oodle_download");
export const kitOodlePick = (path: string) => invoke<void>("kit_oodle_pick", { path });
export const kitEdits = (kit?: string) => invoke<KitEdit[]>("kit_edits", { kit: kit ?? null });
export const kitDeleteEdit = (kit: string, part: string) => invoke<void>("kit_delete_edit", { kit, part });
export const kitApply = () => invoke<ApplyReport>("kit_apply", { savePath: savePath() });
export const kitRemove = () => invoke<void>("kit_remove", { savePath: savePath() });
export const kitMetaGet = (key: string) => invoke<string | null>("kit_meta_get", { key });
export const kitMetaSet = (key: string, value: string | null) => invoke<void>("kit_meta_set", { key, value });
export const kitSquadNations = (teamId: number) => invoke<Nation[]>("kit_squad_nations", { savePath: savePath(), teamId });

/** A kit part as ImageData; `edited` returns the user's saved version. */
export async function kitImage(kit: string, part: string, edited = false): Promise<ImageData> {
  const buf = await invoke<ArrayBuffer>("kit_image", { savePath: savePath(), kit, part, edited });
  const dv = new DataView(buf);
  const w = dv.getUint32(0, true), h = dv.getUint32(4, true);
  return new ImageData(new Uint8ClampedArray(buf, 8, w * h * 4), w, h);
}

export async function kitSaveEdit(kit: string, part: string, recipe: Recipe, img: ImageData): Promise<void> {
  const json = new TextEncoder().encode(JSON.stringify(recipe));
  const body = new Uint8Array(4 + json.length + img.data.length);
  new DataView(body.buffer).setUint32(0, json.length, true);
  body.set(json, 4);
  body.set(img.data, 4 + json.length);
  await invokeRaw("kit_save_edit", body, { kit, part, width: String(img.width), height: String(img.height) });
}

export async function setTeamKit(teamId: number, jersey: string, color1: string, color2: string): Promise<void> {
  const res = await invoke<EditResponse>("set_team_kit", { path: savePath(), teamId, jersey, color1, color2 });
  save.set(res);
}

export const readFileBytes = (path: string) => invoke<ArrayBuffer>("read_file_bytes", { path });
export const writePng = (path: string, bytes: Uint8Array) => invokeRaw<void>("write_png", bytes, { path: encodeURIComponent(path) });

export async function pickFile(name: string, extensions: string[]): Promise<string | null> {
  const { open } = await import("@tauri-apps/plugin-dialog");
  return ((await open({ filters: [{ name, extensions }] }).catch(() => null)) as string | null) ?? null;
}

export async function pickSavePath(defaultPath: string): Promise<string | null> {
  const { save: saveDialog } = await import("@tauri-apps/plugin-dialog");
  return (await saveDialog({ filters: [{ name: "PNG image", extensions: ["png"] }], defaultPath }).catch(() => null)) ?? null;
}

/** Lets the user pick an image; in the browser preview a file input stands in for the dialog. */
export async function pickImage(): Promise<{ name: string; bytes: ArrayBuffer } | null> {
  if (inTauri) {
    const path = await pickFile("Image", ["png", "jpg", "jpeg", "webp", "svg"]);
    return path ? { name: path, bytes: await readFileBytes(path) } : null;
  }
  return new Promise((ok) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = "image/*";
    input.onchange = async () => {
      const f = input.files?.[0];
      ok(f ? { name: f.name, bytes: await f.arrayBuffer() } : null);
    };
    input.click();
  });
}
