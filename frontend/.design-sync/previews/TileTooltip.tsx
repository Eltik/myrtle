import { RoomTile, TileHint, TileTooltip, TooltipProvider } from "frontend";
import { type CSSProperties, type ReactNode, useEffect, useRef } from "react";

/*
 * TileTooltip is the hover hint used everywhere on the base board: a room
 * tile wraps its whole button in one (label = `<TileHint>`), each crew chip
 * wraps itself in one (label = the operator's name, plus "joins / leaves this
 * shift" at a shift boundary), and the room popover's ledger chips reuse it.
 * The tooltip has hover-only open state, so the open stories hover their
 * trigger on mount.
 */
interface ICrew {
    id: string;
    name: string;
    skills: never[];
    change?: "added" | "removed";
}

const op = (id: string, name: string, change?: "added" | "removed"): ICrew => ({ id, name, skills: [], change });

const TRADING = (operators: ICrew[]) => ({
    slotId: "slot_5",
    kind: "flexible" as const,
    facility: "TRADING" as const,
    name: "Trading Post",
    level: 3,
    maxPhase: 3,
    built: true,
    operators,
    seats: 3,
    col: 1,
    row: 1,
    w: 2,
    h: 1,
});

const CREW = [op("char_272_strong", "Jaye"), op("char_1028_texas2", "Texas the Omertosa"), op("char_140_whitew", "Lappland")];
const SHIFT = [op("char_1028_texas2", "Texas the Omertosa"), op("char_140_whitew", "Lappland"), op("char_4032_provs", "Proviso", "added"), op("char_272_strong", "Jaye", "removed")];

/** The board's stage, padded above so the popup (side: top) lands inside the story. */
const Stage = ({ children }: { children: ReactNode }) => (
    <div style={{ width: "fit-content", borderRadius: 6, background: "#191919", padding: "72px 110px 22px" }}>
        <div style={{ "--riic-unit": "32px", display: "grid", gap: 3, width: "max-content", gridTemplateColumns: "calc(2 * var(--riic-unit)) calc(2 * var(--riic-unit))", gridAutoRows: "calc(2 * var(--riic-unit))" } as CSSProperties}>{children}</div>
    </div>
);

/**
 * Hovers the first element matching `selector` once mounted. Under a
 * `TooltipProvider delay={0}` the hover interaction opens on the enter itself
 * instead of after the 600 ms rest, so the popup is up before the capture.
 */
function HoverOnMount({ selector, children }: { selector: string; children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let second = 0;
        const first = requestAnimationFrame(() => {
            second = requestAnimationFrame(() => {
                const el = ref.current?.querySelector<HTMLElement>(selector);
                if (!el) return;
                const box = el.getBoundingClientRect();
                const at = { bubbles: true, clientX: box.left + box.width / 2, clientY: box.top + box.height / 2 };
                el.dispatchEvent(new PointerEvent("pointerover", { ...at, pointerType: "mouse" }));
                el.dispatchEvent(new MouseEvent("mouseover", at));
                el.dispatchEvent(new MouseEvent("mouseenter", { ...at, bubbles: false }));
                el.dispatchEvent(new MouseEvent("mousemove", at));
            });
        });
        return () => {
            cancelAnimationFrame(first);
            cancelAnimationFrame(second);
        };
    }, [selector]);
    return (
        <TooltipProvider delay={0}>
            <div ref={ref}>{children}</div>
        </TooltipProvider>
    );
}

/** Hovering a room tile: the popup carries `<TileHint>` - room name over its level - centred above the tile. */
export const OverTile = () => (
    <HoverOnMount selector='button[data-slot-id="slot_5"]'>
        <Stage>
            <RoomTile tile={TRADING(CREW)} />
        </Stage>
    </HoverOnMount>
);

/** Hovering a crew chip: the label is the operator's name, and at a shift boundary what they do this shift. */
export const OverCrewChip = () => (
    <HoverOnMount selector='[data-change="added"]'>
        <Stage>
            <RoomTile tile={TRADING(SHIFT)} />
        </Stage>
    </HoverOnMount>
);

/** Composed by hand: any element as the trigger, any node as the label - here a ledger chip from the room popover. */
export const Closed = () => (
    <div className="flex flex-col gap-3">
        <div className="flex items-center gap-3">
            <TileTooltip label={<TileHint tile={TRADING(CREW)} />}>
                <span className="rounded border border-border px-2 py-1 font-mono text-[11px] text-foreground">slot_5 · Trading Post</span>
            </TileTooltip>
            <TileTooltip label="Jaye">
                <span className="rounded border border-border px-2 py-1 text-[11px] text-foreground">Jaye's chip</span>
            </TileTooltip>
            <TileTooltip label={<span className="block max-w-56">A count skill's marginal is spread over the skills it counts, its own included.</span>}>
                <span className="font-mono font-semibold text-[10px] text-foreground tabular-nums underline decoration-dotted underline-offset-2">+30% · +10% value</span>
            </TileTooltip>
        </div>
        <p className="text-[11px] text-muted-foreground">Resting state - the popup opens on hover or keyboard focus.</p>
    </div>
);
