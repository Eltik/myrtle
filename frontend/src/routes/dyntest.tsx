import { createFileRoute } from "@tanstack/react-router";
import { SceneIllust } from "#/components/operators/detail/impl/components/dynillust/SceneIllust";
export const Route = createFileRoute("/dyntest")({
    component: DynTest,
    validateSearch: (s: Record<string, unknown>) => ({ skel: (s.skel as string) ?? "", backdrop: (s.backdrop as string) ?? "", server: (s.server as "en" | "cn") ?? "en", framing: (s.framing as "character" | "authored") ?? "authored", surface: (s.surface as "viewer" | "panel") ?? "viewer" }),
});
function DynTest() {
    const { skel, backdrop, server, framing, surface } = Route.useSearch();
    if (!skel) return <div style={{ color: "white" }}>no skel</div>;
    const atlas = skel.replace(/\.skel$/, ".atlas");
    const png = skel.replace(/\.skel$/, ".png");
    return (
        <div style={{ position: "fixed", inset: 0, background: "#000" }}>
            <SceneIllust files={{ skel, atlas, png }} server={server} framing={framing} surface={surface} backdrop={backdrop || undefined} />
        </div>
    );
}
