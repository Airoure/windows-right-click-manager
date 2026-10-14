import { invoke, isTauri } from '@tauri-apps/api/core';
import type { AppData, NewMenu, CapturedMenu } from './types';

export const native = isTauri();
export const api = {
  load: () => invoke<AppData>('load_data'),
  capture: () => invoke<CapturedMenu>('capture_menu'),
  toggle: (id: string, enabled: boolean) => invoke<void>('toggle_menu', { id, enabled }),
  create: (menu: NewMenu) => invoke<void>('add_menu', { menu }),
  restore: (id: string) => invoke<void>('restore_operation', { id }),
};
