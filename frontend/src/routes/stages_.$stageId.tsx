import { createFileRoute } from "@tanstack/react-router";
import { StageDetail } from "#/components/stages/detail/StageDetail";
import { enemiesQueryOptions } from "#/lib/api/enemies";
import { levelQueryOptions } from "#/lib/api/level";
import { materialsQueryOptions } from "#/lib/api/materials";
import { stageIndexQueryOptions, stagesQueryOptions, syntheticStageFromIndex, zonesQueryOptions } from "#/lib/api/stages";
import { defaultOgURL } from "#/lib/og";
import { buildStageOgData } from "#/lib/og/impl/templates/Stage";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stages_/$stageId")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        const [stages, zones, , level] = await Promise.all([
            context.queryClient.ensureQueryData(stagesQueryOptions()),
            context.queryClient.ensureQueryData(zonesQueryOptions()),
            context.queryClient.ensureQueryData(enemiesQueryOptions()),
            context.queryClient.ensureQueryData(levelQueryOptions(params.stageId)),
            context.queryClient.ensureQueryData(materialsQueryOptions()),
        ]);
        const realStage = stages.find((s) => s.stageId === params.stageId) ?? null;
        let stage = realStage;
        let zone = realStage ? (zones.find((z) => z.zoneId === realStage.zoneId) ?? null) : null;

        if (!stage) {
            const index = await context.queryClient.ensureQueryData(stageIndexQueryOptions());
            const entry = index.find((e) => e.stageId === params.stageId);
            if (entry) {
                const synthetic = syntheticStageFromIndex(entry);
                stage = synthetic.stage;
                zone = synthetic.zone;
            }
        }
        if (realStage) warmOg("stage", params.stageId, buildStageOgData(realStage, zone ?? undefined));
        return { stage, zone, level: level ?? null, hasOg: !!realStage };
    },
    head: ({ loaderData, params }) => {
        const stage = loaderData?.stage ?? null;
        const zone = loaderData?.zone ?? null;
        const code = stage?.code ?? params.stageId;
        const name = stage?.name ? `${code} · ${stage.name}` : code;
        const hasOg = loaderData?.hasOg ?? false;
        const image = stage && hasOg ? ogURL("stage", params.stageId, buildStageOgData(stage, zone ?? undefined)) : defaultOgURL("stages");
        const { meta, links } = seo({
            title: `${name} - Stage`,
            description: `Tile layout and enemy pathing for ${name} in Arknights.`,
            path: `/stages/${params.stageId}`,
            image,
            preloadImage: stage != null && hasOg,
        });
        return {
            meta: [{ charSet: "utf-8" }, { name: "viewport", content: "width=device-width, initial-scale=1" }, ...meta],
            links,
        };
    },
});

function RootErrorComponent({ error }: { error: unknown }) {
    console.error("Router error:", error);
    return (
        <div style={{ padding: 20 }}>
            <h1>Something went wrong</h1>
            <pre style={{ color: "red" }}>{error instanceof Error ? error.message : JSON.stringify(error, null, 2)}</pre>
        </div>
    );
}

function RouteComponent() {
    const { level, stage, zone } = Route.useLoaderData();
    return <StageDetail level={level} fallbackStage={stage} fallbackZone={zone} />;
}
