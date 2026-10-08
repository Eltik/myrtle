import { EntityIcon } from "frontend";
import type { ReactNode } from "react";

// A non-operator's art from its resolved icon path, used where only summary
// fields travel (card previews). `fit` picks how it sits: `cover` fills the
// tile, `object` insets a transparent object, `glyph` insets further and
// inverts a white glyph for the light theme. With no icon (or a failed load) it
// typesets initials, or the whole name on a wide `tile` face. Icon paths are
// live `/api/tier-lists/<slug>` placements.

const Tile = ({ children, label, wide = false }: { children: ReactNode; label: string; wide?: boolean }) => (
    <figure className="m-0 flex flex-col items-center gap-1.5">
        <span className="inline-flex h-16 items-center justify-center overflow-hidden rounded-lg bg-muted font-sans text-foreground text-lg" style={{ width: wide ? 134 : 64 }}>{children}</span>
        <figcaption className="font-mono text-[10px] text-muted-foreground">{label}</figcaption>
    </figure>
);

/** The three fits: cover (enemy, story poster), glyph (faction, bond). */
export const Fits = () => (
    <div className="flex flex-wrap gap-3 p-4">
        <Tile label="cover"><EntityIcon kind="enemy" name="Sarkaz Centurion" icon="/enemy-icon/enemy_1501_demonk" /></Tile>
        <Tile label="cover"><EntityIcon kind="main_story" name="Shatterpoint" icon="/assets/textures/spritepack/mixstory_kv_sprites_1/kv_shatterpoint.png" /></Tile>
        <Tile label="glyph"><EntityIcon kind="faction" name="Rhodes Island" icon="/assets/textures/spritepack/ui_camp_logo_0/logo_rhodes.png" /></Tile>
        <Tile label="glyph"><EntityIcon kind="stronghold_bond" name="Agile" icon="/assets/textures/ui/autochess/%5Buc%5Dautochesscommon/icon_skillfulShip.png" /></Tile>
    </div>
);

/** An event banner on its wide tile. */
export const WideBanner = () => (
    <div className="p-4">
        <Tile label="event" wide><EntityIcon kind="event" name="Exodus from the Pale Sea" icon="/event-image/act39side" face="tile" /></Tile>
    </div>
);

/** No art in the extract: initials on a chip, the name spelled out on a wide tile. */
export const NoArt = () => (
    <div className="flex flex-wrap items-start gap-3 p-4">
        <Tile label="chip"><EntityIcon kind="enemy" name="Originium Slug" icon={null} /></Tile>
        <Tile label="chip"><EntityIcon kind="faction" name="Rhine Lab" icon={null} /></Tile>
        <Tile label="tile (wide)" wide><EntityIcon kind="event" name="Act or Die" icon={null} face="tile" /></Tile>
    </div>
);
