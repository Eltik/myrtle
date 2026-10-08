import { TileGrid } from "frontend";
import { useState } from "react";

// TileGrid is the background editor's gallery grid: 16:9 tiles (the backend's
// 320px JPEGs) in as many columns as fit at `minTile` px, each with a one-line
// caption under the art — the picture's own title for an Archives picture, the
// story's name for a story CG or scene. The chosen tile is ringed in the
// primary with a check badge and a primary caption. It flows in the editor's
// scroller and mounts 120 tiles a page. The S/M/L tile-size control sets
// `minTile` (S 180, M 260, L 340 px in the product).

type Kind = "archive_pic" | "story_cg" | "story_scene";

/** Live EN Story CGs of Under Tides (`/api/story/art-gallery/cg`, group act18d3): 19 pictures. */
const UNDER_TIDES = [
    {kind:  "story_cg", id:  "ac18_01", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_02", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_03_1", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_03_2", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_04", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_05", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_07", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_08", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_09", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_10", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_11_1", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_11_2", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_11_3", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_11_4", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_12_1", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_12_2", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_14", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_13_1", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
    {kind:  "story_cg", id:  "ac18_13_2", title:  "", groupId:  "act18d3", groupName:  "Under Tides", category:  "side"},
] as const;


/** Live EN Archives pictures of Near Light (`/api/story/gallery`, group act13side), the first 12 of 16. */
const NEAR_LIGHT = [
    {kind:  "archive_pic", id:  "act13side_pic_0", title:  "Long Night, Near Light", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_1", title:  "Flickering Candle, Fleeting Shadow", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_2", title:  "After Work", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_3", title:  "Uninvited Guest", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_4", title:  "The Swordbearing White-Horned Sarkaz", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_5", title:  "Balance and Imbalance", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_6", title:  "A Momentary Dance", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_7", title:  "Against All Odds", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_8", title:  "A Bell Through the Dark Night", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_9", title:  "Falling Meteor", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_10", title:  "Silverlance Enters the City", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
    {kind:  "archive_pic", id:  "act13side_pic_12", title:  "The Doctor's Battle", groupId:  "act13side", groupName:  "Near Light", category:  "side"},
] as const;

function Stage({ tiles, initial, minTile, width = 820 }: { tiles: readonly { kind: Kind; id: string; title: string; groupId: string; groupName: string; category: "side" }[]; initial: { kind: Kind; id: string } | null; minTile: number; width?: number }) {
    const [selected, setSelected] = useState(initial);
    return (
        <div style={{ width }}>
            <TileGrid tiles={tiles} selected={selected} onPick={(kind, id) => setSelected({ kind, id })} pagingKey="preview" minTile={minTile} />
        </div>
    );
}

/** Story CGs of one story at the small tile size, one chosen: ringed, checked, its caption in the primary. */
export const StoryCgsSelected = () => <Stage tiles={UNDER_TIDES.slice(0, 12)} initial={{ kind: "story_cg", id: "ac18_03_1" }} minTile={180} />;

/** Archives pictures carry their own titles in the captions. */
export const ArchivesTitled = () => <Stage tiles={NEAR_LIGHT} initial={null} minTile={180} />;

/** The large tile size: fewer, wider columns. */
export const LargeTiles = () => <Stage tiles={NEAR_LIGHT.slice(0, 4)} initial={{ kind: "archive_pic", id: "act13side_pic_0" }} minTile={340} />;
