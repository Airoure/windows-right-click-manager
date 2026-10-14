import { invoke, isTauri } from '@tauri-apps/api/core';
import type { AppData, NewMenu } from './types';

export const native = isTauri();
export const api = {
  load: () => invoke<AppData>('load_data'),
  toggle: (id: string, enabled: boolean) => invoke<void>('toggle_menu', { id, enabled }),
  create: (menu: NewMenu) => invoke<void>('add_menu', { menu }),
  restore: (id: string) => invoke<void>('restore_operation', { id }),
};
