export interface ITileType {
    legend?: string;
    content: string;
    content_on_top?: boolean;
    effect?: string;
    render_as?: string;
}

export const TILE_TYPES: Record<string, ITileType> = {
    tile_forbidden: { legend: "Forbidden", content: "" },
    tile_floor: { legend: "Floor", content: "" },
    tile_wall: { legend: "Wall", content: "" },
    tile_road: { legend: "Road", content: "" },
    tile_start: { legend: "Start", content: "" },
    tile_flystart: { legend: "Fly Start", content: "air", content_on_top: true },
    tile_end: { legend: "End", content: "" },
    tile_telin: { legend: "Teleport in", content: "in" },
    tile_telout: { legend: "Teleport out", content: "out" },
    tile_hole: { legend: "Hole", content: "hole" },
    tile_empty: { legend: "Empty", content: "" },
    tile_grass: { legend: "Grass", content: "", content_on_top: true, effect: "effect_grass" },
    tile_fence: { legend: "Fence", content: "", content_on_top: true },
    tile_fence_bound: { render_as: "tile_fence", legend: "Fence Bound", content: "", content_on_top: true },
    tile_healing: { legend: "Healing", content: "", content_on_top: true, effect: "effect_healing" },
    tile_bigforce: { legend: "Big force", content: "F+", content_on_top: true, effect: "effect_buff" },
    tile_defup: { legend: "Def up", content: "def+", content_on_top: true, effect: "effect_buff" },
    tile_gazebo: { legend: "Gazebo", content: "air+", content_on_top: true, effect: "effect_buff" },
    tile_infection: { legend: "Infection", content: "", effect: "effect_infection" },
    tile_volcano: { legend: "Volcano", content: "", effect: "effect_volcano" },
    tile_volcano_emp: { legend: "Volcano", content: "", effect: "effect_volcano" },
    tile_corrosion: { legend: "Corrosion", content: "", effect: "effect_corrosion" },
    tile_defbreak: { legend: "Def break", content: "", effect: "effect_corrosion" },
    tile_shallowwater: { legend: "Deep water", content: "", effect: "effect_shallowwater" },
    tile_deepwater: { legend: "Deep water", content: "", effect: "effect_deepwater" },
    tile_deepsea: { legend: "Deep sea", content: "", effect: "effect_deepwater" },
    tile_volspread: { legend: "Volcano spread", content: "", effect: "effect_volspread" },
    tile_smog: { legend: "Smoke generator", content: "", effect: "effect_smog" },
    tile_wooden_wall: { legend: "Wooden wall", content: "WW" },
    tile_yinyang_road: { legend: "YinYang road", content: "", effect: "effect_yinyang_road" },
    tile_yinyang_wall: { legend: "YinYang wall", content: "", effect: "effect_yinyang_wall" },
    tile_yinyang_switch: { legend: "YinYang switch", content: "", effect: "effect_yinyang_switch" },
    tile_poison: { legend: "Poison", content: "", effect: "effect_poison" },
    tile_icestr: { legend: "Ice", content: "", effect: "effect_ice" },
    tile_icetur_lb: { legend: "Ice corner", content: "", effect: "effect_ice" },
    tile_icetur_lt: { legend: "Ice corner", content: "", effect: "effect_ice" },
    tile_icetur_rb: { legend: "Ice corner", content: "", effect: "effect_ice" },
    tile_icetur_rt: { legend: "Ice corner", content: "", effect: "effect_ice" },
    tile_aircraft: { legend: "Aircraft", content: "" },
    tile_reed: { legend: "Reed", content: "", effect: "effect_reed" },
    tile_reedf: { legend: "Reed forbidden", content: "", effect: "effect_reed" },
    tile_reedw: { legend: "Reed wall", content: "", effect: "effect_reed" },
    tile_mire: { legend: "Mire", content: "", effect: "effect_mire" },
    tile_passable_wall: { legend: "Passable Wall", content: "" },
    tile_passable_wall_forbidden: { legend: "Passable Wall", content: "" },
    tile_stairs: { legend: "Stairs", content: "↕" },
    tile_grvtybtn: { legend: "Gravity Button", content: "" },
    tile_ristar_road: { legend: "Ristar Road", content: "", effect: "effect_ristar_road" },
    tile_ristar_road_forbidden: { legend: "Ristar Road", content: "", effect: "effect_ristar_road" },
    tile_start_cooperate: { render_as: "tile_start", content: "" },
    tile_end_cooperate: { render_as: "tile_end", content: "" },
    tile_rcm_operator: { render_as: "tile_road", legend: "Recommended Operator", content: "" },
    tile_rcm_crate: { render_as: "tile_road", legend: "Recommended Crate", content: "" },
    tile_crate: { legend: "Box", content: "", effect: "effect_crate" },
};

export const BOX_TILES: string[] = ["tile_start", "tile_flystart", "tile_end", "tile_crate", "tile_start_cooperate", "tile_end_cooperate"];
export const SMALL_DEPLOYABLE: string[] = ["tile_wall", "tile_road", "tile_healing", "tile_grass", "tile_gazebo", "tile_defup", "tile_bigforce", "tile_yinyang_road", "tile_yinyang_wall", "tile_icestr", "tile_passable_wall"];
