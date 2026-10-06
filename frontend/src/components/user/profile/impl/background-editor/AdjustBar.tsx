import { RotateCcwIcon, ScanIcon, Trash2Icon } from "lucide-react";
import { Button } from "#/components/ui/button";
import { Slider, SliderPrimitive } from "#/components/ui/slider";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import { resetBackground, SCALE_MAX, SCALE_MIN, sameBackground, shownElite, withElite, withScale, zoomOf } from "../background";
import type { messages } from "./BackgroundEditor.messages";
import { SEGMENT_ITEM } from "./chips";
import { useHasElite2 } from "./useHasElite2";

interface IAdjustBarProps {
    draft: ProfileBackground | null;
    onChange: (next: ProfileBackground) => void;
    onRemove: () => void;
    /** The dead axis the author just tried to move along, for the moment it is shown. */
    deadAxis: "x" | "y" | null;
    disabled: boolean;
    /** The collapsed strip's version: the zoom alone, stacked narrow. */
    compact?: boolean;
}

/** A single-thumb slider's value, which base-ui may report as a one-entry array. */
function sliderValue(value: number | readonly number[]): number {
    return Array.isArray(value) ? (value[0] ?? SCALE_MIN) : (value as number);
}

/**
 * The row under the preview: the zoom (slider and value), Fit (zoom back to 100, the
 * position kept), Reset (the kind's starting position, no zoom) and Remove background.
 * The crop itself has no slider: the preview is dragged. Its right end says how, or for a
 * moment why a drag along a dead axis moved nothing.
 */
export function AdjustBar({ draft, onChange, onRemove, deadAxis, disabled, compact = false }: IAdjustBarProps) {
    const t: TypedT<typeof messages> = useT("user");
    const hasElite2 = useHasElite2(draft?.kind === "operator" ? draft.id : null);
    if (!draft) return <p className="m-0 py-1.5 text-center font-sans text-muted-foreground text-sm">{t("profile.background.none")}</p>;
    // Until the catalogue answers, the operator reads as having elite 2, the art the header draws by default.
    const elite2 = hasElite2 ?? true;
    const zoom = zoomOf(draft);
    const hint = deadAxis === "x" ? t("profile.background.deadX") : deadAxis === "y" ? t("profile.background.deadY") : t("profile.background.panHint");
    const slider = (
        <Slider
            className={cn("flex items-center gap-3 data-[orientation=horizontal]:w-80 max-sm:data-[orientation=horizontal]:w-full [&_[data-slot=slider-control]]:min-w-0 [&_[data-slot=slider-control]]:flex-1", compact && "data-[orientation=horizontal]:w-full")}
            min={SCALE_MIN}
            max={SCALE_MAX}
            step={1}
            value={zoom}
            disabled={disabled}
            onValueChange={(next) => onChange(withScale(draft, sliderValue(next)))}
        >
            <SliderPrimitive.Label className="shrink-0 font-medium font-sans text-xs">{t("profile.background.zoom")}</SliderPrimitive.Label>
            <span className="w-10 shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{t("profile.background.zoomValue", { scale: zoom })}</span>
        </Slider>
    );
    if (compact) return slider;
    return (
        <div className="flex flex-wrap items-center justify-center gap-x-5 gap-y-2">
            {slider}
            {draft.kind === "operator" && (
                <ToggleGroup value={[String(shownElite(draft, elite2))]} onValueChange={(next) => next[0] && onChange(withElite(draft, next[0] === "1" ? 1 : 2, elite2))} variant="outline" size="sm" aria-label={t("profile.background.elite")}>
                    <ToggleGroupItem className={SEGMENT_ITEM} value="1" disabled={disabled}>
                        {t("profile.background.elite.e1")}
                    </ToggleGroupItem>
                    <ToggleGroupItem className={SEGMENT_ITEM} value="2" disabled={disabled || hasElite2 === false} title={hasElite2 === false ? t("profile.background.elite.noE2") : undefined}>
                        {t("profile.background.elite.e2")}
                    </ToggleGroupItem>
                </ToggleGroup>
            )}
            <div className="flex items-center gap-1">
                <Button type="button" variant="ghost" size="sm" onClick={() => onChange(withScale(draft, SCALE_MIN))} disabled={disabled || zoom === SCALE_MIN}>
                    <ScanIcon />
                    {t("profile.background.fit")}
                </Button>
                <Button type="button" variant="ghost" size="sm" onClick={() => onChange(resetBackground(draft))} disabled={disabled || sameBackground(draft, resetBackground(draft))}>
                    <RotateCcwIcon />
                    {t("profile.background.reset")}
                </Button>
                <Button type="button" variant="ghost" size="sm" onClick={onRemove} disabled={disabled} className="text-muted-foreground hover:text-destructive-foreground">
                    <Trash2Icon />
                    {t("profile.background.remove")}
                </Button>
            </div>
            <p className={cn("m-0 min-w-0 font-sans text-xs transition-colors", deadAxis ? "text-foreground" : "text-muted-foreground max-md:hidden")} aria-live="polite">
                {hint}
            </p>
        </div>
    );
}
