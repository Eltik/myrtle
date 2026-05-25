export function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    if (target.isContentEditable) return true;
    const tag = target.tagName;
    if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
    const role = target.getAttribute("role");
    if (role === "textbox" || role === "combobox" || role === "searchbox") return true;
    return false;
}

export function hasMod(e: KeyboardEvent): boolean {
    return e.metaKey || e.ctrlKey;
}

export function isPlainKey(e: KeyboardEvent): boolean {
    return !e.metaKey && !e.ctrlKey && !e.altKey;
}
