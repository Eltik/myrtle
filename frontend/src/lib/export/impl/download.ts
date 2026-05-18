import type { IExportResult } from "./types";

export function downloadExport(filename: string, result: IExportResult): void {
    if (typeof window === "undefined") return;
    const blob = new Blob([result.content], { type: result.mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = filename.endsWith(`.${result.extension}`) ? filename : `${filename}.${result.extension}`;
    a.style.display = "none";
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    // Revoke after a tick so the browser has time to start the download.
    setTimeout(() => URL.revokeObjectURL(url), 1_000);
}

export async function copyToClipboard(text: string): Promise<boolean> {
    if (typeof navigator === "undefined" || !navigator.clipboard?.writeText) return false;
    try {
        await navigator.clipboard.writeText(text);
        return true;
    } catch {
        return false;
    }
}
