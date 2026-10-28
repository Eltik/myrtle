import { HistoryIcon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import { useFormatters } from "#/lib/i18n/formatters";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./DraftRestoreBanner.messages";

interface IDraftRestoreBannerProps {
    savedAt: number;
    /** The server's list changed after the kept edits began. */
    diverged: boolean;
    onRestore: () => void;
    onDiscard: () => void;
}

/** Offers edits this browser kept from a session that ended unsaved. Nothing is restored until the author says so. */
export function DraftRestoreBanner({ savedAt, diverged, onRestore, onDiscard }: IDraftRestoreBannerProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const { relativeShort } = useFormatters();

    return (
        <section role="alert" className="flex flex-col gap-3 rounded-xl border border-primary/30 bg-primary/6 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
            <div className="flex min-w-0 items-start gap-2.5">
                <HistoryIcon className="mt-0.5 h-4 w-4 shrink-0 text-primary" aria-hidden="true" />
                <div className="min-w-0">
                    <p className="m-0 font-medium font-sans text-foreground text-sm">{t("edit.draft.title")}</p>
                    <p className="mt-0.5 font-sans text-[12.5px] text-muted-foreground">{t("edit.draft.body", { when: relativeShort(savedAt) })}</p>
                    {diverged && <p className="mt-1 font-sans text-[12.5px] text-foreground">{t("edit.draft.diverged")}</p>}
                </div>
            </div>
            <div className="flex shrink-0 items-center gap-1.5">
                <Button type="button" size="sm" onClick={onRestore}>
                    {t("edit.draft.restore")}
                </Button>
                <Button type="button" size="sm" variant="outline" onClick={onDiscard}>
                    {t("edit.draft.discard")}
                </Button>
            </div>
        </section>
    );
}
