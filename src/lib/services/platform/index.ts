import {
	getPlatformInfo as getPlatformInfoApi,
	type PlatformInfo,
} from "$lib/api/platform";

export type { PlatformInfo };

export async function getPlatformInfo(): Promise<PlatformInfo> {
	return getPlatformInfoApi();
}
