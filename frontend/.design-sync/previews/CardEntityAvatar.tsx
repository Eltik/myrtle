import { CardEntityAvatar } from "frontend";
import type { ReactNode } from "react";

// One tile's face in a tier-list card thumbnail: an operator's avatar, or any
// other kind's own art (`kind` + `icon` + `fit` set). It fills a sized wrapper,
// here a rounded muted square. Ids and icon paths are live
// `/api/tier-lists/<slug>` placements; operator ids are checked against
// `/api/operators/index`.

const Tile = ({ children, label }: { children: ReactNode; label: string }) => (
    <figure className="m-0 flex flex-col items-center gap-1.5">
        <span className="inline-flex size-14 items-center justify-center overflow-hidden rounded-lg bg-muted font-sans font-semibold text-foreground text-lg">
            {children}
        </span>
        <figcaption className="font-mono text-[10px] text-muted-foreground">{label}</figcaption>
    </figure>
);

/** An operator: the roster avatar. */
export const Operator = () => (
    <div className="p-4">
        <Tile label="Surtr">
            <CardEntityAvatar op={{ id: "char_350_surtr", name: "Surtr", rarity: 6, role: "Guard", arch: "Arts Fighter" }} />
        </Tile>
    </div>
);

/** Mixed kinds as a mixed list's thumbnail shows them: operator, skin, enemy, event banner, faction glyph. */
export const MixedKinds = () => (
    <div className="flex flex-wrap gap-3 p-4">
        <Tile label="operator">
            <CardEntityAvatar op={{ id: "char_4064_mlynar", name: "Młynar", rarity: 6, role: "Guard", arch: "Liberator" }} />
        </Tile>
        <Tile label="skin">
            <CardEntityAvatar op={{ id: "char_293_thorns@boc#8", name: "Blade-cleaved Tides", rarity: 6, role: "", arch: "", kind: "skin", icon: "/avatar/char_293_thorns_boc%238", fit: "cover" }} />
        </Tile>
        <Tile label="enemy">
            <CardEntityAvatar op={{ id: "enemy_1501_demonk", name: "Sarkaz Centurion", rarity: 0, role: "", arch: "", kind: "enemy", icon: "/enemy-icon/enemy_1501_demonk", fit: "cover" }} />
        </Tile>
        <Tile label="event">
            <CardEntityAvatar op={{ id: "act39side", name: "Exodus from the Pale Sea", rarity: 0, role: "", arch: "", kind: "event", icon: "/event-image/act39side", fit: "cover" }} />
        </Tile>
        <Tile label="faction">
            <CardEntityAvatar op={{ id: "rhodes", name: "Rhodes Island", rarity: 0, role: "", arch: "", kind: "faction", icon: "/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png", fit: "glyph" }} />
        </Tile>
    </div>
);

/** A non-operator with no art in the extract: its initials stand in. */
export const NoArt = () => (
    <div className="p-4">
        <Tile label="no art">
            <CardEntityAvatar op={{ id: "rogue_3", name: "Sarkaz's Furnaceside Fables", rarity: 0, role: "", arch: "", kind: "integrated_strategies", icon: null, fit: "cover" }} />
        </Tile>
    </div>
);
