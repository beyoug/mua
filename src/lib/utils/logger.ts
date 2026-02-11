type LogLevel = 'debug' | 'info' | 'warn' | 'error';

interface LogContext {
    [key: string]: unknown;
}

interface Logger {
    debug: (message: string, context?: LogContext) => void;
    info: (message: string, context?: LogContext) => void;
    warn: (message: string, context?: LogContext) => void;
    error: (message: string, context?: LogContext) => void;
}

const appName = 'MUA';
const debugEnabled = Boolean(import.meta.env.DEV || import.meta.env.VITE_DEBUG_LOGS === '1');

function toConsoleContext(context?: LogContext): LogContext | undefined {
    if (!context) return undefined;
    const normalized: LogContext = {};

    for (const [key, value] of Object.entries(context)) {
        if (value instanceof Error) {
            normalized[key] = {
                name: value.name,
                message: value.message,
                stack: value.stack
            };
        } else {
            normalized[key] = value;
        }
    }

    return normalized;
}

function emit(level: LogLevel, scope: string, message: string, context?: LogContext): void {
    if (level === 'debug' && !debugEnabled) {
        return;
    }

    const prefix = `[${appName}][${level.toUpperCase()}][${scope}] ${message}`;
    const normalizedContext = toConsoleContext(context);

    if (!normalizedContext || Object.keys(normalizedContext).length === 0) {
        console[level](prefix);
        return;
    }

    console[level](prefix, normalizedContext);
}

export function createLogger(scope: string): Logger {
    return {
        debug: (message, context) => emit('debug', scope, message, context),
        info: (message, context) => emit('info', scope, message, context),
        warn: (message, context) => emit('warn', scope, message, context),
        error: (message, context) => emit('error', scope, message, context)
    };
}
