/**
 * aria2 JSON-RPC 类型定义
 */

/**
 * aria2 连接配置
 */
export interface Aria2Config {
    host: string;
    port: number;
    secret?: string;
    secure?: boolean;
}

/**
 * JSON-RPC 2.0 请求
 */
export interface JsonRpcRequest {
    jsonrpc: '2.0';
    id: string;
    method: string;
    params?: unknown[];
}

/**
 * JSON-RPC 2.0 响应
 */
export interface JsonRpcResponse<T = unknown> {
    jsonrpc: '2.0';
    id: string;
    result?: T;
    error?: JsonRpcError;
}

/**
 * JSON-RPC 错误
 */
export interface JsonRpcError {
    code: number;
    message: string;
    data?: unknown;
}

/**
 * aria2 下载状态
 */
export type DownloadStatusType =
    | 'active'
    | 'waiting'
    | 'paused'
    | 'error'
    | 'complete'
    | 'removed';

/**
 * aria2 文件信息
 */
export interface Aria2File {
    index: string;
    path: string;
    length: string;
    completedLength: string;
    selected: string;
    uris: Aria2Uri[];
}

/**
 * aria2 URI 信息
 */
export interface Aria2Uri {
    uri: string;
    status: 'used' | 'waiting';
}

/**
 * aria2 下载任务状态
 */
export interface DownloadStatus {
    gid: string;
    status: DownloadStatusType;
    totalLength: string;
    completedLength: string;
    uploadLength: string;
    downloadSpeed: string;
    uploadSpeed: string;
    connections: string;
    numSeeders?: string;
    seeder?: string;
    infoHash?: string;
    dir: string;
    files: Aria2File[];
    bittorrent?: Aria2Bittorrent;
    errorCode?: string;
    errorMessage?: string;
}

/**
 * aria2 BitTorrent 信息
 */
export interface Aria2Bittorrent {
    announceList?: string[][];
    comment?: string;
    creationDate?: number;
    mode?: 'single' | 'multi';
    info?: {
        name: string;
    };
}

/**
 * aria2 全局状态
 */
export interface GlobalStat {
    downloadSpeed: string;
    uploadSpeed: string;
    numActive: string;
    numWaiting: string;
    numStopped: string;
    numStoppedTotal: string;
}

/**
 * aria2 版本信息
 */
export interface VersionInfo {
    version: string;
    enabledFeatures: string[];
}
