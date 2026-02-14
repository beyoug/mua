export interface TorrentInfo {
    name: string;
    files: TorrentFile[];
    total_length: number;
}

export interface TorrentFile {
    path: string;
    length: number;
    index: number;
}
