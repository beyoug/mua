import { invoke } from "@tauri-apps/api/core";
import type { Aria2VersionInfo } from "$lib/types/download";

export async function getAria2ConfigPath(): Promise<string> {
	return invoke<string>("get_aria2_config_path");
}

export async function readAria2Config(): Promise<string> {
	return invoke<string>("read_aria2_config");
}

export async function importAria2Config(path: string): Promise<void> {
	await invoke<void>("import_aria2_config", { path });
}

export async function importCustomBinary(path: string): Promise<string> {
	return invoke<string>("import_custom_binary", { filePath: path });
}

export async function getAria2VersionInfo(): Promise<Aria2VersionInfo> {
	return invoke<Aria2VersionInfo>("get_aria2_version_info");
}

export async function trustCustomBinary(): Promise<void> {
	await invoke("trust_custom_aria2_binary");
}

export async function fetchPublicTrackers(): Promise<string[]> {
	return invoke<string[]>("fetch_public_trackers");
}

export async function startLogStream(): Promise<void> {
	await invoke("start_log_stream");
}

export async function stopLogStream(): Promise<void> {
	await invoke("stop_log_stream");
}
