import { invoke } from '@tauri-apps/api/core';

export interface PlatformInfo {
  os: string;
  arch: string;
}

export async function getPlatformInfo(): Promise<PlatformInfo> {
  return invoke<PlatformInfo>('get_platform_info');
}
