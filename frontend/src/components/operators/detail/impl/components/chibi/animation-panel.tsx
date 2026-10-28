import type React from "react";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { AnimationGroupId, IAnimationGroup, IClipInfo } from "./animation-list";
import type { messages } from "./animation-panel.messages";

interface IAnimationPanelProps {
    groups: readonly IAnimationGroup[];
    selected: string;
    disabled?: boolean;
    onSelect: (name: string) => void;
    className?: string;
}

/** The clip list beside the stage, grouped by kind, each row with its frames and events. */
export function AnimationPanel({ groups, selected, disabled, onSelect, className }: IAnimationPanelProps): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    const count = groups.reduce((sum, group) => sum + group.clips.length, 0);

    return (
        <section className={cn("flex min-h-0 flex-col overflow-hidden rounded-md border border-border bg-background/40", className)}>
            <header className="flex items-center gap-2 border-border border-b px-2.5 py-1.5">
                <h5 className="m-0 font-medium text-foreground text-xs">{t("chibiClips.title")}</h5>
                <span className="rounded-full bg-muted px-1.5 font-mono text-[10px] text-muted-foreground tabular-nums">{t("chibiClips.count", { count })}</span>
            </header>
            <div className="min-h-0 flex-1 overflow-y-auto p-1">
                {groups.map((group) => (
                    <div className="mb-1 last:mb-0" key={group.id}>
                        <div className="px-1.5 pt-1.5 pb-0.5 font-mono text-[10px] text-muted-foreground uppercase tracking-wider">{groupLabel(t, group.id)}</div>
                        <ul className="m-0 list-none p-0">
                            {group.clips.map((clip) => (
                                <ClipRow clip={clip} disabled={disabled} key={clip.name} onSelect={onSelect} selected={clip.name === selected} />
                            ))}
                        </ul>
                    </div>
                ))}
            </div>
        </section>
    );
}

/** Literal keys, one per group, so the extractor sees each one used. */
function groupLabel(t: TypedT<typeof messages>, id: AnimationGroupId): string {
    switch (id) {
        case "idle":
            return t("chibiClips.group.idle");
        case "move":
            return t("chibiClips.group.move");
        case "attack":
            return t("chibiClips.group.attack");
        case "skill":
            return t("chibiClips.group.skill");
        default:
            return t("chibiClips.group.other");
    }
}

function ClipRow({ clip, selected, disabled, onSelect }: { clip: IClipInfo; selected: boolean; disabled?: boolean; onSelect: (name: string) => void }): React.ReactElement {
    const t: TypedT<typeof messages> = useT("operators");
    return (
        <li className="relative">
            <button
                aria-current={selected ? "true" : undefined}
                className="flex w-full cursor-pointer items-center gap-2 rounded px-1.5 py-1 text-left text-muted-foreground text-xs outline-none hover:bg-accent/60 hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-default disabled:opacity-64 aria-[current=true]:bg-accent aria-[current=true]:text-foreground"
                disabled={disabled}
                onClick={() => onSelect(clip.name)}
                type="button"
            >
                <span className="min-w-0 flex-1 truncate">{clip.name}</span>
                {/* Room for the event marker, laid over the row (px-1.5 + w-10 + gap-2 from the right) so its tooltip is not inside a button. */}
                {clip.events.length > 0 && <span aria-hidden="true" className="w-6 shrink-0" />}
                <span className="w-10 shrink-0 text-right font-mono text-[10px] tabular-nums">{t("chibiClips.frames", { count: clip.frames })}</span>
            </button>
            {clip.events.length > 0 && (
                <Tooltip>
                    <TooltipTrigger render={<span aria-label={t("chibiClips.events", { count: clip.events.length })} className="absolute top-1/2 right-[54px] w-6 -translate-y-1/2 cursor-default text-center font-mono text-[10px] text-primary tabular-nums" role="img" />}>◆{clip.events.length}</TooltipTrigger>
                    <TooltipPopup side="left">
                        <ul className="m-0 list-none p-0">
                            {clip.events.map((event) => (
                                <li className="font-mono tabular-nums" key={`${event.name}-${event.time}`}>
                                    {t("chibiClips.event", { name: event.name, seconds: event.time.toFixed(2) })}
                                </li>
                            ))}
                        </ul>
                    </TooltipPopup>
                </Tooltip>
            )}
        </li>
    );
}
