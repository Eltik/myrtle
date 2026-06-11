import { Link } from "@tanstack/react-router";
import { Badge } from "#/components/ui/badge";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { cn, formatNationId, formatProfession, formatSubProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { campLogo, operatorHero } from "../assets";
import { RARITY_COLORS, RARITY_GLOW } from "../constants";
import { useParallaxProgress } from "../useParallaxProgress";
import styles from "./OperatorHero.module.css";

interface IOperatorHeroProps {
    operator: IOperatorListItem;
}

export function OperatorHero({ operator }: IOperatorHeroProps) {
    const ref = useParallaxProgress<HTMLDivElement>();

    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[operator.rarity] ?? RARITY_COLORS.TIER_1;
    const rarityGlow = RARITY_GLOW[operator.rarity] ?? RARITY_GLOW.TIER_1;

    const img = operatorHero(operator.id ?? "", operator.skin, operator.portrait, operator.server);

    const factionId = operator.nationId?.length ? operator.nationId : operator.teamId?.length ? operator.teamId : operator.groupId?.length ? operator.groupId : null;

    return (
        <div className={cn("relative w-full overflow-hidden contain-layout", styles["parallax-hero"])} ref={ref}>
            <div className="md:hidden">
                <div className="relative h-90 w-full overflow-hidden sm:h-100">
                    <div aria-hidden className={cn("backface-hidden absolute inset-x-0 top-0 will-change-transform contain-paint", styles["parallax-image"])}>
                        <div className="flex items-start justify-center pt-0">
                            <div className="relative h-120 w-[85vw] max-w-95 sm:h-135 sm:w-110 sm:max-w-none">
                                <img alt={operator.name} className={cn("h-full w-full object-contain object-top", rarityGlow)} decoding="async" loading="eager" src={img} />
                            </div>
                        </div>
                    </div>
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/2 bg-linear-to-t from-background via-background/80 to-transparent" />
                    <div className={cn("backface-hidden relative z-10 flex h-full flex-col justify-end px-3 pb-4 will-change-[transform,opacity] sm:px-4 sm:pb-5", styles["parallax-content"])}>
                        <Breadcrumb className="mb-2">
                            <BreadcrumbList className="text-xs">
                                <BreadcrumbItem>
                                    <BreadcrumbLink render={<Link to="/operators" />}>Operators</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator />
                                <BreadcrumbItem>
                                    <BreadcrumbPage className="font-medium">{operator.name}</BreadcrumbPage>
                                </BreadcrumbItem>
                            </BreadcrumbList>
                        </Breadcrumb>

                        <div>
                            <h1 className="font-bold text-2xl text-foreground tracking-tight sm:text-3xl">{operator.name}</h1>
                            <div className="mt-1.5 flex items-center gap-2">
                                <span className={cn("font-semibold text-base tracking-wider", rarityColor)}>{"★".repeat(rarityNum)}</span>
                                <span className="text-muted-foreground/50">|</span>
                                <span className="text-muted-foreground text-sm">{formatSubProfession(operator.subProfessionId)}</span>
                            </div>
                        </div>

                        <div className="mt-3 flex flex-wrap gap-2">
                            <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                {formatProfession(operator.profession)}
                            </Badge>
                            <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                {operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : operator.position}
                            </Badge>
                            {operator.nationId && (
                                <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                    {formatNationId(operator.nationId)}
                                </Badge>
                            )}
                        </div>
                    </div>
                </div>
            </div>

            <div className="hidden md:block">
                <div className="lg:h105 relative h-95 w-full overflow-hidden">
                    <div aria-hidden className={cn("backface-hidden absolute inset-x-0 top-0 will-change-transform contain-paint", styles["parallax-image"])}>
                        <div className="flex items-start justify-end pr-[5%] lg:pr-[10%]">
                            <div className="relative h-155 w-130 lg:h-180 lg:w-150">
                                <img alt={operator.name} className={cn("h-full w-full object-contain object-top", rarityGlow)} decoding="async" loading="eager" src={img} />
                            </div>
                        </div>
                    </div>
                    <div className="pointer-events-none absolute inset-x-0 bottom-0 h-1/3 bg-linear-to-t from-background via-background/80 to-transparent" />
                    <div className={cn("backface-hidden relative z-10 mx-auto flex h-full max-w-6xl flex-col justify-end px-8 pb-6 will-change-[transform,opacity] lg:pb-8", styles["parallax-content"])}>
                        <Breadcrumb className="mb-3">
                            <BreadcrumbList>
                                <BreadcrumbItem>
                                    <BreadcrumbLink render={<Link to="/operators" />}>Operators</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator />
                                <BreadcrumbItem>
                                    <BreadcrumbPage className="font-medium">{operator.name}</BreadcrumbPage>
                                </BreadcrumbItem>
                            </BreadcrumbList>
                        </Breadcrumb>

                        <div className="flex flex-row items-end justify-between gap-4">
                            <div className="flex flex-col gap-2">
                                <div>
                                    <h1 className="font-bold text-4xl text-foreground tracking-tight lg:text-5xl">{operator.name}</h1>
                                    <div className="mt-1.5 flex items-center gap-3">
                                        <span className={cn("font-semibold text-lg tracking-wider", rarityColor)}>{"★".repeat(rarityNum)}</span>
                                        <span className="text-muted-foreground/50">|</span>
                                        <span className="text-base text-muted-foreground">{formatSubProfession(operator.subProfessionId)}</span>
                                    </div>
                                </div>

                                <div className="flex flex-wrap gap-2">
                                    <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                        {formatProfession(operator.profession)}
                                    </Badge>
                                    <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                        {operator.position === "RANGED" ? "Ranged" : operator.position === "MELEE" ? "Melee" : operator.position}
                                    </Badge>
                                    {operator.nationId && (
                                        <Badge variant="outline" className="border-transparent bg-accent text-foreground">
                                            {formatNationId(operator.nationId)}
                                        </Badge>
                                    )}
                                </div>
                            </div>

                            {factionId && (
                                <div className="flex items-center gap-4">
                                    <div className="h-14 w-14 lg:h-16 lg:w-16">
                                        <img alt={factionId} className="h-full w-full object-contain opacity-80" decoding="async" loading="lazy" src={campLogo(factionId, operator.server)} />
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
