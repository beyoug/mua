import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
	fetchPublicTrackers,
	getAria2ConfigPath,
	getAria2VersionInfo,
	importCustomBinary,
	startLogStream,
	stopLogStream,
	trustCustomBinary,
} from "$lib/api/aria2";
import { EVENT_ARIA2_STDOUT } from "$lib/api/events";
import type { Aria2VersionInfo } from "$lib/types/download";
import { createLogger } from "$lib/utils/logger";

const logger = createLogger("Aria2Service");

let logUnlisten: UnlistenFn | null = null;
let startInFlight: Promise<void> | null = null;
let stopInFlight: Promise<void> | null = null;

export async function fetchTrackers(): Promise<string[]> {
	return fetchPublicTrackers();
}

export async function startAria2LogStream(): Promise<void> {
	if (logUnlisten || startInFlight) {
		return startInFlight ?? Promise.resolve();
	}

	startInFlight = (async () => {
		await startLogStream();
	})();

	try {
		await startInFlight;
	} finally {
		startInFlight = null;
	}
}

export async function stopAria2LogStream(): Promise<void> {
	if (stopInFlight) {
		await stopInFlight;
		return;
	}

	stopInFlight = (async () => {
		await stopLogStream();
	})();

	try {
		await stopInFlight;
	} finally {
		stopInFlight = null;
	}
}

export async function importAria2Binary(path: string): Promise<string> {
	return importCustomBinary(path);
}

export async function trustImportedAria2Binary(): Promise<void> {
	await trustCustomBinary();
}

export async function getAria2KernelVersionInfo(): Promise<Aria2VersionInfo> {
	return getAria2VersionInfo();
}

export async function getAria2ConfigPathInfo(): Promise<string> {
	return getAria2ConfigPath();
}

export async function subscribeAria2Stdout(
	onLog: (line: string) => void,
): Promise<() => void> {
	if (logUnlisten) {
		logger.warn(
			"Aria2 stdout listener already active, replacing previous subscriber",
		);
		logUnlisten();
		logUnlisten = null;
	}

	const unlisten = await listen<string>(EVENT_ARIA2_STDOUT, (event) => {
		onLog(event.payload);
	});

	logUnlisten = unlisten;

	return () => {
		if (!logUnlisten) return;
		logUnlisten();
		logUnlisten = null;
	};
}
