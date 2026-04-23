import { HoverCard, HoverCardContent, HoverCardTrigger } from "#/components/ui/preview-card";
import type { IOperatorListItem } from "#/types/operators";
import { Link } from "@tanstack/react-router";
import { OperatorPreview } from "./OperatorPreview";
import { RARITY_COLORS } from "../constants";
import { formatSubProfession, rarityToNumber } from "#/lib/utils";
import { env } from "#/env";
import { CampIcon, ClassIcon } from "./Icons";

interface IOperatorCardCompactProps {
    operator: IOperatorListItem;
}

const NAME_SPLIT_REGEX = /( the )|\(/gi;

function parseOperatorName(name: string) {
    const parts = name.replace(/\)$/, "").split(NAME_SPLIT_REGEX);
    const displayName = parts[0] ?? name;
    const subtitle = parts[2] ?? null;
    const isLong = displayName.split(" ").length > 1 && displayName.length >= 16;
    return { displayName, subtitle, isLong };
}

export function OperatorCardCompact({ operator }: IOperatorCardCompactProps) {
    const rarityColor = RARITY_COLORS[rarityToNumber(operator.rarity)] ?? "#ffffff";
    const operatorId = operator.id ?? "";
    const operatorName = operator.name ?? "Unknown";
    const { displayName, subtitle, isLong } = parseOperatorName(operatorName);
    const factionLogoId = operator.nationId || operator.teamId || operator.groupId || "rhodes";

    return (
        <HoverCard>
            <HoverCardTrigger>
                <Link to={`/operators/${operator.id}`} className="group relative flex flex-col rounded bg-card pt-1 pr-2 pb-1 pl-1.5 transition-transform hover:scale-102">
                    <div className="ml-px flex h-4.25 flex-col justify-center text-left sm:h-5">
                        {subtitle && <span className="text-[0.4375rem] text-foreground leading-normal sm:text-[0.5625rem] sm:leading-loose">{subtitle}</span>}
                        <span
                            className="truncate text-foreground"
                            style={{
                                fontSize: isLong ? "9px" : "12px",
                                lineHeight: isLong ? "9px" : "17px",
                            }}
                        >
                            {displayName}
                        </span>
                    </div>
                    <div className="relative box-content aspect-square h-20 overflow-hidden sm:h-30" style={{ borderBottom: `4px solid ${rarityColor}` }}>
                        <div className="pointer-events-none absolute -translate-x-3 -translate-y-2">
                            <CampIcon groupId={factionLogoId} className="opacity-5 transition-opacity group-hover:opacity-10" size={140} />
                        </div>
                        <img alt={operatorName} className="relative h-full w-full object-contain" height={120} loading="lazy" src={`${env.VITE_BACKEND_URL}/api/avatar/${operatorId}`} width={120} />

                        <div className="absolute right-0 bottom-0 h-4 w-4 opacity-70 group-hover:opacity-100 sm:h-5 sm:w-5">
                            <ClassIcon profession={operator.profession} size={80} />
                        </div>
                    </div>
                    <div className="mt-0.5 truncate text-center text-[0.5625rem] text-muted-foreground leading-tight sm:text-xs">{formatSubProfession(operator.subProfessionId.toLowerCase())}</div>
                </Link>
            </HoverCardTrigger>
            <HoverCardContent className="w-max max-w-85 p-0">
                <OperatorPreview operator={operator} />
            </HoverCardContent>
        </HoverCard>
    );
}
