import type * as React from "react";
import { formatNationId, formatProfession, parseOperatorName } from "#/lib/utils";
import { operatorRarity } from "../helpers";
import type { IOperatorBirthday } from "../types";
import { OpChip } from "./OpChip";
import { Stars } from "./Stars";

/** Detailed operator line: portrait tile · name (+ appellation) · class·nation · rarity stars. */
export function OperatorRow({ birthday }: { birthday: IOperatorBirthday }): React.ReactElement {
    const { displayName, subtitle } = parseOperatorName(birthday.operator.name);
    return (
        <div className="grid grid-cols-[auto_1fr_auto] items-center gap-3.5 rounded-[10px] px-3 py-2.5 transition-colors hover:bg-accent">
            <OpChip operator={birthday.operator} size="lg" />
            <div className="min-w-0">
                <div className="truncate font-sans font-semibold text-[14.5px] text-foreground tracking-[-0.005em]">
                    {displayName}
                    {subtitle && <span className="ml-1.5 font-normal text-[12px] text-muted-foreground">{subtitle}</span>}
                </div>
                <div className="truncate font-medium font-mono text-[11.5px] text-muted-foreground uppercase tracking-[0.06em]">
                    {formatProfession(birthday.operator.profession)} · {formatNationId(birthday.operator.nationId)}
                </div>
            </div>
            <Stars rarity={operatorRarity(birthday.operator)} />
        </div>
    );
}
