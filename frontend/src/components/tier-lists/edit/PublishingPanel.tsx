import { CheckIcon, ChevronDownIcon, EyeIcon, EyeOffIcon, RocketIcon, TagIcon } from "lucide-react";
import { useId } from "react";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { Switch } from "#/components/ui/switch";
import type { ITierListFlair } from "#/lib/api/tier-lists";
import { cn } from "#/lib/utils";

const SECTION_LABEL = "inline-flex items-center gap-1.5 font-mono text-[10.5px] font-medium uppercase tracking-[0.12em] text-muted-foreground";

interface IPublishingPanelProps {
    flair: { id: number; label: string; color: string | null } | null;
    flairOptions: ITierListFlair[];
    isListed: boolean;
    canPublish: boolean;
    publishingDisabledReason: string | null;
    onSetFlair: (flairId: number | null) => void;
    onSetVisibility: (next: boolean) => void;
    onOpenPublishDialog: () => void;
    settingFlair: boolean;
    settingVisibility: boolean;
}

export function PublishingPanel({ flair, flairOptions, isListed, canPublish, publishingDisabledReason, onSetFlair, onSetVisibility, onOpenPublishDialog, settingFlair, settingVisibility }: IPublishingPanelProps) {
    const visibilityId = useId();
    const visibilityHintId = `${visibilityId}-hint`;

    return (
        <div className="rounded-xl border border-border bg-card p-3 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
            <div className="flex items-center justify-between gap-2">
                <span className="font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Publishing</span>
            </div>

            <div className="mt-3 flex flex-col gap-3">
                <div className="flex flex-col gap-1.5">
                    <label htmlFor={`${visibilityId}-flair-trigger`} className={SECTION_LABEL}>
                        <TagIcon className="h-3 w-3" aria-hidden="true" />
                        Flair
                    </label>
                    <Menu>
                        <MenuTrigger
                            id={`${visibilityId}-flair-trigger`}
                            disabled={settingFlair}
                            className="inline-flex h-9 w-full cursor-pointer items-center justify-between gap-2 rounded-lg border border-border bg-popover px-2.5 font-medium font-sans text-foreground text-xs leading-none transition-colors hover:bg-accent disabled:cursor-not-allowed disabled:opacity-60"
                        >
                            <span className="flex min-w-0 items-center gap-1.5">
                                {flair ? (
                                    <>
                                        <span className="h-1.5 w-1.5 shrink-0 rounded-full" style={{ background: flair.color ?? "var(--primary)" }} aria-hidden="true" />
                                        <span className="truncate">{flair.label}</span>
                                    </>
                                ) : (
                                    <span className="truncate text-muted-foreground">No flair</span>
                                )}
                            </span>
                            <ChevronDownIcon className="h-3 w-3 shrink-0 opacity-70" aria-hidden="true" />
                        </MenuTrigger>
                        <MenuPopup align="start" sideOffset={6} className="min-w-56">
                            <MenuItem onClick={() => onSetFlair(null)} className={cn(!flair && "bg-accent/60 text-accent-foreground")}>
                                <span className="flex items-center gap-2">
                                    <span className="h-1.5 w-1.5 rounded-full border border-border" aria-hidden="true" />
                                    <span>No flair</span>
                                </span>
                                {!flair && <CheckIcon className="ml-auto h-3.5 w-3.5" aria-hidden="true" />}
                            </MenuItem>
                            {flairOptions.map((opt) => {
                                const selected = flair?.id === opt.id;
                                return (
                                    <MenuItem key={opt.id} onClick={() => onSetFlair(opt.id)} className={cn(selected && "bg-accent/60 text-accent-foreground")}>
                                        <span className="flex items-center gap-2">
                                            <span className="h-1.5 w-1.5 rounded-full" style={{ background: opt.color ?? "var(--primary)" }} aria-hidden="true" />
                                            <span>{opt.label}</span>
                                        </span>
                                        {selected && <CheckIcon className="ml-auto h-3.5 w-3.5" aria-hidden="true" />}
                                    </MenuItem>
                                );
                            })}
                        </MenuPopup>
                    </Menu>
                    <p className="m-0 font-sans text-[11px] text-muted-foreground leading-snug">Tags your list with a topic shown on the browse page.</p>
                </div>

                <div className="flex items-start justify-between gap-3 rounded-lg border border-border bg-muted/30 px-2.5 py-2">
                    <div className="flex min-w-0 flex-col gap-0.5">
                        <label htmlFor={visibilityId} className={SECTION_LABEL}>
                            {isListed ? <EyeIcon className="h-3 w-3" aria-hidden="true" /> : <EyeOffIcon className="h-3 w-3" aria-hidden="true" />}
                            Visibility
                        </label>
                        <span className="font-medium font-sans text-[12.5px] text-foreground leading-tight">{isListed ? "Listed publicly" : "Hidden from browse"}</span>
                        <p id={visibilityHintId} className="m-0 font-sans text-[11px] text-muted-foreground leading-snug">
                            {isListed ? "Anyone can find this list from /tier-lists." : "Only people with the direct link can open it."}
                        </p>
                    </div>
                    <Switch id={visibilityId} checked={isListed} disabled={settingVisibility} aria-describedby={visibilityHintId} onCheckedChange={onSetVisibility} className="mt-1 shrink-0" />
                </div>

                <button
                    type="button"
                    onClick={onOpenPublishDialog}
                    disabled={!canPublish}
                    title={publishingDisabledReason ?? undefined}
                    aria-disabled={!canPublish || undefined}
                    className="inline-flex h-9 w-full cursor-pointer items-center justify-center gap-1.5 rounded-lg border border-primary bg-primary px-3 font-sans font-semibold text-primary-foreground text-xs leading-none shadow-[0_1px_2px_oklch(0_0_0/0.06)] transition-shadow hover:shadow-[0_2px_8px_color-mix(in_srgb,var(--primary)_30%,transparent)] disabled:cursor-not-allowed disabled:border-border disabled:bg-muted disabled:text-muted-foreground disabled:shadow-none"
                >
                    <RocketIcon className="h-3.5 w-3.5" aria-hidden="true" />
                    Publish version
                </button>
                {publishingDisabledReason && (
                    <p className="m-0 font-sans text-[11px] text-muted-foreground leading-snug" role="note">
                        {publishingDisabledReason}
                    </p>
                )}
            </div>
        </div>
    );
}
