import { SlabFrame } from "frontend";

// The chrome every randomizer result sits in: a numbered rail on the left with a
// vertical kicker, and an accent hairline whose colour identifies the slot.
// Ported from ModifierSlab / SquadSlab / StageSlab, which use exactly these three
// index + kicker + accent triples.

export const StageSlot = () => (
    <SlabFrame index="01" kicker="STAGE" accent="primary">
        <p className="m-0 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.18em]">Target</p>
        <h2 className="mt-3 mb-0 font-bold text-4xl text-foreground leading-none tracking-tight">R8-7</h2>
        <p className="mt-2 text-muted-foreground text-sm">Baptism by Tractor Fire</p>
        <p className="mt-1.5 font-mono text-[12px] text-muted-foreground/80 uppercase tracking-[0.14em]">Roaring Flare</p>
    </SlabFrame>
);

export const SquadSlot = () => (
    <SlabFrame index="02" kicker="SQUAD" accent="lagoon">
        <p className="m-0 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.18em]">
            Deployment · <span className="text-foreground/80 normal-case tracking-normal">12/12</span>
        </p>
        <div className="mt-4 grid grid-cols-6 gap-2">
            {["char_4064_mlynar", "char_180_amgoat", "char_202_demkni", "char_102_texas", "char_140_whitew", "char_128_plosis"].map((id) => (
                <img alt="" aria-hidden="true" className="aspect-square w-full rounded-md border border-border/60 object-cover" key={id} src={`https://api.myrtle.moe/api/avatar/${id}`} />
            ))}
        </div>
    </SlabFrame>
);

export const ModifierSlot = () => (
    <SlabFrame index="03" kicker="RULE" accent="palm">
        <p className="m-0 font-mono text-[11px] text-muted-foreground uppercase tracking-[0.18em]">Restriction</p>
        <h3 className="mt-3 mb-0 font-semibold text-3xl text-foreground leading-tight tracking-tight">Ranged only</h3>
        <p className="mt-2 max-w-prose text-base text-muted-foreground">Only ranged operators allowed.</p>
    </SlabFrame>
);

export const NestedRuns = () => (
    <div className="flex flex-col gap-4">
        <SlabFrame index="01" kicker="STAGE" accent="primary">
            <p className="m-0 font-semibold text-[24px] text-foreground leading-none tracking-tight">10-17</p>
            <p className="mt-1.5 text-muted-foreground text-sm">A Citadel and Its Walls · AP 24</p>
        </SlabFrame>
        <SlabFrame index="02" kicker="SQUAD" accent="lagoon">
            <p className="m-0 font-semibold text-[24px] text-foreground leading-none tracking-tight">8/12 drawn</p>
            <p className="mt-1.5 text-muted-foreground text-sm">Pool narrowed by the active restriction.</p>
        </SlabFrame>
        <SlabFrame index="03" kicker="RULE" accent="palm">
            <p className="m-0 font-semibold text-[24px] text-foreground leading-none tracking-tight">Boss solo</p>
            <p className="mt-1.5 text-muted-foreground text-sm">Only one operator may be deployed during the boss phase.</p>
        </SlabFrame>
    </div>
);
