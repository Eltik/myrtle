import { createFileRoute } from "@tanstack/react-router";
import { StageDetail } from "#/components/stages/detail/StageDetail";
import { enemiesQueryOptions } from "#/lib/api/enemies";
import { levelQueryOptions } from "#/lib/api/level";
import { materialsQueryOptions } from "#/lib/api/materials";
import { stageDetailQueryOptions, stageIndexQueryOptions, syntheticStageFromIndex } from "#/lib/api/stages";
import { defaultOgURL } from "#/lib/og";
import { buildStageOgData } from "#/lib/og/impl/templates/Stage";
import { ogURL, warmOg } from "#/lib/og/impl/url";
import { seo } from "#/lib/seo";
import type { IStage, IZone } from "#/types/stages";

export const Route = createFileRoute("/stages_/$stageId")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        // Primary path: one slim endpoint returns the stage + zone + level + only
        // the enemies/materials this stage references.
        const detail = await context.queryClient.ensureQueryData(stageDetailQueryOptions(params.stageId));
        if (detail) {
            const zone = detail.zone ?? null;
            warmOg("stage", params.stageId, buildStageOgData(detail.stage, zone ?? undefined));
            return { stage: detail.stage as IStage | null, zone, level: detail.levelData ?? null, enemyData: detail.enemies, materials: detail.materials, hasOg: true };
        }

        // Fallback: procedural IS/RA/CC nodes have no stage_table entry (404). Build a
        // synthetic stage from the stage index (fetched lazily, only here) and load the
        // full enemy/material tables for the map.
        const [index, level, handbook, materials] = await Promise.all([
            context.queryClient.ensureQueryData(stageIndexQueryOptions()),
            context.queryClient.ensureQueryData(levelQueryOptions(params.stageId)),
            context.queryClient.ensureQueryData(enemiesQueryOptions()),
            context.queryClient.ensureQueryData(materialsQueryOptions()),
        ]);
        const entry = index.find((e) => e.stageId === params.stageId);
        let stage: IStage | null = null;
        let zone: IZone | null = null;
        if (entry) {
            const synthetic = syntheticStageFromIndex(entry);
            stage = synthetic.stage;
            zone = synthetic.zone;
        }
        return { stage, zone, level: level ?? null, enemyData: handbook.enemyData, materials: materials.items, hasOg: false };
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
    const { stage, zone, level, enemyData, materials } = Route.useLoaderData();
    return <StageDetail stage={stage} zone={zone} level={level} enemyData={enemyData} materials={materials} />;
}
