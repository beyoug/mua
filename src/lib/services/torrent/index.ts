import { parseTorrent } from '$lib/api/torrent';
import type { TorrentInfo } from '$lib/types/torrent';

export async function parseTorrentFile(path: string): Promise<TorrentInfo> {
    return parseTorrent(path);
}
