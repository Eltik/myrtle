import { useMutation } from "@tanstack/react-query";
import { ChevronDownIcon, ChevronUpIcon, EyeIcon, EyeOffIcon, PlusIcon, RotateCcwIcon, XIcon } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Kicker } from "#/components/ui/kicker";
import { toastManager } from "#/components/ui/toast";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import { addCard, HEADER_FIELDS, type HeaderField, headerDraft, headerForSave, type IHeaderDraft, isDefaultHeader, removeCard, STRIP_CARDS, STRIP_MAX, toggleHidden } from "../header";
import { moveEntry } from "../layout";
import type { messages } from "./ProfileHeaderEditor.messages";
import type { messages as stripMessages } from "./StatStrip.messages";

interface IProfileHeaderEditorProps {
    /** The owner's own profile, whose layout seeds the editor. */
    profile: IUserProfile;
    /** Receives every draft, so the page can preview it in the header and strip above. */
    onDraft?: (draft: IHeaderDraft) => void;
    onClose: () => void;
}

/** One row of either list: the same chrome as the tab editor's rows. */
const ROW = "flex items-center gap-2 rounded-xl border border-border/50 bg-background/60 py-1.5 pr-1.5 pl-3";
const BADGE = "rounded-full bg-muted px-2 py-0.5 font-medium font-mono text-[11px] text-muted-foreground leading-none";
const HEADING = "font-medium font-mono text-[11px] text-muted-foreground uppercase tracking-widest";

/**
 * The owner's header editor: which numbers fill the stat strip, in what order, and which
 * header numbers visitors never receive. Opened from the header, rendered under the stat
 * strip, which previews the draft (`onDraft`) along with the header's markers. Nothing is stored until Save, which sends the `header` key alone,
 * so the tabs, showcase and background are kept; a draft back at the default sends `null`,
 * which stores what a never-customized profile has.
 */
export function ProfileHeaderEditor({ profile, onDraft, onClose }: IProfileHeaderEditorProps) {
    const t: TypedT<typeof messages & typeof stripMessages> = useT("user");
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    const [draft, setDraft] = useState<IHeaderDraft>(() => headerDraft(profile.profile_layout ?? null));
    const [announcement, setAnnouncement] = useState("");
    useEffect(() => onDraft?.(draft), [draft, onDraft]);

    // The currency names are the game's own and stay as game vocabulary, as on the cards.
    const labels = useMemo<Record<HeaderField, string>>(
        () => ({
            level: t("profile.headerEditor.field.level"),
            registered: t("profile.headerEditor.field.registered"),
            sign_in: t("profile.headerEditor.field.signIn"),
            operators: t("profile.strip.operators.kicker"),
            skins: t("profile.strip.skins.kicker"),
            items: t("profile.strip.items.kicker"),
            orundum: "Orundum",
            originite: "Originite Prime",
            lmd: "LMD",
            headhunting_permits: "Headhunting Permits",
            ten_pull_permits: "Ten-roll Permits",
        }),
        [t],
    );

    const save = useMutation({
        mutationFn: (next: IHeaderDraft) => updateUserSettingsFn({ data: { profile_layout: { header: headerForSave(next) } } }),
        onSuccess: async () => {
            await invalidateProfile();
            toastManager.add({ id: `profile-header-${Date.now()}`, title: t("profile.headerEditor.saved.title"), description: t("profile.headerEditor.saved.body"), type: "success" });
            onClose();
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `profile-header-err-${Date.now()}`, title: t("profile.headerEditor.saveFailed.title"), description: describeError(err), type: "error" });
        },
    });

    const move = (index: number, delta: number) => {
        const moved = moveEntry(draft.strip, index, delta);
        if (!moved) return;
        setDraft({ ...draft, strip: moved.next });
        setAnnouncement(t("profile.headerEditor.cards.moved", { card: labels[draft.strip[index]], position: moved.to + 1, total: moved.next.length }));
    };

    const addable = STRIP_CARDS.filter((card) => !draft.strip.includes(card));
    const full = draft.strip.length >= STRIP_MAX;

    return (
        <section className="flex flex-col gap-5 rounded-2xl border border-border/50 bg-card/40 p-4 sm:p-5" aria-labelledby="profile-header-kicker">
            <div>
                <Kicker id="profile-header-kicker">{t("profile.headerEditor.kicker")}</Kicker>
                <p className="max-w-prose text-muted-foreground text-sm">{t("profile.headerEditor.help")}</p>
            </div>

            <div className="grid gap-6 lg:grid-cols-2">
                <div className="flex flex-col gap-2.5">
                    <div className="flex items-baseline justify-between gap-3">
                        <h3 className={HEADING}>{t("profile.headerEditor.cards.title")}</h3>
                        <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{t("profile.headerEditor.cards.count", { count: draft.strip.length, max: STRIP_MAX })}</span>
                    </div>
                    {draft.strip.length === 0 ? (
                        <p className="rounded-xl border border-border/60 border-dashed px-3 py-3 text-muted-foreground text-sm">{t("profile.headerEditor.cards.empty")}</p>
                    ) : (
                        <ol className="flex flex-col gap-1.5" aria-label={t("profile.headerEditor.cards.list")}>
                            {draft.strip.map((card, i) => {
                                const label = labels[card];
                                const hidden = draft.hidden.includes(card);
                                return (
                                    <li key={card} className={ROW}>
                                        <span className="w-4 shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums">{i + 1}</span>
                                        <span className={cn("min-w-0 flex-1 truncate font-medium text-sm", hidden && "text-muted-foreground")}>{label}</span>
                                        {hidden && <span className={BADGE}>{t("profile.headerEditor.hiddenBadge")}</span>}
                                        <Button type="button" size="icon-xs" variant="outline" onClick={() => move(i, -1)} disabled={i === 0} aria-label={t("profile.headerEditor.cards.moveUp", { card: label })}>
                                            <ChevronUpIcon />
                                        </Button>
                                        <Button type="button" size="icon-xs" variant="outline" onClick={() => move(i, 1)} disabled={i === draft.strip.length - 1} aria-label={t("profile.headerEditor.cards.moveDown", { card: label })}>
                                            <ChevronDownIcon />
                                        </Button>
                                        <Button type="button" size="icon-xs" variant="ghost" onClick={() => setDraft(removeCard(draft, card))} aria-label={t("profile.headerEditor.cards.remove", { card: label })}>
                                            <XIcon />
                                        </Button>
                                    </li>
                                );
                            })}
                        </ol>
                    )}
                    <p className="sr-only" aria-live="polite">
                        {announcement}
                    </p>
                    {addable.length > 0 && (
                        <div className="flex flex-col gap-2 pt-1">
                            <span className={HEADING}>{t("profile.headerEditor.cards.add")}</span>
                            <div className="flex flex-wrap gap-1.5">
                                {addable.map((card) => (
                                    <Button key={card} type="button" size="xs" variant="outline" onClick={() => setDraft(addCard(draft, card))} disabled={full} aria-label={t("profile.headerEditor.cards.addOne", { card: labels[card] })}>
                                        <PlusIcon />
                                        {labels[card]}
                                    </Button>
                                ))}
                            </div>
                            {full && <p className="text-muted-foreground text-xs">{t("profile.headerEditor.cards.full", { max: STRIP_MAX })}</p>}
                        </div>
                    )}
                </div>

                <div className="flex flex-col gap-2.5">
                    <h3 className={HEADING}>{t("profile.headerEditor.hidden.title")}</h3>
                    <ul className="grid gap-1.5 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2" aria-label={t("profile.headerEditor.hidden.list")}>
                        {HEADER_FIELDS.map((field) => {
                            const label = labels[field];
                            const hidden = draft.hidden.includes(field);
                            return (
                                <li key={field} className={ROW}>
                                    <span className={cn("min-w-0 flex-1 truncate font-medium text-sm", hidden && "text-muted-foreground")}>{label}</span>
                                    {hidden && <span className={BADGE}>{t("profile.headerEditor.hiddenBadge")}</span>}
                                    <Button type="button" size="icon-xs" variant="ghost" onClick={() => setDraft(toggleHidden(draft, field))} aria-pressed={hidden} aria-label={hidden ? t("profile.headerEditor.show", { field: label }) : t("profile.headerEditor.hide", { field: label })}>
                                        {hidden ? <EyeOffIcon /> : <EyeIcon />}
                                    </Button>
                                </li>
                            );
                        })}
                    </ul>
                </div>
            </div>

            <div className="flex flex-wrap items-center gap-2">
                <Button type="button" variant="ghost" size="sm" onClick={() => setDraft(headerDraft(null))} disabled={isDefaultHeader(draft)}>
                    <RotateCcwIcon />
                    {t("profile.headerEditor.reset")}
                </Button>
                <div className="ml-auto flex items-center gap-2">
                    <Button type="button" variant="outline" size="sm" onClick={onClose} disabled={save.isPending}>
                        {t("profile.headerEditor.cancel")}
                    </Button>
                    <Button type="button" size="sm" onClick={() => save.mutate(draft)} loading={save.isPending}>
                        {t("profile.headerEditor.save")}
                    </Button>
                </div>
            </div>
        </section>
    );
}
