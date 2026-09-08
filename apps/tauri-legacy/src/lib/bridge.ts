import { invoke } from "@tauri-apps/api/core";
import type {
  CanvasObject,
  DeliveryOutcome,
  InvocationGesture,
} from "../domain/types";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export async function syncScene(objects: CanvasObject[]): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("commit_scene", { objects });
}

export async function loadCurrentScene(): Promise<CanvasObject[]> {
  if (!isTauriRuntime()) return [];
  return invoke<CanvasObject[]>("current_scene");
}

export async function discardSession(): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("discard_session");
}

export async function setInvocationGesture(
  gesture: InvocationGesture,
): Promise<void> {
  if (!isTauriRuntime()) return;
  await invoke("set_invocation_gesture", { gesture });
}

export async function deliverImage(dataUrl: string): Promise<DeliveryOutcome> {
  if (isTauriRuntime()) {
    return invoke<DeliveryOutcome>("deliver_image", { dataUrl });
  }

  try {
    const response = await fetch(dataUrl);
    const blob = await response.blob();
    await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
    return { status: "preview", message: "Copied in browser preview" };
  } catch {
    return { status: "preview", message: "Browser preview cannot dispatch paste" };
  }
}
