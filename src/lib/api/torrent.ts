import { invoke } from '@tauri-apps/api/core';
import type { TorrentInfo } from '$lib/types/torrent';

export async function parseTorrent(path: string): Promise<TorrentInfo> {
    return invoke<TorrentInfo>('parse_torrent', { path });
}
