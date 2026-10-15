import { toastManager } from "#/components/ui/toast";

/*
 * The panel's write-result toasts. `prefix` names the action; the id appends
 * the time so a repeat never replaces an earlier toast still on screen.
 */

export function toastSuccess(prefix: string, title: string, description: string | undefined): string {
    return toastManager.add({ id: `${prefix}-${Date.now()}`, title, description, type: "success" });
}

export function toastError(prefix: string, title: string, description: string | undefined): string {
    return toastManager.add({ id: `${prefix}-${Date.now()}`, title, description, type: "error" });
}
