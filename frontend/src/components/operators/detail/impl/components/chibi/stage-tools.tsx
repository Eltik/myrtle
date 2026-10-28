import { FlipHorizontal2Icon, Grid3x3Icon, Maximize2Icon, Minimize2Icon, ScanIcon } from "lucide-react";
import type React from "react";
import { Button } from "#/components/ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { messages } from "./stage-tools.messages";

export type StageBackground = "checker" | "black" | "white" | "custom";

/** The checkerboard in theme colours, so "transparent" reads in light and dark alike. */
const CHECKER = "repeating-conic-gradient(var(--color-muted) 0% 25%, var(--color-card) 0% 50%) 0 0 / 16px 16px";

/** The CSS `background` behind the canvas for a swatch. */
export function stageBackgroundCss(background: StageBackground, custom: string): string {
    switch (background) {
        case "black":
            return "#000000";
        case "white":
            return "#ffffff";
        case "custom":
            return custom;
        default:
            return CHECKER;
    }
}

/** A swatch's fill as a PixiJS colour, or `null` for the see-through checkerboard. */
export function stageBackgroundColor(background: StageBackground, custom: string): number | null {
    switch (background) {
        case "black":
            return 0x000000;
        case "white":
            return 0xffffff;
        case "custom":
            return Number.parseInt(custom.replace("#", ""), 16);
        default:
            return null;
    }
}

const toolSurface = "bg-background/80 backdrop-blur-sm";

function ToolButton({ label, pressed, onClick, children }: { label: string; pressed?: boolean; onClick: () => void; children: React.ReactNode }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger render={<Button aria-label={label} aria-pressed={pressed} className={cn(toolSurface, "aria-pressed:bg-accent aria-pressed:text-accent-foreground")} onClick={onClick} size="icon-xs" variant="ghost" />}>{children}</TooltipTrigger>
            <TooltipPopup side="left">{label}</TooltipPopup>
        </Tooltip>
    );
}

interface IStageToolsProps {
    flipped: boolean;
    grid: boolean;
    fullscreen: boolean;
    zoom: number;
    onFlip: () => void;
    onGrid: () => void;
    onFit: () => void;
    onFullscreen: () => void;
}

/** The vertical tool stack in the stage's top-right corner, with the zoom readout under it. */
export function StageTools({ flipped, grid, fullscreen, zoom, onFlip, onGrid, onFit, onFullscreen }: IStageToolsProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    return (
        <div className="absolute top-2 right-2 flex flex-col items-center gap-1">
            <ToolButton label={t("chibiStage.flip")} onClick={onFlip} pressed={flipped}>
                <FlipHorizontal2Icon />
            </ToolButton>
            <ToolButton label={t("chibiStage.grid")} onClick={onGrid} pressed={grid}>
                <Grid3x3Icon />
            </ToolButton>
            <ToolButton label={t("chibiStage.fit")} onClick={onFit}>
                <ScanIcon />
            </ToolButton>
            <ToolButton label={fullscreen ? t("chibiStage.exitFullscreen") : t("chibiStage.fullscreen")} onClick={onFullscreen}>
                {fullscreen ? <Minimize2Icon /> : <Maximize2Icon />}
            </ToolButton>
            <span className={cn(toolSurface, "rounded px-1 font-mono text-[10px] text-muted-foreground tabular-nums")}>{t("chibiStage.zoom", { percent: Math.round(zoom * 100) })}</span>
        </div>
    );
}

/** The interaction help in the bottom-left corner. Touch screens get none of these gestures, and a narrow viewer leaves it no room beside the swatches. */
export function StageHint(): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    return <p className={cn(toolSurface, "pointer-events-none absolute bottom-2 left-2 m-0 @lg:pointer-fine:block hidden rounded px-1.5 py-0.5 text-[10px] text-muted-foreground")}>{t("chibiStage.hint")}</p>;
}

interface IBackgroundSwatchesProps {
    value: StageBackground;
    custom: string;
    onChange: (value: StageBackground) => void;
    onCustomChange: (color: string) => void;
}

const swatchClass = "block size-5 cursor-pointer rounded-full border border-border shadow-xs outline-none ring-offset-1 ring-offset-background focus-visible:ring-2 focus-visible:ring-ring aria-pressed:ring-2 aria-pressed:ring-primary data-[active=true]:ring-2 data-[active=true]:ring-primary";

/** Backdrop swatches in the stage's bottom-right corner. They change the view only. */
export function BackgroundSwatches({ value, custom, onChange, onCustomChange }: IBackgroundSwatchesProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    const swatches: { id: Exclude<StageBackground, "custom">; label: string }[] = [
        { id: "checker", label: t("chibiStage.bg.checker") },
        { id: "black", label: t("chibiStage.bg.black") },
        { id: "white", label: t("chibiStage.bg.white") },
    ];
    return (
        <fieldset aria-label={t("chibiStage.background")} className={cn(toolSurface, "absolute right-2 bottom-2 m-0 flex min-w-0 items-center gap-1.5 rounded-full border-0 px-1.5 py-1")}>
            {swatches.map((swatch) => (
                <Tooltip key={swatch.id}>
                    <TooltipTrigger render={<button aria-label={swatch.label} aria-pressed={value === swatch.id} className={swatchClass} onClick={() => onChange(swatch.id)} style={{ background: stageBackgroundCss(swatch.id, custom) }} type="button" />} />
                    <TooltipPopup>{swatch.label}</TooltipPopup>
                </Tooltip>
            ))}
            <Tooltip>
                <TooltipTrigger
                    render={
                        // The native picker, laid invisibly over the swatch, takes the click; picking selects it.
                        <span className={cn(swatchClass, "relative overflow-hidden focus-within:ring-2 focus-within:ring-ring")} data-active={value === "custom"} style={{ background: value === "custom" ? custom : "conic-gradient(#f43f5e, #f59e0b, #22c55e, #3b82f6, #a855f7, #f43f5e)" }} />
                    }
                >
                    <input
                        aria-label={t("chibiStage.bg.custom")}
                        className="absolute inset-0 cursor-pointer opacity-0"
                        onChange={(event) => {
                            onCustomChange(event.currentTarget.value);
                            onChange("custom");
                        }}
                        onClick={() => onChange("custom")}
                        type="color"
                        value={custom}
                    />
                </TooltipTrigger>
                <TooltipPopup>{t("chibiStage.bg.custom")}</TooltipPopup>
            </Tooltip>
        </fieldset>
    );
}
