import { useMutation } from "@tanstack/react-query";
import { ChevronDownIcon, ChevronUpIcon, EyeIcon, EyeOffIcon, GripVerticalIcon, RotateCcwIcon } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Kicker } from "#/components/ui/kicker";
import { toastManager } from "#/components/ui/toast";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { ProfileTab } from "#/types/generated/ProfileTab";
import type { IUserProfile } from "#/types/user";
import { defaultLayout, isDefaultLayout, moveEntry, normalizeLayout, tabsForSave, toggleTab } from "../layout";
import type { TabId } from "../types";
import type { messages } from "./ProfileLayoutEditor.messages";

const DRAG_MIME = "application/x-profile-tab";

interface IProfileLayoutEditorProps {
    /** The owner's own profile, whose layout seeds the editor. */
    profile: IUserProfile;
    labels: Record<TabId, string>;
    onClose: () => void;
}

/**
 * The owner's tab editor, shown in place of the tab bar. Order changes by drag or by
 * the arrow buttons (the keyboard and touch path), visibility by the eye toggle.
 * Nothing is stored until Save, which sends the tabs alone. After Reset, Save with the
 * canonical layout and no showcase or background stores "no layout", which is the same
 * page the profile showed before layouts existed.
 */
export function ProfileLayoutEditor({ profile, labels, onClose }: IProfileLayoutEditorProps) {
    const t: TypedT<typeof messages> = useT("user");
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    const [tabs, setTabs] = useState<ProfileTab[]>(() => normalizeLayout(profile.profile_layout?.tabs));
    // Whether the owner pressed Reset: the only way this editor clears the layout.
    const [reset, setReset] = useState(false);
    const [dragFrom, setDragFrom] = useState<number | null>(null);
    const [dragOver, setDragOver] = useState<number | null>(null);
    const [announcement, setAnnouncement] = useState("");

    const save = useMutation({
        mutationFn: (next: ProfileTab[]) =>
            // Only the layout: the privacy flags are left out, so the backend keeps
            // them as stored rather than overwriting them with this page's copy.
            updateUserSettingsFn({ data: { profile_layout: tabsForSave(next, reset, profile.profile_layout ?? null) } }),
        onSuccess: async () => {
            await invalidateProfile();
            toastManager.add({ id: `profile-layout-${Date.now()}`, title: t("profile.layout.saved.title"), description: t("profile.layout.saved.body"), type: "success" });
            onClose();
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `profile-layout-err-${Date.now()}`, title: t("profile.layout.saveFailed.title"), description: describeError(err), type: "error" });
        },
    });

    const move = (index: number, delta: number) => {
        const moved = moveEntry(tabs, index, delta);
        if (!moved) return;
        setTabs(moved.next);
        setAnnouncement(t("profile.layout.moved", { tab: labels[tabs[index].id], position: moved.to + 1, total: moved.next.length }));
    };

    const drop = (to: number) => {
        if (dragFrom !== null) move(dragFrom, to - dragFrom);
        setDragFrom(null);
        setDragOver(null);
    };

    const allHidden = tabs.every((tab) => !tab.visible);

    return (
        <section className="flex flex-col gap-4 rounded-2xl border border-border/50 bg-card/40 p-4 sm:p-5" aria-labelledby="profile-layout-kicker">
            <div>
                <Kicker id="profile-layout-kicker">{t("profile.layout.kicker")}</Kicker>
                <p className="max-w-prose text-muted-foreground text-sm">{t("profile.layout.help")}</p>
            </div>

            <ol className="flex flex-col gap-1.5" aria-label={t("profile.layout.list")}>
                {tabs.map((tab, i) => {
                    const label = labels[tab.id];
                    return (
                        <li
                            key={tab.id}
                            draggable
                            onDragStart={(e) => {
                                e.dataTransfer.effectAllowed = "move";
                                e.dataTransfer.setData(DRAG_MIME, tab.id);
                                setDragFrom(i);
                            }}
                            onDragOver={(e) => {
                                if (dragFrom === null || !e.dataTransfer.types.includes(DRAG_MIME)) return;
                                e.preventDefault();
                                e.dataTransfer.dropEffect = "move";
                                if (dragOver !== i) setDragOver(i);
                            }}
                            onDrop={(e) => {
                                e.preventDefault();
                                drop(i);
                            }}
                            onDragEnd={() => {
                                setDragFrom(null);
                                setDragOver(null);
                            }}
                            className={cn("flex items-center gap-2 rounded-xl border border-border/50 bg-background/60 py-1.5 pr-1.5 pl-2 transition-colors", dragFrom === i && "opacity-50", dragOver === i && dragFrom !== i && "border-primary/60 bg-primary/5")}
                        >
                            <GripVerticalIcon aria-hidden="true" className="size-4 shrink-0 cursor-grab text-muted-foreground active:cursor-grabbing" />
                            <span className={cn("min-w-0 flex-1 truncate font-medium text-sm", !tab.visible && "text-muted-foreground")}>{label}</span>
                            {!tab.visible && <span className="rounded-full bg-muted px-2 py-0.5 font-medium font-mono text-[11px] text-muted-foreground leading-none">{t("profile.layout.hiddenBadge")}</span>}
                            <Button type="button" size="icon-xs" variant="ghost" onClick={() => setTabs(toggleTab(tabs, tab.id))} aria-pressed={!tab.visible} aria-label={tab.visible ? t("profile.layout.hide", { tab: label }) : t("profile.layout.show", { tab: label })}>
                                {tab.visible ? <EyeIcon /> : <EyeOffIcon />}
                            </Button>
                            <Button type="button" size="icon-xs" variant="outline" onClick={() => move(i, -1)} disabled={i === 0} aria-label={t("profile.layout.moveUp", { tab: label })}>
                                <ChevronUpIcon />
                            </Button>
                            <Button type="button" size="icon-xs" variant="outline" onClick={() => move(i, 1)} disabled={i === tabs.length - 1} aria-label={t("profile.layout.moveDown", { tab: label })}>
                                <ChevronDownIcon />
                            </Button>
                        </li>
                    );
                })}
            </ol>
            <p className="sr-only" aria-live="polite">
                {announcement}
            </p>

            {allHidden && <p className="text-muted-foreground text-sm">{t("profile.layout.allHidden")}</p>}

            <div className="flex flex-wrap items-center gap-2">
                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                        setTabs(defaultLayout());
                        setReset(true);
                    }}
                    disabled={isDefaultLayout(tabs)}
                >
                    <RotateCcwIcon />
                    {t("profile.layout.reset")}
                </Button>
                <div className="ml-auto flex items-center gap-2">
                    <Button type="button" variant="outline" size="sm" onClick={onClose} disabled={save.isPending}>
                        {t("profile.layout.cancel")}
                    </Button>
                    <Button type="button" size="sm" onClick={() => save.mutate(tabs)} loading={save.isPending}>
                        {t("profile.layout.save")}
                    </Button>
                </div>
            </div>
        </section>
    );
}
