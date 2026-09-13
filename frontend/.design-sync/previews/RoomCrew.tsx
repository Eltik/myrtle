import { RoomCrew } from "frontend";
import type { CSSProperties, ReactNode } from "react";

/*
 * RoomCrew is the avatar strip pinned to a room tile's bottom-right corner:
 * one 16px chip per seated operator, then a dashed outline per open seat
 * (`seats - operators.length`). It is absolutely positioned, so it needs the
 * tile (or here, a slab of the same size) as its positioned parent. A tile
 * with `seats: 0` renders nothing.
 */
interface ICrew {
    id: string;
    name: string;
    skills: never[];
    change?: "added" | "removed";
}

const op = (id: string, name: string, change?: "added" | "removed"): ICrew => ({ id, name, skills: [], change });

const JAYE = op("char_272_strong", "Jaye");
const TEXAS2 = op("char_1028_texas2", "Texas the Omertosa");
const LAPPLAND = op("char_140_whitew", "Lappland");
const PROVISO = op("char_4032_provs", "Proviso");
const PINECONE = op("char_440_pinecn", "Pinecone");
const ANSEL = op("char_212_ansel", "Ansel");
const HIBISCUS = op("char_120_hibisc", "Hibiscus");
const ORCHID = op("char_278_orchid", "Orchid");
const GREYY = op("char_253_greyy", "Greyy");
const SHAW = op("char_277_sqrrel", "Shaw");

type Facility = "TRADING" | "MANUFACTURE" | "POWER" | "DORMITORY";
const ROOMS: Record<Facility, { name: string; accent?: string; w: number }> = {
    TRADING: { name: "Trading Post", accent: "#38c9fe", w: 2 },
    MANUFACTURE: { name: "Factory", accent: "#fec904", w: 2 },
    POWER: { name: "Power Plant", accent: "#c3ff43", w: 2 },
    DORMITORY: { name: "Dormitory", w: 3 },
};

/** The slice of an ITile the strip reads: `seats` and `operators`. */
const tile = (slotId: string, facility: Facility, level: number, seats: number, operators: ICrew[]) => ({
    slotId,
    kind: "flexible" as const,
    facility,
    name: ROOMS[facility].name,
    level,
    maxPhase: 3,
    built: true,
    operators,
    seats,
    col: 1,
    row: 1,
    w: ROOMS[facility].w,
    h: 1,
});

/** A room tile's slab (64px tall, two or three 64px tracks wide) with the name row, as the crew's positioned parent. */
const Slab = ({ facility, label, children }: { facility: Facility; label: string; children: ReactNode }) => (
    <div style={{ "--tile-accent": ROOMS[facility].accent, position: "relative", width: ROOMS[facility].w * 64, height: 64, background: "#323232", border: "1px solid rgb(255 255 255 / 0.08)", color: "#fff" } as CSSProperties}>
        {ROOMS[facility].accent && <span style={{ position: "absolute", top: 0, bottom: 0, left: 0, width: 5, background: "var(--tile-accent)" }} />}
        <span style={{ position: "absolute", top: 5, left: 12, fontSize: 11, fontWeight: 700, letterSpacing: 0.1, lineHeight: 1.15 }}>{label}</span>
        {children}
    </div>
);

const Stage = ({ children }: { children: ReactNode }) => (
    <div className="flex flex-wrap items-start gap-3" style={{ width: "fit-content", borderRadius: 6, background: "#191919", padding: "22px 16px" }}>
        {children}
    </div>
);

/** Fully staffed rooms: three trading chips, a single power-plant seat, five dorm beds. */
export const Staffed = () => (
    <Stage>
        <Slab facility="TRADING" label="Trading Post">
            <RoomCrew tile={tile("slot_5", "TRADING", 3, 3, [JAYE, TEXAS2, LAPPLAND])} />
        </Slab>
        <Slab facility="POWER" label="Power Plant">
            <RoomCrew tile={tile("slot_7", "POWER", 3, 1, [GREYY])} />
        </Slab>
        <Slab facility="DORMITORY" label="Dormitory">
            <RoomCrew tile={tile("slot_9", "DORMITORY", 5, 5, [ANSEL, HIBISCUS, ORCHID, op("char_181_flower", "Perfumer"), op("char_237_gravel", "Gravel")])} />
        </Slab>
    </Stage>
);

/** Open seats trail the crew as dashed outlines - one per vacancy, and all five when nobody is assigned. */
export const OpenSeats = () => (
    <Stage>
        <Slab facility="MANUFACTURE" label="Factory">
            <RoomCrew tile={tile("slot_15", "MANUFACTURE", 3, 3, [PINECONE])} />
        </Slab>
        <Slab facility="TRADING" label="Trading Post">
            <RoomCrew tile={tile("slot_14", "TRADING", 2, 2, [])} />
        </Slab>
        <Slab facility="DORMITORY" label="Dormitory">
            <RoomCrew tile={tile("slot_20", "DORMITORY", 4, 5, [ANSEL, HIBISCUS, ORCHID])} />
        </Slab>
    </Stage>
);

/** Shift marks: `added` rings the incoming chip in the room's accent; `removed` greys and dashes the outgoing one, listed after the crew. */
export const ShiftChange = () => (
    <Stage>
        <Slab facility="TRADING" label="Trading Post">
            <RoomCrew tile={tile("slot_14", "TRADING", 3, 3, [TEXAS2, LAPPLAND, op(PROVISO.id, PROVISO.name, "added"), op(JAYE.id, JAYE.name, "removed")])} />
        </Slab>
        <Slab facility="POWER" label="Power Plant">
            <RoomCrew tile={tile("slot_16", "POWER", 3, 1, [op(SHAW.id, SHAW.name, "added"), op(GREYY.id, GREYY.name, "removed")])} />
        </Slab>
    </Stage>
);
