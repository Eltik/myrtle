import { RocketIcon } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Field, FieldDescription, FieldLabel } from "#/components/ui/field";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./PublishVersionDialog.messages";

const CHANGELOG_MAX = 1000;

interface IPublishVersionDialogProps {
    open: boolean;
    publishing: boolean;
    latestVersion: number | null;
    nextVersion: number | null;
    publishError: string | null;
    onClose: () => void;
    onPublish: (changelog: string) => void;
}

export function PublishVersionDialog({ open, publishing, latestVersion, nextVersion, publishError, onClose, onPublish }: IPublishVersionDialogProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const changelogId = useId();
    const [changelog, setChangelog] = useState("");

    useEffect(() => {
        if (open) setChangelog("");
    }, [open]);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (publishing) return;
        onPublish(changelog.trim());
    };

    return (
        <Dialog open={open} onOpenChange={(o) => !o && !publishing && onClose()}>
            <DialogPopup className="sm:max-w-md">
                <form onSubmit={handleSubmit}>
                    <DialogHeader>
                        <DialogTitle className="flex items-center gap-2">
                            <RocketIcon className="h-4 w-4 text-primary" aria-hidden="true" />
                            {t("edit.publishDialog.title")}
                        </DialogTitle>
                        <DialogDescription>{t("edit.publishDialog.description", { version: nextVersion ?? "?" })}</DialogDescription>
                    </DialogHeader>

                    <div className="flex flex-col gap-5 px-6 pb-2">
                        <div className="flex items-center justify-between rounded-lg border border-border bg-muted/30 px-3 py-2 font-mono text-[11px] text-muted-foreground tabular-nums">
                            <span>
                                {t("edit.publishDialog.latest")} <span className="text-foreground">{latestVersion ? `v${latestVersion}` : "-"}</span>
                            </span>
                            <span>
                                {t("edit.publishDialog.next")} <span className="text-foreground">v{nextVersion ?? "?"}</span>
                            </span>
                        </div>

                        <Field>
                            <FieldLabel htmlFor={changelogId}>
                                {t("edit.publishDialog.changelog")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {changelog.length} / {CHANGELOG_MAX}
                                </span>
                            </FieldLabel>
                            <MarkdownEditor id={changelogId} value={changelog} onChange={setChangelog} placeholder={t("edit.publishDialog.changelogPlaceholder")} rows={4} autoFocus disabled={publishing} maxLength={CHANGELOG_MAX} showHint={false} />
                            <FieldDescription>{t("edit.publishDialog.changelogHint")}</FieldDescription>
                        </Field>

                        {publishError && (
                            <div role="alert" className="rounded-md border border-destructive/30 bg-destructive/8 px-2.5 py-2 font-sans text-[12px] text-destructive-foreground">
                                {publishError}
                            </div>
                        )}
                    </div>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" disabled={publishing} />}>{t("edit.publishDialog.cancel")}</DialogClose>
                        <Button type="submit" loading={publishing}>
                            <RocketIcon />
                            {t("edit.publishDialog.submit", { version: nextVersion ?? "?" })}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}
