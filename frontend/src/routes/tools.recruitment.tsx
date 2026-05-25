import { createFileRoute, stripSearchParams } from "@tanstack/react-router";
import { AlertTriangle } from "lucide-react";
import { MAX_SELECTED_TAGS } from "#/components/tools/recruitment/impl/constants";
import { RecruitmentCalculator } from "#/components/tools/recruitment/RecruitmentCalculator";
import { Button } from "#/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "#/components/ui/empty";
import { recruitmentDataQueryOptions } from "#/lib/api/recruitment";
import { defaultOgURL } from "#/lib/og";
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
    loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(recruitmentDataQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "Recruitment Calculator",
            description: "Calculate optimal tag combinations for Arknights recruitment. Find guaranteed 5-star and 6-star operators with the best tag combos.",
            path: "/tools/recruitment",
            image: defaultOgURL("tools-recruitment"),
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
                    <EmptyTitle>Recruitment calculator failed to load</EmptyTitle>
                    <EmptyDescription>{message || "An unexpected error occurred while loading recruitment data. Try again, or check the browser console for details."}</EmptyDescription>
                </EmptyHeader>
                <Button onClick={reset} variant="outline">
                    Retry
                </Button>
            </Empty>
        </div>
    );
}
