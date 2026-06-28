import { BOX_TILES, TILE_TYPES } from "../impl/tile/tile-defs";
import type { IMapOperator, IRawTile } from "../impl/types";
import { coordLabel } from "../impl/util/coord";
import { cx } from "../impl/util/cx";
import { encodeBuildableType, encodeHeightType } from "../impl/util/enums";
import { Marker } from "./Marker";

const PASSABLE_WALLS = ["tile_passable_wall", "tile_passable_wall_forbidden"];
const COORD_EXCLUDE = ["tile_hole", "tile_empty", "tile_deepwater", "tile_smog"];

interface IOverlaySpec {
    type: string;
    content?: string;
    options: {
        content_on_top?: boolean;
        content_on_box?: boolean;
        content_fade_out_perspective?: boolean;
        content_fade_out_flat?: boolean;
    };
}

const OVERLAY_BASE = "absolute flex flex-col content-center items-center justify-center h-12 w-12";

const TYPE_BASE: Record<string, string> = {
    content: "bg-transparent [text-shadow:0_0_5px_rgba(48,48,48,0.6)] [transition:transform_0.4s_ease,opacity_0.4s_ease]",
    coord: "pointer-events-none [transition:opacity_0.4s_ease_0.5s,transform_0.4s_ease]",
    deployable: "border border-[hsla(0,0%,87%,0.9)] pointer-events-auto [transition:border_0.4s_ease]",
};

function overlayTransform(spec: IOverlaySpec, hovered?: boolean): string {
    const o = spec.options;
    if (spec.type === "content") {
        return hovered && o.content_on_top ? "scale-90 translate-z-[19.2px]" : "scale-100 translate-z-px";
    }
    if (spec.type === "coord") {
        return hovered ? "scale-100 translate-x-0 translate-y-0" : "scale-[0.6] translate-x-4.5 translate-y-5";
    }
    if (hovered && o.content_on_top) return o.content_on_box ? "translate-z-12" : "translate-z-[19.2px]";
    return "";
}

function overlayOpacity(spec: IOverlaySpec, hovered?: boolean): string {
    const o = spec.options;
    if (spec.type === "content") return o.content_fade_out_perspective && hovered ? "opacity-10" : "opacity-80";
    if (spec.type === "coord") {
        if (hovered) return o.content_fade_out_perspective ? "opacity-10" : "opacity-20";
        return o.content_fade_out_flat ? "opacity-10" : "opacity-50";
    }
    return "";
}

function Overlay({ spec, zIndex, hovered }: { spec: IOverlaySpec; zIndex: number; hovered?: boolean }) {
    return (
        <span className={cx(OVERLAY_BASE, TYPE_BASE[spec.type] ?? cx(spec.type, "enable_animation"), overlayTransform(spec, hovered), overlayOpacity(spec, hovered))} style={{ zIndex }}>
            {spec.content || null}
        </span>
    );
}

export interface ITileContext {
    width: number;
    height: number;
    coordOverride?: string | null;
    operators: Map<number, IMapOperator>;
    disallow: Set<number>;
    hovered?: boolean;
}

export function Tile({ def, row, col, ctx }: { def: IRawTile; row: number; col: number; ctx: ITileContext }) {
    const key = def.tileKey;
    const type = TILE_TYPES[key] || ({ content: "" } as (typeof TILE_TYPES)[string]);
    const heightCode = encodeHeightType(def.heightType); // 0 lowland, 1 highland
    const buildCode = encodeBuildableType(def.buildableType); // 0 none .. 3 all

    const block = (heightCode === 1 && key !== "tile_flystart") || PASSABLE_WALLS.includes(key);
    const box = BOX_TILES.includes(key);
    const hole = ["tile_hole", "tile_deepwater"].includes(key);
    const fence = ["tile_fence", "tile_fence_bound"].includes(key);
    const deployable = buildCode > 0 && !ctx.disallow.has(row * ctx.width + col);
    const coordStr = coordLabel({ row, col, height: ctx.height, override: ctx.coordOverride });

    const overlays: IOverlaySpec[] = [];
    if (type.effect) overlays.push({ type: type.effect, content: "", options: { content_on_top: block || box } });

    const contentOv: IOverlaySpec | null = type.content ? { type: "content", content: type.content, options: { content_on_top: block && type.content_on_top, content_on_box: box && type.content_on_top, content_fade_out_perspective: key === "tile_hole" } } : null;

    const coordShown = !COORD_EXCLUDE.includes(key) && !(!block && contentOv) && coordStr;
    if (coordShown) overlays.push({ type: "coord", content: coordStr, options: { content_fade_out_perspective: block, content_fade_out_flat: type.content_on_top } });

    if (contentOv) overlays.push(contentOv);
    if (deployable) overlays.push({ type: "deployable", content: "", options: { content_on_top: block } });

    const zIndex = 5900 - 10 * row;
    const operator = ctx.operators.get(row * ctx.width + col);
    const showBox = block || box || fence || hole;

    return (
        <div className="transform-3d h-12 min-h-12 w-12 min-w-12">
            <div className={cx("tile transform-3d relative flex h-full w-full select-none items-center justify-center", type.render_as || key, block && "tile_block", box && "tile_box", hole && "tile_hole", fence && "tile_fence", deployable && "tile_deployable")} style={{ zIndex }}>
                {showBox && (
                    <>
                        <span className="tile_box_left" />
                        <span className="tile_box_top" />
                        <span className="tile_box_right" />
                    </>
                )}
                {overlays.map((spec, i) => (
                    <Overlay key={spec.type} spec={spec} zIndex={zIndex + i + 1} hovered={ctx.hovered} />
                ))}
                {operator && (
                    <div className={cx("transform-3d absolute z-9999 h-full w-full p-1 [transition:transform_0.4s_ease,opacity_0.4s_ease]", ctx.hovered ? (block ? "translate-z-[21.2px]" : "translate-z-0.5") : "translate-z-0")}>
                        <Marker operator={operator} />
                    </div>
                )}
            </div>
        </div>
    );
}
