"use client";

import Image from "next/image";
import { memo, useCallback, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { capitalize, formatSubProfession, rarityToNumber } from "~/lib/utils";
import type { OperatorFromList } from "~/types/api/operators";
import { RARITY_COLORS } from "../../list/constants";
import { FactionLogo } from "../../list/ui/impl/faction-logo";

interface OperatorTooltipProps {
    operator: OperatorFromList;
    children: React.ReactNode;
    onHoverChange?: (isHovered: boolean) => void;
}

interface TooltipPosition {
    x: number;
    y: number;
}

const TOOLTIP_WIDTH = 320;
const TOOLTIP_HEIGHT = 100; // Approximate height
const TOOLTIP_OFFSET = 12;

export const OperatorTooltip = memo(function OperatorTooltip({ operator, children, onHoverChange }: OperatorTooltipProps) {
    const [isVisible, setIsVisible] = useState(false);
    const [position, setPosition] = useState<TooltipPosition>({ x: 0, y: 0 });
    const triggerRef = useRef<HTMLDivElement>(null);

    const rarityNum = rarityToNumber(operator.rarity);
    const rarityColor = RARITY_COLORS[rarityNum] ?? "#ffffff";

    const calculatePosition = useCallback(() => {
        if (!triggerRef.current) return;

        const rect = triggerRef.current.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        // Default: position above the element, centered horizontally
        let x = rect.left + rect.width / 2 - TOOLTIP_WIDTH / 2;
        let y = rect.top - TOOLTIP_HEIGHT - TOOLTIP_OFFSET;

        // Clamp horizontal position to viewport
        if (x < 8) {
            x = 8;
        } else if (x + TOOLTIP_WIDTH > viewportWidth - 8) {
            x = viewportWidth - TOOLTIP_WIDTH - 8;
        }

        // If tooltip would go above viewport, position below instead
        if (y < 8) {
            y = rect.bottom + TOOLTIP_OFFSET;
        }

        // If tooltip would go below viewport, clamp it
        if (y + TOOLTIP_HEIGHT > viewportHeight - 8) {
            y = viewportHeight - TOOLTIP_HEIGHT - 8;
        }

        setPosition({ x, y });
    }, []);

    const handleMouseEnter = useCallback(() => {
        calculatePosition();
        setIsVisible(true);
        onHoverChange?.(true);
    }, [calculatePosition, onHoverChange]);

    const handleMouseLeave = useCallback(() => {
        setIsVisible(false);
        onHoverChange?.(false);
    }, [onHoverChange]);

    const tooltipContent = isVisible && typeof window !== "undefined" && (
        <div
            className="fade-in-0 zoom-in-95 pointer-events-none fixed z-[9999] w-80 animate-in rounded-lg border border-border bg-popover p-3 shadow-xl duration-100"
            style={{
                left: position.x,
                top: position.y,
            }}
        >
            <div className="flex items-start gap-3">
                {/* Avatar */}
                <div className="relative h-16 w-16 shrink-0 overflow-hidden rounded-md border border-border/50">
                    <Image alt={`${operator.name} Avatar`} className="object-cover" fill sizes="64px" src={`/api/cdn${operator.portrait}`} />
                </div>

                <div className="min-w-0 flex-1 space-y-1">
                    {/* Name and faction */}
                    <div className="flex items-center justify-between gap-2">
                        <h4 className="truncate font-semibold text-base text-foreground">{operator.name}</h4>
                        <div className="h-5 w-5 shrink-0">
                            <FactionLogo className="object-contain" groupId={operator.groupId} nationId={operator.nationId} size={20} teamId={operator.teamId} />
                        </div>
                    </div>

                    {/* Rarity and class info */}
                    <p className="font-semibold text-sm" style={{ color: rarityColor }}>
                        {`${rarityNum}★ ${formatSubProfession(operator.subProfessionId.toLowerCase())}`}
                    </p>

                    {/* Position and race */}
                    <div className="flex flex-wrap gap-x-2 text-muted-foreground text-xs">
                        <span>{capitalize(operator.position?.toLowerCase() ?? "Unknown")}</span>
                        {operator.profile?.basicInfo?.race && (
                            <>
                                <span>•</span>
                                <span>{operator.profile.basicInfo.race}</span>
                            </>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );

    return (
        <>
            {/* biome-ignore lint/a11y/noStaticElementInteractions: Wrapper for tooltip hover events, child Link is the interactive element */}
            <div onMouseEnter={handleMouseEnter} onMouseLeave={handleMouseLeave} ref={triggerRef}>
                {children}
            </div>
            {tooltipContent && createPortal(tooltipContent, document.body)}
        </>
    );
});
