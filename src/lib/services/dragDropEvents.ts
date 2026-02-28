import { listen } from "@tauri-apps/api/event";

interface DragEventPayload {
	paths?: string[];
	position?: { x: number; y: number };
}

interface DragDropHandlers {
	onEnter: () => void;
	onOver: () => void;
	onDrop: (paths: string[]) => void;
	onLeave: () => void;
}

export async function registerDragDropHandlers(
	handlers: DragDropHandlers,
): Promise<() => void> {
	const unlisteners: Array<() => void> = [];

	const u1 = await listen<DragEventPayload>("tauri://drag-enter", () => {
		handlers.onEnter();
	});
	unlisteners.push(u1);

	const u2 = await listen<DragEventPayload>("tauri://drag-over", () => {
		handlers.onOver();
	});
	unlisteners.push(u2);

	const u3 = await listen<DragEventPayload>("tauri://drag-drop", (event) => {
		handlers.onDrop(event.payload.paths ?? []);
	});
	unlisteners.push(u3);

	const u4 = await listen("tauri://drag-leave", () => {
		handlers.onLeave();
	});
	unlisteners.push(u4);

	return () => {
		unlisteners.forEach((unlisten) => {
			unlisten();
		});
	};
}
