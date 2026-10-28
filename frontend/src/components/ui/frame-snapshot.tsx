import { ImageDownIcon } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { Menu, MenuGroup, MenuGroupLabel, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./frame-snapshot.messages";

/** 1x is the art's own size (one skeleton unit, one pixel); 2x for a larger copy. */
export const FRAME_SNAPSHOT_SCALES = [1, 2] as const;

interface IFrameSnapshotProps {
    /** The PNG's pixel size at `scale`, read when the menu opens; `null` hides that option. */
    sizeOf: (scale: number) => { width: number; height: number } | null;
    onSave: (scale: number) => void;
    /** Called as the menu opens; a viewer pauses here so the frame picked from it is the one on screen. */
    onOpen?: () => void;
    disabled?: boolean;
    className?: string;
    /** Trigger content for a labelled outline button; without it the trigger is an icon button. */
    children?: React.ReactNode;
}

/**
 * Save the frame on screen as a transparent PNG, sized from a menu. Sits
 * beside {@link FrameStepper}; the viewer does the drawing through
 * `#/lib/frame-png`.
 */
export function FrameSnapshot({ sizeOf, onSave, onOpen, disabled, className, children }: IFrameSnapshotProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("common");
    const [open, setOpen] = useState(false);
    const label = t("frameSnapshot.save");
    // Read on open, not on render: the size follows the clip and the frame.
    const options = open ? FRAME_SNAPSHOT_SCALES.map((scale) => ({ scale, size: sizeOf(scale) })) : [];
    // A popup portalled to <body> is invisible while the viewer is fullscreen.
    const fullscreen = typeof document !== "undefined" ? (document.fullscreenElement as HTMLElement | null) : null;
    const trigger = children ? <Button aria-label={label} className={className} disabled={disabled} size="xs" variant="outline" /> : <Button aria-label={label} className={className} disabled={disabled} size="icon-xs" variant="ghost" />;

    return (
        <Menu
            onOpenChange={(next) => {
                if (next) onOpen?.();
                setOpen(next);
            }}
            open={open}
        >
            <Tooltip>
                <TooltipTrigger render={<MenuTrigger render={trigger} />}>{children ?? <ImageDownIcon />}</TooltipTrigger>
                <TooltipPopup>{label}</TooltipPopup>
            </Tooltip>
            <MenuPopup align="end" portalProps={{ container: fullscreen ?? undefined }}>
                <MenuGroup>
                    <MenuGroupLabel>{t("frameSnapshot.heading")}</MenuGroupLabel>
                    {options.map(({ scale, size }) =>
                        size ? (
                            <MenuItem className="tabular-nums" key={scale} onClick={() => onSave(scale)}>
                                {t("frameSnapshot.option", { scale, width: size.width, height: size.height })}
                            </MenuItem>
                        ) : null,
                    )}
                </MenuGroup>
            </MenuPopup>
        </Menu>
    );
}
