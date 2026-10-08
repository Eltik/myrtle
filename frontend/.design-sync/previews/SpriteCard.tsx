import { SpriteCard } from "frontend";
import type { ReactNode } from "react";

// One character in the Stories > Characters grid: the head-cropped sprite
// thumb, the primary name, an Operator / NPC badge, story and alias counts.
// Entries are the live `/api/story/sprites` index.

const KALTSIT = {"base":"avg_003_kalts_1","kind":"operator","charId":"char_003_kalts","operatorName":"Kal'tsit","variant":"1","names":[{"name":"Kal'tsit","count":1531.0}],"noise":14.5,"lines":1545.5,"storyCount":69,"firstSeen":1728561600,"firstStory":"main_14_level_st_14-01","firstOrder":315,"variantCount":16,"thumb":{"key":"#1$1","bodyUrl":"/textures/avg/characters/avg_003_kalts_1/avg_003_kalts_1$1.png","faceUrl":"/textures/avg/characters/avg_003_kalts_1/1$1.png","facePos":{"x":551.0,"y":62.0,"w":136.0,"h":181.0},"bodySize":{"w":1280.0,"h":1280.0},"plate":{"x":0.0,"y":180.0,"w":890.0,"h":890.0},"wholeBody":false,"uses":600}};
const CLOSURE = {"base":"avg_007_closre_1","kind":"npc","names":[{"name":"Closure","count":57.0}],"noise":2.0,"lines":59.0,"storyCount":1,"firstStory":"story_weedy_set_1_story_1","firstOrder":1869,"variantCount":5,"thumb":{"key":"#1$1","bodyUrl":"/textures/avg/characters/avg_007_closre_1/avg_007_closre_1$1.png","faceUrl":"/textures/avg/characters/avg_007_closre_1/1$1.png","facePos":{"x":505.0,"y":22.0,"w":142.0,"h":143.0},"bodySize":{"w":1024.0,"h":1024.0},"plate":{"x":-20.0,"y":165.0,"w":860.0,"h":860.0},"wholeBody":false,"uses":34}};
const EXECUTOR = {"base":"avg_1032_excu2_1","kind":"operator","charId":"char_1032_excu2","operatorName":"Executor the Ex Foedere","variant":"1","names":[{"name":"Federico","count":1097.0},{"name":"Executor","count":116.0}],"noise":65.5,"lines":1278.5,"storyCount":53,"firstSeen":1703160000,"firstStory":"act26side_level_act26side_st01","firstOrder":899,"variantCount":11,"thumb":{"key":"#1$1","bodyUrl":"/textures/avg/characters/avg_1032_excu2_1/avg_1032_excu2_1$1.png","faceUrl":"/textures/avg/characters/avg_1032_excu2_1/1$1.png","facePos":{"x":502.0,"y":95.0,"w":68.0,"h":76.0},"bodySize":{"w":1024.0,"h":1024.0},"plate":{"x":0.0,"y":110.0,"w":1200.0,"h":1200.0},"wholeBody":false,"uses":551}};
const SARKAZ = {"base":"avg_npc_053","kind":"npc","names":[{"name":"Sarkaz Mercenary","count":339.0},{"name":"Sarkaz Warrior","count":286.0},{"name":"Sarkaz Teacher","count":80.0},{"name":"Sarkaz Boiler Worker","count":66.0},{"name":"Old Sarkaz Hunter","count":60.0},{"name":"Sarkaz Soldier","count":44.0},{"name":"Sarkaz Traitor","count":43.0},{"name":"Beleaguered Mercenary","count":31.0}],"noise":247.5,"lines":1196.5,"storyCount":78,"firstSeen":1603897200,"firstStory":"main_7_level_st_07-01","firstOrder":127,"variantCount":1,"thumb":{"key":"#1$1","bodyUrl":"/textures/avg/characters/avg_npc_053/avg_npc_053.png","bodySize":{"w":1024.0,"h":1024.0},"plate":{"x":0.0,"y":180.0,"w":1024.0,"h":1024.0},"wholeBody":true,"uses":1123}};
/** An entry with no thumb: the card falls back to its folder id. */
const NO_THUMB = { ...CLOSURE, base: "avg_npc_1402", names: [{ name: "Lungmen Guard", count: 12 }], storyCount: 2, thumb: null };

const noop = () => {};
const Cell = ({ children }: { children: ReactNode }) => <div className="w-44 p-4">{children}</div>;

/** An operator's sprite. */
export const Operator = () => (
    <Cell>
        <SpriteCard entry={KALTSIT} onOpen={noop} />
    </Cell>
);

/** An NPC with a single name. */
export const Npc = () => (
    <Cell>
        <SpriteCard entry={CLOSURE} onOpen={noop} />
    </Cell>
);

/** As the grid lays them out: operator with an alias, NPC with eight names, and the no-thumb fallback. */
export const GridRow = () => (
    <div className="grid w-full max-w-2xl grid-cols-4 gap-3 p-4">
        <SpriteCard entry={KALTSIT} onOpen={noop} />
        <SpriteCard entry={EXECUTOR} onOpen={noop} />
        <SpriteCard entry={SARKAZ} onOpen={noop} />
        <SpriteCard entry={NO_THUMB} onOpen={noop} />
    </div>
);
