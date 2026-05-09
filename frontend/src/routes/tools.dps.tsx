import { createFileRoute } from "@tanstack/react-router";
import { dpsOperatorsQueryOptions } from "#/lib/api/dps";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/tools/dps")({
    component: RouteComponent,
    loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(dpsOperatorsQueryOptions()),
    head: () => {
        const { meta, links } = seo({
            title: "DPS Calculator",
            description: "Compare Arknights operator DPS across varying enemy DEF, RES, or target count. Configure skills, modules, buffs, and conditionals.",
            path: "/tools/dps",
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RouteComponent() {
    return <></>;
}
