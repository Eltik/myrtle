import { useState } from "react";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { ComingSoon } from "./ComingSoon";
import { OPTIMIZERS } from "./optimizers";
import { SegmentedTabs } from "./SegmentedTabs";

interface IProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function OptimizerTab({ uid, roster, operatorsStatic }: IProps) {
    const [activeId, setActiveId] = useState(OPTIMIZERS[0].id);
    const active = OPTIMIZERS.find((o) => o.id === activeId) ?? OPTIMIZERS[0];

    return (
        <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
                <SegmentedTabs active={active.id} label="Optimizers" onChange={setActiveId} segments={OPTIMIZERS.map((o) => ({ id: o.id, label: o.label }))} />
                <p className="text-[12.5px] text-muted-foreground">{active.blurb}</p>
            </div>

            {active.Component ? <active.Component operatorsStatic={operatorsStatic} roster={roster} uid={uid} /> : <ComingSoon def={active} />}
        </div>
    );
}
