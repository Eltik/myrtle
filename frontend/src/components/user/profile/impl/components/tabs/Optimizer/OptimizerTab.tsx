import { useState } from "react";
import { Tabs, TabsList, TabsPanel, TabsTab } from "#/components/ui/tabs";
import type { IRosterEntry } from "#/lib/api/user";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IOperatorListItem } from "#/types/operators";
import { ComingSoon } from "./ComingSoon";
import { OPTIMIZERS } from "./optimizers";
import type { messages as optimizerMessages } from "./optimizers.messages";

interface IProps {
    uid: string;
    roster: IRosterEntry[];
    operatorsStatic: IOperatorListItem[];
}

export function OptimizerTab({ uid, roster, operatorsStatic }: IProps) {
    /** The optimizer names and blurbs are declared in `optimizers.messages.ts`. */
    const t: TypedT<typeof optimizerMessages> = useT("user");
    const [active, setActive] = useState(OPTIMIZERS[0].id);

    return (
        <Tabs className="gap-3" onValueChange={(value) => setActive(String(value))} value={active}>
            <TabsList variant="underline">
                {OPTIMIZERS.map((optimizer) => (
                    <TabsTab key={optimizer.id} value={optimizer.id}>
                        {t(optimizer.labelKey)}
                    </TabsTab>
                ))}
            </TabsList>

            {OPTIMIZERS.map((optimizer) => (
                <TabsPanel className="flex flex-col gap-3" key={optimizer.id} keepMounted value={optimizer.id}>
                    <p className="text-[12.5px] text-muted-foreground">{t(optimizer.blurbKey)}</p>
                    {optimizer.Component ? <optimizer.Component operatorsStatic={operatorsStatic} roster={roster} uid={uid} /> : <ComingSoon def={optimizer} />}
                </TabsPanel>
            ))}
        </Tabs>
    );
}
