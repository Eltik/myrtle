import { OperatorAvatar } from "#/components/ui/operator-avatar";
import type { ITile } from "#/lib/base/layout";
import { isProduction, powerOf } from "#/lib/base/layout";
import { useOptimizerApi } from "./optimizer-context";

function Stat({ label, value }: { label: string; value: string }) {
    return (
        <div className="flex flex-col gap-0.5">
            <dt className="text-[10px] text-muted-foreground uppercase tracking-wider">{label}</dt>
            <dd className="font-mono font-semibold text-[12px] tabular-nums">{value}</dd>
        </div>
    );
}

export function RoomPopover({ tile }: { tile: ITile }) {
    const api = useOptimizerApi();
    const room = api.boardRooms.find((r) => r.slot_id === tile.slotId);
    const scored = api.evaluation?.assignment.rooms.find((r: { slot_id: string }) => r.slot_id === tile.slotId);

    const producesOwnOutput = scored !== undefined;
    const unstaffed = tile.seats > 0 && tile.operators.length === 0;

    const formula = room?.formula_type ? api.formulas.find((f) => f.formula_type === room.formula_type) : undefined;
    const power = room ? powerOf(room, api.catalog) : 0;
    const vacancies = Math.max(0, tile.seats - tile.operators.length);

    return (
        <div className="flex w-62 flex-col gap-3">
            <header className="flex items-baseline justify-between gap-2">
                <h2 className="font-semibold text-[13px] text-foreground">{tile.name}</h2>
                <span className="font-mono text-[11px] text-muted-foreground tabular-nums">
                    Lv {tile.level}/{tile.maxPhase}
                </span>
            </header>

            <dl className="flex flex-wrap gap-x-6 gap-y-2">
                {tile.seats > 0 && <Stat label="Staffed" value={`${tile.operators.length}/${tile.seats}`} />}
                {power !== 0 && <Stat label={power > 0 ? "Generates" : "Draws"} value={`${Math.abs(power)} kW`} />}
                {formula && <Stat label="Producing" value={formula.label} />}
                {scored && <Stat label="Efficiency" value={`${Math.round(scored.total_efficiency)}%`} />}
                {scored && isProduction(tile.facility ?? "") && scored.yield_lmd_per_day > 0 && <Stat label="LMD / day" value={Math.round(scored.yield_lmd_per_day).toLocaleString()} />}
            </dl>

            {tile.seats > 0 && (
                <section className="flex flex-col gap-1 border-border border-t pt-2">
                    {tile.operators.map((op) => (
                        <div key={op.id} className="flex items-center gap-2">
                            <span className="size-6 shrink-0 overflow-hidden rounded-sm bg-muted text-center font-bold text-[9px] leading-6">
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className="min-w-0 flex-1 truncate text-[12px]">{op.name}</span>
                        </div>
                    ))}
                    {vacancies > 0 && <p className="text-[11px] text-muted-foreground">{vacancies === tile.seats ? "Nobody is working here." : `${vacancies} seat${vacancies === 1 ? "" : "s"} open.`}</p>}
                </section>
            )}

            {!producesOwnOutput && tile.seats > 0 && (
                <p className="border-border border-t pt-2 text-[11px] text-muted-foreground">
                    {api.evaluationError ? "This layout could not be scored." : api.evaluating ? "Scoring…" : unstaffed ? "An empty room produces nothing and buffs nothing." : "Only producing rooms report an efficiency. This crew contributes through the bonuses they cast elsewhere."}
                </p>
            )}
        </div>
    );
}
