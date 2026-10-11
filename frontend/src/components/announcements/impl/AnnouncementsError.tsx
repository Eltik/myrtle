import type { UseQueryResult } from "@tanstack/react-query";
import { CloudOff, RotateCw } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "../Announcements.messages";

interface IRetryProps {
    onRetry: () => void;
    retrying: boolean;
}

export function AnnouncementsError({ onRetry, retrying }: IRetryProps) {
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
                <RetryButton onRetry={onRetry} retrying={retrying} />
            </EmptyContent>
        </Empty>
    );
}

/** {@link AnnouncementsError} and {@link RetryButton} props for a query: refetch it, spin while it runs. */
export function retryOf(query: Pick<UseQueryResult, "refetch" | "isFetching">): IRetryProps {
    return { onRetry: () => void query.refetch(), retrying: query.isFetching };
}

/** Asks the board again; spins while the request is in flight. */
export function RetryButton({ onRetry, retrying }: IRetryProps) {
    const t: TypedT<typeof messages> = useT("announcements");
    return (
        <Button variant="outline" onClick={onRetry} disabled={retrying} className="max-lg:h-11!">
            <RotateCw className={cn("size-4", retrying && "animate-spin")} aria-hidden="true" />
            {t("error.retry")}
        </Button>
    );
}
