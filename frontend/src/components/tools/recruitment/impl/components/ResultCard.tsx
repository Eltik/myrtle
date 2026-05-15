import type * as React from "react";
import { Badge } from "#/components/ui/badge";
import { Card, CardHeader, CardPanel } from "#/components/ui/card";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
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
            <CardHeader className="flex grid-rows-1 flex-row items-center justify-between gap-3 px-4 py-3">
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
        <HoverCard>
            <HoverCardTrigger
                render={
                    <li className={cn("flex cursor-pointer items-center gap-2 rounded-md border px-2 py-1.5 transition-colors", colors?.border, colors?.bg, colors?.hoverBg, colors?.hoverBorder)}>
                        <span aria-hidden="true" className="inline-flex size-8 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted font-semibold text-[11px]">
                            <OperatorAvatar charId={operator.id} name={operator.name} />
                        </span>
                        <div className="min-w-0 flex-1">
                            <div className="truncate font-medium text-[13px] text-foreground leading-tight">{operator.name}</div>
                            <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                                <span className={cn("font-mono", colors?.text)}>{getStarsDisplay(operator.rarity)}</span>
                                <span>·</span>
                                <span className="truncate">{profession}</span>
                            </div>
                        </div>
                    </li>
                }
            />
            <HoverCardContent className="w-max max-w-72 p-3">
                <div className="flex flex-col gap-2">
                    <div className="font-medium text-[12px] text-muted-foreground">Tags</div>
                    {operator.tagList.length > 0 ? (
                        <div className="flex flex-wrap items-center gap-1.5">
                            {operator.tagList.map((tag) => (
                                <Badge key={tag} variant="outline" size="default">
                                    {tag}
                                </Badge>
                            ))}
                        </div>
                    ) : (
                        <div className="text-[12px] text-muted-foreground italic">No tags</div>
                    )}
                </div>
            </HoverCardContent>
        </HoverCard>
    );
}
