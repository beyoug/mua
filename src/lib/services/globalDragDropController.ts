import { createLogger } from '$lib/utils/logger';

const logger = createLogger('GlobalDragDropController');

type Unlisten = () => void;

interface DragDropPayload {
    paths?: string[];
    position?: {
        x: number;
        y: number;
    };
}

export interface DragMeta {
    totalFiles: number;
    torrentFiles: number;
    hasSupportedFiles: boolean;
}

function buildDragMeta(paths?: string[]): DragMeta | null {
    if (!Array.isArray(paths) || paths.length === 0) {
        return null;
    }

    const torrentFiles = paths.filter((path) => path.toLowerCase().endsWith('.torrent')).length;
    return {
        totalFiles: paths.length,
        torrentFiles,
        hasSupportedFiles: torrentFiles > 0
    };
}

interface Params {
    onDragStateChange: (isDragOver: boolean) => void;
    onDragMetaChange: (meta: DragMeta | null) => void;
    onDragPulse: () => void;
    onDropPaths: (paths: string[], position?: { x: number; y: number }) => void;
}

export async function setupGlobalDragDrop(params: Params): Promise<Unlisten> {
    const { onDragStateChange, onDragMetaChange, onDragPulse, onDropPaths } = params;
    const unlisteners: Unlisten[] = [];

    try {
        const { listen } = await import('@tauri-apps/api/event');

        const dragEnterUnlisten = await listen<DragDropPayload>('tauri://drag-enter', (event) => {
            onDragStateChange(true);
            onDragMetaChange(buildDragMeta(event.payload.paths));
            onDragPulse();
        });
        unlisteners.push(dragEnterUnlisten);

        const dragOverUnlisten = await listen<DragDropPayload>('tauri://drag-over', () => {
            onDragPulse();
        });
        unlisteners.push(dragOverUnlisten);

        const dropUnlisten = await listen<DragDropPayload>('tauri://drag-drop', (event) => {
            onDragStateChange(false);
            onDragMetaChange(null);
            const paths = event.payload.paths;
            if (Array.isArray(paths) && paths.length > 0) {
                onDropPaths(paths, event.payload.position);
            }
        });
        unlisteners.push(dropUnlisten);

        const dragLeaveUnlisten = await listen('tauri://drag-leave', () => {
            onDragStateChange(false);
            onDragMetaChange(null);
        });
        unlisteners.push(dragLeaveUnlisten);
    } catch (e) {
        logger.error('Failed to register drag-drop handlers', { error: e });
    }

    return () => {
        for (const unlisten of unlisteners) {
            unlisten();
        }
    };
}
