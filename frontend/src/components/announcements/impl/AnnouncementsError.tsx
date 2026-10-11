import { CloudOff, RotateCw } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "../Announcements.messages";

export function AnnouncementsError({ onRetry, retrying }: { onRetry: () => void; retrying: boolean }) {
    const t: TypedT<typeof messages> = useT("announcements");
    return (
        <Empty role="alert" className="rounded-xl border border-dashed py-10">
            <EmptyHeader>
                <EmptyMedia variant="icon">
                    <CloudOff aria-hidden="true" />
                </EmptyMedia>
                <EmptyTitle>{t("error.title")}</EmptyTitle>
                <EmptyDescription>{t("error.body")}</EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
                <Button variant="outline" onClick={onRetry} disabled={retrying} className="max-lg:h-11!">
                    <RotateCw className={cn("size-4", retrying && "animate-spin")} aria-hidden="true" />
                    {t("error.retry")}
                </Button>
            </EmptyContent>
        </Empty>
    );
}
