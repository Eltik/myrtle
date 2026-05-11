import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Card, CardHeader, CardPanel } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { cn } from "#/lib/utils";
import { PROFESSION_LABELS, RARITY_COLORS } from "../constants";
import { getStarsDisplay } from "../helpers";
import type { IRecruitableOperator, ITagCombinationResult } from "../types";

interface IResultCardProps {
    result: ITagCombinationResult;
}

export function ResultCard({ result }: IResultCardProps): React.ReactElement {
    return (
        <Card>
            <CardHeader className="flex flex-row items-center justify-between gap-3 px-4 py-3 grid-rows-1">
                <div className="flex min-w-0 flex-1 flex-wrap items-center gap-1.5">
                    {result.tagNames.map((name) => (
                        <Badge key={name} variant="outline" size="default">
                            {name}
                        </Badge>
                    ))}
                </div>
            </CardHeader>
            <CardPanel className="px-4 pt-0 pb-3">
                <ul className="grid grid-cols-1 gap-1.5 sm:grid-cols-2 lg:grid-cols-3">
                    {result.operators.map((op) => (
                        <OperatorRow key={op.id} operator={op} />
                    ))}
                </ul>
            </CardPanel>
        </Card>
    );
}

function OperatorRow({ operator }: { operator: IRecruitableOperator }): React.ReactElement {
    const colors = RARITY_COLORS[operator.rarity];
    const profession = PROFESSION_LABELS[operator.profession] ?? operator.profession;

    return (
        <li className={cn("flex items-center gap-2 rounded-md border px-2 py-1.5", colors?.border, colors?.bg)}>
            <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted text-[11px] font-semibold">
                <OperatorAvatar charId={operator.id} name={operator.name} />
            </span>
            <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-[13px] leading-tight text-foreground">{operator.name}</div>
                <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                    <span className={cn("font-mono", colors?.text)}>{getStarsDisplay(operator.rarity)}</span>
                    <span>·</span>
                    <span className="truncate">{profession}</span>
                </div>
            </div>
        </li>
    );
}
