import { confirm, message } from '@tauri-apps/plugin-dialog';

type FeedbackKind = 'info' | 'warning' | 'error';

interface FeedbackOptions {
    title: string;
    kind?: FeedbackKind;
}

interface ConfirmOptions {
    title: string;
    okLabel?: string;
    cancelLabel?: string;
}

export async function showFeedback(text: string, options: FeedbackOptions): Promise<void> {
    await message(text, {
        title: options.title,
        kind: options.kind ?? 'info'
    });
}

export async function showSuccessFeedback(title: string, text: string): Promise<void> {
    await showFeedback(text, { title, kind: 'info' });
}

export async function showErrorFeedback(title: string, error: unknown): Promise<void> {
    const detail = typeof error === 'string' ? error : error instanceof Error ? error.message : String(error);
    await showFeedback(`${title}: ${detail}`, { title, kind: 'error' });
}

export async function confirmAction(text: string, options: ConfirmOptions): Promise<boolean> {
    return confirm(text, {
        title: options.title,
        okLabel: options.okLabel,
        cancelLabel: options.cancelLabel
    });
}
