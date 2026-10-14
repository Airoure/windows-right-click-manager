export type Scope = 'all' | 'file' | 'filesystem' | 'folder' | 'background' | 'drive';
export interface MenuEntry {
  id: string;
  name: string;
  scope: Scope;
  source: string;
  kind: 'static' | 'system' | 'extension';
  enabled: boolean;
  manageable: boolean;
  reason: string;
  command: string;
  registryPath: string;
  owner: string;
  requiresAdmin: boolean;
  extended: boolean;
}
export interface HistoryItem {
  id: string;
  timestamp: number;
  label: string;
  scope: Scope;
  operation: string;
  status: string;
  error: string | null;
  canRestore: boolean;
  requiresAdmin: boolean;
}
export interface AppData {
  scan: { entries: MenuEntry[]; warnings: string[]; scannedAt: number };
  history: HistoryItem[];
  storagePath: string;
}
export interface NewMenu {
  name: string;
  scope: Exclude<Scope, 'all' | 'filesystem'>;
  program: string;
  arguments: string;
}
