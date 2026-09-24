/**
 * WHERE THE TEXT BOX SITS: a 16:9 miniature of the stage, and the five presets
 * beside it.
 *
 * The miniature is not decorative. Its thumbnail sits where the real box sits,
 * over the same travel: `(1 + x) / 2` of the room left over across, and `y` of
 * the lift's own 40% up. Pressing anywhere on the pad puts the box there, so
 * one press is a coarse move and a drag is a fine one, and the arrow keys nudge
 * the THUMB, which is the control that takes focus.
 */
import { RotateCcwIcon } from "lucide-react";
import type React from "react";
import { useRef } from "react";
import { Button } from "#/components/ui/button";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { BOX_NUDGE_X, BOX_NUDGE_Y, BOX_PRESET_POSITIONS, BOX_PRESETS, type BoxPosition, type BoxPreset, clampBoxPosition, presetForPosition } from "#/lib/story/settings";
import type { messages } from "./reader.messages";
import { TOUCH_BUTTON } from "./SettingsRow";

const PAD_THUMB_W = 0.55;
const PAD_THUMB_H = 0.16;
const PAD_MARGIN_B = 0.03;
const PAD_TRAVEL_Y = 0.4;

function PositionPad({ value, label, onChange }: { value: BoxPosition; label: string; onChange: (next: BoxPosition) => void }): React.ReactElement {
    const padRef = useRef<HTMLDivElement>(null);
    const dragging = useRef(false);

    const place = (clientX: number, clientY: number) => {
        const rect = padRef.current?.getBoundingClientRect();
        if (!rect || !(rect.width > 0) || !(rect.height > 0)) return;
        const freeX = rect.width * (1 - PAD_THUMB_W);
        const left = clientX - rect.left - (rect.width * PAD_THUMB_W) / 2;
        const x = freeX > 0 ? (left / freeX) * 2 - 1 : 0;
        const travelY = rect.height * PAD_TRAVEL_Y;
        const fromBottom = rect.bottom - clientY - (rect.height * PAD_THUMB_H) / 2 - rect.height * PAD_MARGIN_B;
        const y = travelY > 0 ? fromBottom / travelY : 0;
        onChange(clampBoxPosition({ x, y }));
    };

    return (
        <div
            ref={padRef}
            data-story-position-pad
            className="relative aspect-video w-full max-w-sm touch-none rounded-md border bg-linear-to-b from-muted to-background"
            onPointerDown={(e) => {
                dragging.current = true;
                e.currentTarget.setPointerCapture(e.pointerId);
                place(e.clientX, e.clientY);
            }}
            onPointerMove={(e) => {
                if (dragging.current) place(e.clientX, e.clientY);
            }}
            onPointerUp={() => {
                dragging.current = false;
            }}
            onPointerCancel={() => {
                dragging.current = false;
            }}
        >
            {/* The THUMB is the control, not the pad: it is what takes focus,
                what the arrow keys nudge and what carries the accessible name.
                The pad around it only reads the pointer. */}
            <button
                type="button"
                data-story-position-thumb
                aria-label={label}
                onKeyDown={(e) => {
                    if (e.key === "ArrowUp") onChange(clampBoxPosition({ x: value.x, y: value.y + BOX_NUDGE_Y }));
                    else if (e.key === "ArrowDown") onChange(clampBoxPosition({ x: value.x, y: value.y - BOX_NUDGE_Y }));
                    else if (e.key === "ArrowLeft") onChange(clampBoxPosition({ x: value.x - BOX_NUDGE_X, y: value.y }));
                    else if (e.key === "ArrowRight") onChange(clampBoxPosition({ x: value.x + BOX_NUDGE_X, y: value.y }));
                    else return;
                    e.preventDefault();
                }}
                className="absolute cursor-grab rounded-sm bg-primary/70 ring-1 ring-primary focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background active:cursor-grabbing"
                style={{
                    width: `${PAD_THUMB_W * 100}%`,
                    height: `${PAD_THUMB_H * 100}%`,
                    insetInlineStart: `calc(${(1 + value.x) / 2} * ${(1 - PAD_THUMB_W) * 100}%)`,
                    bottom: `${(PAD_MARGIN_B + value.y * PAD_TRAVEL_Y) * 100}%`,
                }}
            />
        </div>
    );
}

/** The pad and the presets together, which is the whole of the box-position row's control. */
export function BoxPositionControl({ position, onChange }: { position: BoxPosition; onChange: (next: BoxPosition) => void }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("story");
    const active = presetForPosition(position);
    return (
        <div className="flex w-full flex-col gap-2">
            <PositionPad value={position} label={t("settings.boxPosition.pad")} onChange={onChange} />
            <div className="flex flex-wrap gap-1.5">
                {BOX_PRESETS.map((name: BoxPreset) => (
                    <Button key={name} variant={active === name ? "secondary" : "outline"} size="sm" className={TOUCH_BUTTON} aria-pressed={active === name} data-story-box-preset={name} onClick={() => onChange(BOX_PRESET_POSITIONS[name])}>
                        {t(`settings.boxPreset.${name}`)}
                    </Button>
                ))}
                <Button variant="ghost" size="sm" className={TOUCH_BUTTON} onClick={() => onChange({ x: 0, y: 0 })}>
                    <RotateCcwIcon /> {t("settings.boxPreset.reset")}
                </Button>
            </div>
        </div>
    );
}
