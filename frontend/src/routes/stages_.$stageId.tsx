import { createFileRoute } from "@tanstack/react-router";
import { levelQueryOptions } from "#/lib/api/level";
import { stagesQueryOptions, zonesQueryOptions } from "#/lib/api/stages";
import { defaultOgURL } from "#/lib/og";
import { seo } from "#/lib/seo";

export const Route = createFileRoute("/stages_/$stageId")({
    component: RouteComponent,
    errorComponent: RootErrorComponent,
    loader: async ({ context, params }) => {
        const [stages] = await Promise.all([
            context.queryClient.ensureQueryData(stagesQueryOptions()),
            context.queryClient.ensureQueryData(zonesQueryOptions()),
            // Warm the level so the simulator renders without a client fetch flash.
            context.queryClient.ensureQueryData(levelQueryOptions(params.stageId)),
        ]);
        return stages.find((s) => s.stageId === params.stageId) ?? null;
    },
    head: ({ loaderData, params }) => {
        const code = loaderData?.code ?? params.stageId;
        const name = loaderData?.name ? `${code} · ${loaderData.name}` : code;
        const { meta, links } = seo({
            title: `${name} - Stage`,
            description: `Tile layout and enemy pathing for ${name} in Arknights.`,
            path: `/stages/${params.stageId}`,
            image: defaultOgURL("stages"),
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
    return <h1>Hello world</h1>;
}
