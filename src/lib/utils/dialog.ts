import { open as openDialog } from '@tauri-apps/plugin-dialog';

export type DialogSelection = string | { path?: string } | Array<string | { path?: string }> | null;

export interface DialogFileFilter {
    name: string;
    extensions: string[];
}

export function resolveSingleDialogPath(selection: DialogSelection): string | null {
    if (!selection) return null;
    if (typeof selection === 'string') return selection;

    if (Array.isArray(selection)) {
        const first = selection[0];
        if (!first) return null;
        return typeof first === 'string' ? first : first.path ?? null;
    }

    return selection.path ?? null;
}

export async function pickSingleDirectory(title: string, defaultPath?: string): Promise<string | null> {
    const selected = await openDialog({
        directory: true,
        multiple: false,
        title,
        defaultPath
    });

    return resolveSingleDialogPath(selected as DialogSelection);
}

export async function pickSingleFile(
    title: string,
    filters?: DialogFileFilter[],
    defaultPath?: string
): Promise<string | null> {
    const selected = await openDialog({
        multiple: false,
        title,
        filters,
        defaultPath
    });

    return resolveSingleDialogPath(selected as DialogSelection);
}
