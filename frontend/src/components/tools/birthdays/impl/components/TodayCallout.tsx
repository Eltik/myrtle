import type * as React from "react";
import { Kicker } from "#/components/ui/kicker";
import { formatProfession, parseOperatorName } from "#/lib/utils";
import { operatorRarity } from "../helpers";
import type { IOperatorBirthday } from "../types";
import { OpChip } from "./OpChip";

interface ITodayCalloutProps {
    ops: IOperatorBirthday[];
    today: Date;
}

/** Coral-tinted hero card highlighting operators whose birthday is today. Renders nothing when empty. */
export function TodayCallout({ ops, today }: ITodayCalloutProps): React.ReactElement | null {
    if (ops.length === 0) return null;

    const dateLabel = today.toLocaleDateString("en-US", { month: "long", day: "numeric", weekday: "long" });

    return (
        <div className="relative mt-5.5 overflow-hidden rounded-[18px] border border-primary/20 bg-[linear-gradient(90deg,color-mix(in_oklab,var(--primary),transparent_92%)_0%,transparent_55%)] bg-card p-5 shadow-xs/5">
            <div className="relative mb-3 flex items-center gap-2.5">
                <Kicker className="mb-0">Today</Kicker>
                <span className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">{dateLabel}</span>
            </div>
            <div className="relative flex flex-wrap gap-3.5">
                {ops.map((b) => {
                    const { displayName } = parseOperatorName(b.operator.name);
                    return (
                        <div key={b.operator.id} className="flex items-center gap-2.5 rounded-full border border-border bg-card py-2 pr-3.5 pl-2">
                            <OpChip operator={b.operator} size="lg" />
                            <div>
                                <div className="font-sans font-semibold text-[13.5px] text-foreground">{displayName}</div>
                                <div className="font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-[0.08em]">
                                    {operatorRarity(b.operator)}★ · {formatProfession(b.operator.profession)}
                                </div>
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}
