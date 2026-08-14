import type { IPerceptionPlan } from "#/lib/api/user";
import { cn } from "#/lib/utils";
import { TEXT_KICKER, TEXT_META } from "../shared";
import { MiniChip, TEXT_TINY } from "./parts";
import { roomAccent, roomLabel } from "./roomColors";

/** The base-wide resource economy: support operators to station outside production, and the
 *  production operators they power. A niche, max-output ceiling - not a normal-base default. */
export function PerceptionSection({ plan }: { plan: IPerceptionPlan }) {
    return (
        <>
            <p className={cn(TEXT_META, "text-muted-foreground")}>Operators resting in your dormitories fuel a shared pool that supercharges specific production operators. Station the support crew, and the powered operators climb far past a normal team - a high-ceiling, high-effort strategy most players can skip.</p>
            {plan.support.length > 0 && (
                <div className="flex flex-col gap-1.5">
                    <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Station support</span>
                    <div className="flex flex-wrap gap-1">
                        {plan.support.map((s) => (
                            <MiniChip
                                key={s.operator.operator_id}
                                op={s.operator}
                                suffix={
                                    <span className={cn("font-mono", TEXT_TINY)} style={{ color: roomAccent(s.room_type).text }}>
                                        → {roomLabel(s.room_type)}
                                    </span>
                                }
                                tip={
                                    <p>
                                        Station {s.operator.name} in the {roomLabel(s.room_type)} to feed the pool.
                                    </p>
                                }
                            />
                        ))}
                    </div>
                </div>
            )}
            <div className="flex flex-col gap-1.5">
                <span className={cn(TEXT_KICKER, "text-muted-foreground")}>
                    Powers <span className="font-normal normal-case opacity-70">(peak · sustained 24/7)</span>
                </span>
                <div className="flex flex-wrap gap-1">
                    {plan.consumers.map((c) => {
                        const a = roomAccent(c.room_type);
                        return (
                            <MiniChip
                                key={c.operator.operator_id}
                                op={c.operator}
                                suffix={
                                    <span className={cn("font-mono", TEXT_TINY)}>
                                        <span style={{ color: a.strong }}>+{c.bonus_pct.toFixed(0)}%</span> <span className="text-muted-foreground/60">+{c.sustained_pct.toFixed(0)}%·24/7</span>
                                    </span>
                                }
                                tip={
                                    <p>
                                        {c.operator.name} gains +{c.bonus_pct.toFixed(0)}% at peak ({c.sustained_pct.toFixed(0)}% sustained 24/7) from the {roomLabel(c.room_type)} pool.
                                    </p>
                                }
                            />
                        );
                    })}
                </div>
            </div>
            {plan.rotation_manager && (
                <div className="flex flex-wrap items-center gap-1.5">
                    <span className={cn(TEXT_KICKER, "text-muted-foreground")}>Rotation manager</span>
                    <MiniChip op={plan.rotation_manager} suffix={<span className={cn("font-mono text-muted-foreground/70", TEXT_TINY)}>swaps morale</span>} tip={<p>{plan.rotation_manager.name} keeps the Ling/Dusk morale rotation running.</p>} />
                </div>
            )}
            {plan.needs_rotation_manager && (
                <p className={cn(TEXT_META, "rounded-md border border-amber-500/40 bg-amber-500/10 px-2 py-1.5 text-amber-500/90")}>
                    ⚠ This rotation relies on Ling/Dusk, which need a morale-swap operator (e.g. <span className="font-medium">Fiammetta</span>) to sustain. You don't have one - the figures above are a peak ceiling rather than a sustainable plan.
                </p>
            )}
        </>
    );
}
