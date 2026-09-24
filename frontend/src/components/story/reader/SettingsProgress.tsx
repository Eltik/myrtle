/**
 * BACK UP, RESTORE, RESET: the three buttons that touch the reading document
 * rather than a setting.
 *
 * They are in the settings dialog because that is where a reader looks for
 * them, and in their own file because they are the one block there that writes
 * `myrtle.story.progress` instead of `myrtle.story.settings`. The reset is the
 * only destructive control in the reader, so it is the only one behind a
 * confirm.
 *
 * The confirm is a SIBLING of the settings dialog, not a child of it: an alert
 * nested inside an open dialog is a nested dialog, with its own focus and
 * dismiss behaviour, and this one is neither. That is why the state lives in
 * {@link useProgressActions} and the two halves render where the sheet puts
 * them, rather than in one component that returns both.
 *
 * A restore is CHECKED before it is written: `parseProgress` coerces anything
 * into a valid document, so a file that is not a backup at all would silently
 * restore as empty progress. The guard is the raw text carrying a `"read"` key,
 * which every document this reader has ever written does.
 */
import { DownloadIcon, RotateCcwIcon, UploadIcon } from "lucide-react";
import type React from "react";
import { useId, useRef, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { emptyProgress, loadProgress, parseProgress, saveProgress } from "#/lib/story/progress";
import type { messages } from "./reader.messages";
import { TOUCH_BUTTON } from "./SettingsRow";

export interface IProgressActions {
    notice: string | null;
    confirmOpen: boolean;
    setConfirmOpen: (open: boolean) => void;
    backup: () => void;
    restore: (file: File | undefined) => Promise<void>;
    reset: () => void;
}

export function useProgressActions(): IProgressActions {
    const t: TypedT<typeof messages> = useT("story");
    const [notice, setNotice] = useState<string | null>(null);
    const [confirmOpen, setConfirmOpen] = useState(false);

    return {
        notice,
        confirmOpen,
        setConfirmOpen,
        backup() {
            const blob = new Blob([JSON.stringify(loadProgress(), null, 2)], { type: "application/json" });
            const url = URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `myrtle-story-progress-${new Date().toISOString().slice(0, 10)}.json`;
            a.click();
            URL.revokeObjectURL(url);
        },
        async restore(file: File | undefined) {
            if (!file) return;
            const text = await file.text();
            const p = parseProgress(text);
            const stories = Object.keys(p.read).length;
            const positions = Object.keys(p.pos).length;
            if (stories === 0 && positions === 0 && !text.includes('"read"')) {
                setNotice(t("settings.progress.restoreFailed"));
                return;
            }
            saveProgress(p);
            setNotice(t("settings.progress.restored", { stories, positions }));
        },
        reset() {
            saveProgress(emptyProgress());
            setConfirmOpen(false);
            setNotice(t("settings.progress.wasReset"));
        },
    };
}

/** The three buttons and the outcome line, inside the sheet. */
export function ProgressSection({ actions }: { actions: IProgressActions }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const fileId = useId();
    const fileRef = useRef<HTMLInputElement>(null);
    return (
        <div className="mt-2 border-t pt-4">
            <div className="mb-2 font-heading font-semibold text-sm">{t("settings.progress.heading")}</div>
            <div className="flex flex-wrap gap-2">
                <Button variant="outline" size="sm" className={TOUCH_BUTTON} onClick={actions.backup}>
                    <DownloadIcon /> {t("settings.progress.backup")}
                </Button>
                <Button variant="outline" size="sm" className={TOUCH_BUTTON} onClick={() => fileRef.current?.click()}>
                    <UploadIcon /> {t("settings.progress.restore")}
                </Button>
                <input id={fileId} ref={fileRef} type="file" accept="application/json,.json" className="sr-only" onChange={(e) => void actions.restore(e.target.files?.[0])} />
                <Button variant="destructive" size="sm" className={TOUCH_BUTTON} onClick={() => actions.setConfirmOpen(true)}>
                    <RotateCcwIcon /> {t("settings.progress.reset")}
                </Button>
            </div>
            {/* `<output>` carries role=status implicitly, so the outcome of a
                backup, a restore or a reset is announced without a role attribute. */}
            <output className="mt-2 block text-muted-foreground text-xs" aria-live="polite">
                {actions.notice}
            </output>
        </div>
    );
}

/** The confirm, rendered beside the settings dialog rather than inside it. */
export function ResetProgressDialog({ actions }: { actions: IProgressActions }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    return (
        <AlertDialog open={actions.confirmOpen} onOpenChange={actions.setConfirmOpen}>
            <AlertDialogPopup>
                <AlertDialogHeader>
                    <AlertDialogTitle>{t("settings.reset.title")}</AlertDialogTitle>
                    <AlertDialogDescription>{t("settings.reset.body")}</AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogClose render={<Button variant="outline" />}>{t("settings.reset.cancel")}</AlertDialogClose>
                    <Button variant="destructive" onClick={actions.reset}>
                        {t("settings.reset.confirm")}
                    </Button>
                </AlertDialogFooter>
            </AlertDialogPopup>
        </AlertDialog>
    );
}
