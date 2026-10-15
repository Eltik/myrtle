import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { MAX_SELECTED_TAGS } from "#/components/tools/recruitment/impl/constants";
import { RecruitmentCalculator } from "#/components/tools/recruitment/RecruitmentCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { resolveGamedataServer } from "#/lib/api/gamedata";
import { recruitmentDataQueryOptions } from "#/lib/api/recruitment";
import { metaT } from "#/lib/meta";
import { defaultOgURL, ogURL } from "#/lib/og";
import { recruitmentOgId } from "#/lib/og/impl/recruitment";
import { buildRecruitmentOgData } from "#/lib/og/impl/recruitment.build";
import { seo } from "#/lib/seo";

export interface IRecruitmentSearch {
    tags: number[];
}

const RECRUITMENT_DEFAULTS: IRecruitmentSearch = { tags: [] };

function parseTagsParam(raw: unknown): number[] {
    const parts: string[] = Array.isArray(raw) ? raw.map(String) : typeof raw === "string" ? raw.split(",") : typeof raw === "number" ? [String(raw)] : [];

    const seen = new Set<number>();
    const out: number[] = [];
    for (const part of parts) {
        const n = Number(part);
        if (!Number.isFinite(n) || !Number.isInteger(n) || n <= 0) continue;
        if (seen.has(n)) continue;
        seen.add(n);
        out.push(n);
        if (out.length >= MAX_SELECTED_TAGS) break;
    }
    return out;
}

export const Route = createFileRoute("/tools/recruitment")({
    component: RouteComponent,
    errorComponent: RecruitmentErrorComponent,
    validateSearch: (search: Record<string, unknown>): IRecruitmentSearch => ({
        tags: parseTagsParam(search.tags),
    }),
    search: { middlewares: [stripSearchParams(RECRUITMENT_DEFAULTS)] },
    loader: ({ context: { queryClient, i18n } }) => queryClient.prefetchQuery(recruitmentDataQueryOptions(i18n.gamedataServer)),
    head: ({ match }) => {
        const t = metaT(match.context.i18n);
        // A link with tags unfurls as that selection's card, built from the data the
        // loader prefetched; with no tags, no tag the server knows, or no data, it is
        // the tool's preset card.
        const server = resolveGamedataServer(match.context.i18n.gamedataServer);
        const data = match.search.tags.length > 0 ? match.context.queryClient.getQueryData(recruitmentDataQueryOptions(server).queryKey) : undefined;
        const og = data ? buildRecruitmentOgData(match.search.tags, data, server) : null;
        const image = og
            ? ogURL(
                  "recruitment",
                  recruitmentOgId(
                      og.tags.map((tag) => tag.id),
                      server,
                  ),
                  og,
              )
            : defaultOgURL("tools-recruitment", match.context.i18n);
        const { meta, links } = seo({
            title: t("toolsRecruitment.title"),
            description: t("toolsRecruitment.description"),
            path: "/tools/recruitment",
            image,
            locale: match.context.i18n?.locale,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <RecruitmentCalculator />;
}

function RecruitmentErrorComponent({ error, reset }: { error: unknown; reset: () => void }) {
    const message = error instanceof Error ? error.message : String(error);
    return (
        <div className="relative z-1 mx-auto w-[min(640px,calc(100%-2rem))] py-20">
            <Empty>
                <EmptyHeader>
                    <EmptyMedia variant="icon">
                        <AlertTriangle className="text-destructive" />
                    </EmptyMedia>
                    <EmptyTitle>Recruitment Calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading recruitment data. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
