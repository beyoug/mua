import {
    fetchPublicTrackers,
    getAria2VersionInfo,
    importCustomBinary,
    startLogStream,
    stopLogStream
} from '$lib/api/aria2';
import type { Aria2VersionInfo } from '$lib/types/download';

export async function fetchTrackers(): Promise<string[]> {
    return fetchPublicTrackers();
}

export async function startAria2LogStream(): Promise<void> {
    await startLogStream();
}

export async function stopAria2LogStream(): Promise<void> {
    await stopLogStream();
}

export async function importAria2Binary(path: string): Promise<string> {
    return importCustomBinary(path);
}

export async function getAria2KernelVersionInfo(): Promise<Aria2VersionInfo> {
    return getAria2VersionInfo();
}
