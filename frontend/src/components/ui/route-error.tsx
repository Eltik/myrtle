import { Link, useRouter } from "@tanstack/react-router";
import type * as React from "react";
import { Button } from "#/components/ui/button";
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle } from "#/components/ui/empty";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./route-error.messages";

export interface IRouteErrorProps {
    error: unknown;
    /** The error boundary's own reset, from TanStack's `errorComponent` props. */
    reset?: () => void;
}

function messageOf(error: unknown): string | null {
    if (error instanceof Error) return error.message || null;
    if (typeof error === "string") return error || null;
    return null;
}

/**
 * What a route renders when its loader or its component throws: the
 * `errorComponent` of a route, or the router's `defaultErrorComponent`.
 *
 * Retry invalidates the router so the loaders run again, then resets the
 * boundary so the component tree mounts fresh. The error's own message is
 * shown because it is usually the server's text and the only clue a visitor
 * can report; the stack is a development aid and never ships to production.
 */
export function RouteError({ error, reset }: IRouteErrorProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const router = useRouter();
    const message = messageOf(error);
    const stack = import.meta.env.DEV && error instanceof Error ? error.stack : undefined;

    return (
        <Empty role="alert">
            <EmptyHeader>
                <EmptyTitle>{t("routeError.title")}</EmptyTitle>
                <EmptyDescription>{t("routeError.description")}</EmptyDescription>
            </EmptyHeader>
            <EmptyContent>
                {message ? <p className="m-0 max-w-full break-words font-mono text-muted-foreground text-xs">{message}</p> : null}
                <div className="flex flex-wrap items-center justify-center gap-2">
                    <Button
                        onClick={() => {
                            void router.invalidate();
                            reset?.();
                        }}
                    >
                        {t("routeError.retry")}
                    </Button>
                    <Button render={<Link to="/" />} variant="outline">
                        {t("routeError.home")}
                    </Button>
                </div>
            </EmptyContent>
            {stack ? <pre className="m-0 max-h-64 w-full max-w-3xl overflow-auto rounded-lg border bg-muted p-3 text-left font-mono text-muted-foreground text-xs">{stack}</pre> : null}
        </Empty>
    );
}
