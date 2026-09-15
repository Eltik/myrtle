import type { messages as tileMessages } from "./tile-defs.messages";

/** A key in `tile-defs.messages.ts`; resolved by `Tile.tsx` when it paints the tile. */
export type TileMessageKey = keyof typeof tileMessages & string;

export interface ITileType {
    legend?: string;
    /** Message key for the glyph painted on the tile, when it carries one. */
    contentKey?: TileMessageKey;
    content_on_top?: boolean;
    effect?: string;
    render_as?: string;
}

export const TILE_TYPES: Record<string, ITileType> = {
    tile_forbidden: { legend: "Forbidden" },
    tile_floor: { legend: "Floor" },
    tile_wall: { legend: "Wall" },
    tile_road: { legend: "Road" },
    tile_start: { legend: "Start" },
    tile_flystart: { legend: "Fly Start", contentKey: "tile.flyStart", content_on_top: true },
    tile_end: { legend: "End" },
    tile_telin: { legend: "Teleport in", contentKey: "tile.teleportIn" },
    tile_telout: { legend: "Teleport out", contentKey: "tile.teleportOut" },
    tile_hole: { legend: "Hole", contentKey: "tile.hole" },
    tile_empty: { legend: "Empty" },
    tile_grass: { legend: "Grass", content_on_top: true, effect: "effect_grass" },
    tile_fence: { legend: "Fence", content_on_top: true },
    tile_fence_bound: { render_as: "tile_fence", legend: "Fence Bound", content_on_top: true },
    tile_healing: { legend: "Healing", content_on_top: true, effect: "effect_healing" },
    tile_bigforce: { legend: "Big force", contentKey: "tile.bigForce", content_on_top: true, effect: "effect_buff" },
    tile_defup: { legend: "Def up", contentKey: "tile.defUp", content_on_top: true, effect: "effect_buff" },
    tile_gazebo: { legend: "Gazebo", contentKey: "tile.gazebo", content_on_top: true, effect: "effect_buff" },
    tile_infection: { legend: "Infection", effect: "effect_infection" },
    tile_volcano: { legend: "Volcano", effect: "effect_volcano" },
    tile_volcano_emp: { legend: "Volcano", effect: "effect_volcano" },
    tile_corrosion: { legend: "Corrosion", effect: "effect_corrosion" },
    tile_defbreak: { legend: "Def break", effect: "effect_corrosion" },
    tile_shallowwater: { legend: "Deep water", effect: "effect_shallowwater" },
    tile_deepwater: { legend: "Deep water", effect: "effect_deepwater" },
    tile_deepsea: { legend: "Deep sea", effect: "effect_deepwater" },
    tile_volspread: { legend: "Volcano spread", effect: "effect_volspread" },
    tile_smog: { legend: "Smoke generator", effect: "effect_smog" },
    tile_wooden_wall: { legend: "Wooden wall", contentKey: "tile.woodenWall" },
    tile_yinyang_road: { legend: "YinYang road", effect: "effect_yinyang_road" },
    tile_yinyang_wall: { legend: "YinYang wall", effect: "effect_yinyang_wall" },
    tile_yinyang_switch: { legend: "YinYang switch", effect: "effect_yinyang_switch" },
    tile_poison: { legend: "Poison", effect: "effect_poison" },
    tile_icestr: { legend: "Ice", effect: "effect_ice" },
    tile_icetur_lb: { legend: "Ice corner", effect: "effect_ice" },
    tile_icetur_lt: { legend: "Ice corner", effect: "effect_ice" },
    tile_icetur_rb: { legend: "Ice corner", effect: "effect_ice" },
    tile_icetur_rt: { legend: "Ice corner", effect: "effect_ice" },
    tile_aircraft: { legend: "Aircraft" },
    tile_reed: { legend: "Reed", effect: "effect_reed" },
    tile_reedf: { legend: "Reed forbidden", effect: "effect_reed" },
    tile_reedw: { legend: "Reed wall", effect: "effect_reed" },
    tile_mire: { legend: "Mire", effect: "effect_mire" },
    tile_passable_wall: { legend: "Passable Wall" },
    tile_passable_wall_forbidden: { legend: "Passable Wall" },
    tile_stairs: { legend: "Stairs", contentKey: "tile.stairs" },
    tile_grvtybtn: { legend: "Gravity Button" },
    tile_ristar_road: { legend: "Ristar Road", effect: "effect_ristar_road" },
    tile_ristar_road_forbidden: { legend: "Ristar Road", effect: "effect_ristar_road" },
    tile_start_cooperate: { render_as: "tile_start" },
    tile_end_cooperate: { render_as: "tile_end" },
    tile_rcm_operator: { render_as: "tile_road", legend: "Recommended Operator" },
    tile_rcm_crate: { render_as: "tile_road", legend: "Recommended Crate" },
    tile_crate: { legend: "Box", effect: "effect_crate" },
};

export const BOX_TILES: string[] = ["tile_start", "tile_flystart", "tile_end", "tile_crate", "tile_start_cooperate", "tile_end_cooperate"];
export const SMALL_DEPLOYABLE: string[] = ["tile_wall", "tile_road", "tile_healing", "tile_grass", "tile_gazebo", "tile_defup", "tile_bigforce", "tile_yinyang_road", "tile_yinyang_wall", "tile_icestr", "tile_passable_wall"];
