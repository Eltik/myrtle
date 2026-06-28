import type { ILevel, IMapOperator } from "../impl/types";
import { cx } from "../impl/util/cx";
import { type ITileContext, Tile } from "./Tile";

export function Board({ level, coordOverride, operators, hovered }: { level: ILevel; coordOverride?: string | null; operators: Map<number, IMapOperator>; hovered?: boolean }) {
    const { tiles } = level.mapData;
    const width = level.mapData.map[0].length;
    const height = level.mapData.map.length;
    const ctx: ITileContext = {
        width,
        height,
        coordOverride,
        operators,
        disallow: new Set(level.tilesDisallowToLocate || []),
        hovered,
    };

    return (
        <div className={cx("transform-3d flex flex-col-reverse items-center justify-center gap-y-0.5 py-12 transition-transform duration-400 ease-[ease]", hovered ? "map-hovered rotate-x-30" : "rotate-x-0")}>
            {Array.from({ length: height }, (_, r) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: fixed grid, row position is its identity
                <div key={`row-${r}`} className="transform-3d flex flex-row items-center justify-start gap-x-0.5">
                    {Array.from({ length: width }, (_, c) => (
                        // biome-ignore lint/suspicious/noArrayIndexKey: fixed grid, (row,col) is the tile identity
                        <Tile key={`tile-${r}-${c}`} def={tiles[r * width + c]} row={r} col={c} ctx={ctx} />
                    ))}
                </div>
            ))}
        </div>
    );
}
