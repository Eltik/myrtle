import { ThumbTierRow } from "frontend";
import type { ReactNode } from "react";

// One tier row of a tier-list browse card's thumbnail: the coloured tier pill,
// as many tiles as the row's measured width holds, and a "+N" badge for the
// rest. It takes the card's CSS module as `styles`; BrowseCard passes its own,
// whose classes ship in the bundle as `BrowseCard_*`, so the preview hands it
// the same map inside the same `thumb` frame (`data-rows` sets tile size).
// The frame rests desaturated, as an un-hovered card does.
// Operator ids are checked against `/api/operators/index`; non-operator tiles
// are live tier-list placements.

const STYLES = { tierRow: "BrowseCard_tierRow", tierPill: "BrowseCard_tierPill", tierOps: "BrowseCard_tierOps", op: "BrowseCard_op", opOverflow: "BrowseCard_opOverflow" };
const RED = "oklch(0.62 0.21 24)";
const ORANGE = "oklch(0.70 0.17 50)";
const YELLOW = "oklch(0.78 0.15 92)";

const op = (id: string, name: string) => ({ id, name, rarity: 6, role: "", arch: "" });

const S_TIER = [op("char_4064_mlynar", "Młynar"), op("char_350_surtr", "Surtr"), op("char_2012_typhon", "Typhon"), op("char_377_gdglow", "Goldenglow"), op("char_4116_blkkgt", "Degenbrecher"), op("char_4087_ines", "Ines"), op("char_4133_logos", "Logos"), op("char_4123_ela", "Ela"), op("char_103_angel", "Exusiai"), op("char_010_chen", "Ch'en"), op("char_263_skadi", "Skadi"), op("char_202_demkni", "Saria"), op("char_179_cgbird", "Nightingale"), op("char_147_shining", "Shining")];
const A_TIER = [op("char_358_lisa", "Suzuran"), op("char_136_hsguma", "Hoshiguma"), op("char_128_plosis", "Ptilopsis"), op("char_171_bldsk", "Warfarin")];
const EVENTS = [
    { id: "act43side", name: "Act or Die", rarity: 0, role: "", arch: "", kind: "event" as const, icon: "/event-image/act43side", fit: "cover" as const },
    { id: "act39side", name: "Exodus from the Pale Sea", rarity: 0, role: "", arch: "", kind: "event" as const, icon: "/event-image/act39side", fit: "cover" as const },
    { id: "main_10", name: "Shatterpoint", rarity: 0, role: "", arch: "", kind: "main_story" as const, icon: "/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png", fit: "cover" as const },
    { id: "enemy_1501_demonk", name: "Sarkaz Centurion", rarity: 0, role: "", arch: "", kind: "enemy" as const, icon: "/enemy-icon/enemy_1501_demonk", fit: "cover" as const },
];

/** The card's thumbnail frame: the light theme (as `<html class="light">` sets it), card width. */
const Thumb = ({ rows, children }: { rows: number; children: ReactNode }) => (
    <div className="light p-4">
        <div className="BrowseCard_thumb overflow-hidden rounded-lg" data-rows={rows} style={{ width: 340 }}>
            {children}
        </div>
    </div>
);

/** A crowded top tier on a two-row card: tiles until the row is full, then "+N". */
export const Overflow = () => (
    <Thumb rows={2}>
        <ThumbTierRow row={{ name: "S", color: RED, operators: S_TIER, fallbackVisible: 6 }} styles={STYLES} title="Tier S" />
        <ThumbTierRow row={{ name: "A", color: ORANGE, operators: A_TIER, fallbackVisible: 6 }} styles={STYLES} title="Tier A" />
    </Thumb>
);

/** A single-row card: the largest tiles and pill. */
export const SingleRow = () => (
    <Thumb rows={1}>
        <ThumbTierRow row={{ name: "Required", color: YELLOW, operators: [op("char_263_skadi", "Skadi"), op("char_202_demkni", "Saria"), op("char_358_lisa", "Suzuran")], fallbackVisible: 4 }} styles={STYLES} title="Tier Required" />
    </Thumb>
);

/** A mixed list's row: event banners, a story poster and an enemy as card tiles. */
export const MixedKinds = () => (
    <Thumb rows={3}>
        <ThumbTierRow row={{ name: "S", color: RED, operators: EVENTS, fallbackVisible: 7 }} styles={STYLES} title="Tier S" />
        <ThumbTierRow row={{ name: "A", color: ORANGE, operators: A_TIER, fallbackVisible: 7 }} styles={STYLES} title="Tier A" />
        <ThumbTierRow row={{ name: "B", color: YELLOW, operators: S_TIER.slice(8), fallbackVisible: 7 }} styles={STYLES} title="Tier B" />
    </Thumb>
);
