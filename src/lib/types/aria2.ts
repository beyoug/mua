/**
 * aria2.ts - Aria2 Backend Types
 */

export interface Aria2Task {
    gid: string;
    status: string;
    totalLength: string;
    completedLength: string;
    downloadSpeed: string;
    uploadLength: string;
    uploadSpeed: string;
    errorCode: string | null;
    errorMessage: string | null;
    dir: string;
    files: Aria2File[];
}

export interface Aria2File {
    index: string;
    path: string;
    length: string;
    completedLength: string;
    selected: string;
    uris: Aria2Uri[];
}

export interface Aria2Uri {
    uri: string;
    status: string;
}
