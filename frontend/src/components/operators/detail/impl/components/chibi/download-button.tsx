import { Settings2, X } from "lucide-react";
import type React from "react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { FrameSnapshot } from "#/components/ui/frame-snapshot";
import { Popover, PopoverContent, PopoverTrigger } from "#/components/ui/popover";
import { Progress } from "#/components/ui/progress";
import { Separator } from "#/components/ui/separator";
import { ToggleGroup, ToggleGroupItem } from "#/components/ui/toggle-group";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { EXPORT_FPS_OPTIONS, EXPORT_LOOP_OPTIONS } from "./constants";
import type { messages } from "./download-button.messages";
import { effectiveGifFps } from "./helpers";
import type { ExportFormat } from "./recorder";

export interface IRecordOptions {
    /** 1 for the standard size, 2 for double. */
    scale: number;
    fps: number;
    loopCount: number;
}

interface IExportGroupProps {
    disabled: boolean;
    isRecording: boolean;
    progress: number;
    onCancel: () => void;
    onRecord: (format: ExportFormat, options: IRecordOptions) => void;
    /** The PNG menu's sizes and save, as {@link FrameSnapshot} takes them. */
    pngSizeOf: (scale: number) => { width: number; height: number } | null;
    onSavePng: (scale: number) => void;
    onOpenPng: () => void;
}

const DEFAULT_FPS = 60;
const DEFAULT_LOOPS = 3;

function RecordButton({ label, hint, disabled, onClick }: { label: string; hint: string; disabled: boolean; onClick: () => void }): React.ReactElement {
    return (
        <Tooltip>
            <TooltipTrigger render={<Button disabled={disabled} onClick={onClick} size="xs" variant="outline" />}>{label}</TooltipTrigger>
            <TooltipPopup>{hint}</TooltipPopup>
        </Tooltip>
    );
}

/**
 * The viewer's save row: the frame on screen as PNG, the clip as WebM or GIF (standard or
 * double size), and a settings popover for frame rate, video loops and the MP4 fallback.
 * While a recording runs, its progress and a cancel button take the row's place.
 */
export function ExportGroup({ disabled, isRecording, progress, onCancel, onRecord, pngSizeOf, onSavePng, onOpenPng }: IExportGroupProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    const [fps, setFps] = useState(DEFAULT_FPS);
    const [loopCount, setLoopCount] = useState(DEFAULT_LOOPS);
    const [open, setOpen] = useState(false);
    const gifFps = Math.round(effectiveGifFps(fps) * 10) / 10;
    const record = (format: ExportFormat, scale = 1) => onRecord(format, { scale, fps, loopCount });
    // A popup portalled to <body> is invisible while the viewer is fullscreen.
    const fullscreen = typeof document !== "undefined" ? (document.fullscreenElement as HTMLElement | null) : null;

    if (isRecording) {
        return (
            <div className="flex items-center gap-2">
                <span className="text-muted-foreground text-xs">{t("chibiExport.label")}</span>
                <div className="flex min-w-30 flex-1 items-center gap-2">
                    <Progress className="h-2 w-full" value={progress} />
                    <span className="text-muted-foreground text-xs tabular-nums">{Math.round(progress)}%</span>
                </div>
                <Button aria-label={t("chibiExport.cancel")} onClick={onCancel} size="icon-xs" variant="ghost">
                    <X />
                </Button>
            </div>
        );
    }

    return (
        <div className="flex flex-wrap items-center gap-1">
            <span className="mr-1 text-muted-foreground text-xs">{t("chibiExport.label")}</span>
            <FrameSnapshot disabled={disabled} onOpen={onOpenPng} onSave={onSavePng} sizeOf={pngSizeOf}>
                {t("chibiExport.png")}
            </FrameSnapshot>
            <RecordButton disabled={disabled} hint={t("chibiExport.webmHint")} label={t("chibiExport.webm")} onClick={() => record("webm")} />
            <RecordButton disabled={disabled} hint={t("chibiExport.gifHint")} label={t("chibiExport.gif")} onClick={() => record("gif")} />
            <RecordButton disabled={disabled} hint={t("chibiExport.gif2xHint")} label={t("chibiExport.gif2x")} onClick={() => record("gif", 2)} />
            <Popover onOpenChange={setOpen} open={open}>
                <PopoverTrigger
                    render={
                        <Button aria-label={t("chibiExport.settings")} disabled={disabled} size="icon-xs" variant="ghost">
                            <Settings2 />
                        </Button>
                    }
                />
                <PopoverContent align="end" className="w-72" portalProps={{ container: fullscreen ?? undefined }}>
                    <div className="grid gap-3">
                        <h4 className="m-0 font-medium text-sm">{t("chibiExport.settings")}</h4>
                        <div className="grid gap-1.5">
                            <span className="text-xs">{t("chibiExport.frameRate")}</span>
                            <ToggleGroup aria-label={t("chibiExport.frameRate")} className="flex-wrap" onValueChange={(next) => next[0] && setFps(Number(next[0]))} size="sm" value={[String(fps)]} variant="outline">
                                {EXPORT_FPS_OPTIONS.map((option) => (
                                    <ToggleGroupItem className="text-xs" key={option} value={String(option)}>
                                        {t("chibiExport.fps", { fps: option })}
                                    </ToggleGroupItem>
                                ))}
                            </ToggleGroup>
                            {gifFps !== fps && <p className="m-0 text-[11px] text-muted-foreground leading-snug">{t("chibiExport.gifFpsNote", { gifFps, fps })}</p>}
                        </div>
                        <div className="grid gap-1.5">
                            <span className="text-xs">{t("chibiExport.loopCount")}</span>
                            <ToggleGroup aria-label={t("chibiExport.loopCount")} onValueChange={(next) => next[0] && setLoopCount(Number(next[0]))} size="sm" value={[String(loopCount)]} variant="outline">
                                {EXPORT_LOOP_OPTIONS.map((option) => (
                                    <ToggleGroupItem className="text-xs" key={option} value={String(option)}>
                                        {t("chibiExport.loops", { count: option })}
                                    </ToggleGroupItem>
                                ))}
                            </ToggleGroup>
                        </div>
                        <p className="m-0 text-[11px] text-muted-foreground leading-snug">{t("chibiExport.backgroundNote")}</p>
                        <Separator />
                        <Button
                            className="text-xs"
                            onClick={() => {
                                setOpen(false);
                                record("mp4");
                            }}
                            size="sm"
                            variant="outline"
                        >
                            {t("chibiExport.downloadMp4")}
                        </Button>
                    </div>
                </PopoverContent>
            </Popover>
        </div>
    );
}
