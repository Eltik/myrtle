import { useState } from "react";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import type { IRosterEntry } from "#/lib/api/user";
import type { IOperatorListItem } from "#/types/operators";
import { ComingSoon } from "./ComingSoon";
import { OPTIMIZERS } from "./optimizers";

interface IProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function OptimizerTab({ uid, roster, operatorsStatic }: IProps) {
    const [active, setActive] = useState(OPTIMIZERS[0].id);

    return (
        <Tabs className="gap-3" onValueChange={(value) => setActive(String(value))} value={active}>
            <TabsList variant="underline">
                {OPTIMIZERS.map((optimizer) => (
                    <TabsTab key={optimizer.id} value={optimizer.id}>
                        {optimizer.label}
                    </TabsTab>
                ))}
            </TabsList>

            {OPTIMIZERS.map((optimizer) => (
                <TabsPanel className="flex flex-col gap-3" key={optimizer.id} keepMounted value={optimizer.id}>
                    <p className="text-[12.5px] text-muted-foreground">{optimizer.blurb}</p>
                    {optimizer.Component ? <optimizer.Component operatorsStatic={operatorsStatic} roster={roster} uid={uid} /> : <ComingSoon def={optimizer} />}
                </TabsPanel>
            ))}
        </Tabs>
    );
}
