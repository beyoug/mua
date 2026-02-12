interface BackendErrorPayload {
    type?: string;
    message?: unknown;
}

function isBackendErrorPayload(value: unknown): value is BackendErrorPayload {
    return typeof value === 'object' && value !== null;
}

export function getErrorMessage(error: unknown, fallback = '操作失败'): string {
    if (typeof error === 'string') {
        return error;
    }

    if (error instanceof Error && error.message.trim()) {
        return error.message;
    }

    if (isBackendErrorPayload(error)) {
        const payload = error as BackendErrorPayload;
        if (typeof payload.message === 'string' && payload.message.trim()) {
            return payload.message;
        }
    }

    return fallback;
}
