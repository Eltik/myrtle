import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useEffect, useRef, useState } from "react";
import { useRoleLabel } from "#/components/admin/shell/nav";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Sheet, SheetDescription, SheetFooter, SheetHeader, SheetPanel, SheetPopup, SheetTitle } from "#/components/ui/sheet";
import { useLastDefined } from "#/hooks/use-last-defined";
import { type IAdminUser, setUserRoleFn, type TierListPermissionLevel, type UserRole } from "#/lib/api/admin";
import { browseTierListsQueryOptions } from "#/lib/api/tier-lists";
import { useFormatters, useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { ROLE_ORDER, roleVariant, useLevelLabel, useRoleDescription } from "./labels";
import { PersonAvatar } from "./PersonAvatar";
import { GrantPicker, GrantRow } from "./PersonGrants";
import type { messages } from "./Users.messages";
import type { IPeopleGrants } from "./usePeopleData";
import { usePersonGrantMutations } from "./usePersonGrantMutations";
import { type AccessWarning, accessWarning, grantsOf, personLabel, pickerForWarning } from "./warnings";

type UsersT = TypedT<typeof messages>;

interface ILocaleOption {
    code: string;
    englishName: string;
    isSource: boolean;
}

export interface IPersonSheetProps {
    /**
     * `null` = closed; `"missing"` = the linked account is not in any loaded
     * list; `"deleted"` = the account is gone but still holds grants.
     */
    person: IAdminUser | "missing" | "deleted" | null;
    onClose: () => void;
    meId: string | null;
    canAssign: boolean;
    grants: IPeopleGrants | undefined;
    locales: readonly ILocaleOption[];
    localeName: (code: string) => string;
}

const SECTION_LABEL = "font-semibold text-[11px] text-muted-foreground uppercase tracking-[0.06em]";
const MUTED_LINE = "text-[13px] text-muted-foreground";

function warningText(t: UsersT, warning: AccessWarning, name: string): string {
    switch (warning) {
        case "translatorNoEdit":
            return t("users.warn.translatorNoEdit", { name });
    }
}

/** The backend refuses a self role change with a 400 naming "your own role". */
function roleErrorMessage(err: unknown, t: UsersT, describeError: (err: unknown) => string): string {
    const raw = err instanceof Error ? err.message : String(err);
    if (raw.toLowerCase().includes("your own role")) return t("users.sheet.roleSelf");
    return describeError(err);
}

/** The People detail sheet (design `su`): role, score, grants, and the controls to change them. */
export function PersonSheet({ person, onClose, ...rest }: IPersonSheetProps): React.ReactElement {
    // Keep the last person on screen while the sheet animates closed.
    const current = useLastDefined(person);
    return (
        <Sheet open={person !== null} onOpenChange={(open) => (open ? undefined : onClose())}>
            <SheetPopup side="right">{current === "missing" || current === "deleted" ? <MissingPerson deleted={current === "deleted"} /> : current ? <PersonBody key={current.id} person={current} {...rest} /> : null}</SheetPopup>
        </Sheet>
    );
}

function MissingPerson({ deleted }: { deleted: boolean }): React.ReactElement {
    const t: UsersT = useT("admin");
    return (
        <>
            <SheetHeader>
                <SheetTitle>{deleted ? t("users.deletedAccount") : t("users.sheet.notFound.title")}</SheetTitle>
            </SheetHeader>
            <SheetPanel>
                <p className={MUTED_LINE}>{deleted ? t("users.sheet.deleted.body") : t("users.sheet.notFound.body")}</p>
            </SheetPanel>
        </>
    );
}

type Picker = "tierLists" | "locales" | null;

function PersonBody({ person, meId, canAssign, grants, locales, localeName }: Omit<IPersonSheetProps, "person" | "onClose"> & { person: IAdminUser }): React.ReactElement {
    const t: UsersT = useT("admin");
    const f = useFormatters();
    const roleLabel = useRoleLabel();
    const roleDescription = useRoleDescription();
    const levelLabel = useLevelLabel();
    const queryClient = useQueryClient();
    const describeError = useErrorMessage();

    const name = personLabel(person, t("users.deletedAccount"));
    const isSelf = person.id === meId;
    const held = grants ? grantsOf(grants.byUser, person.id) : undefined;
    const warning = held ? accessWarning(person.role, held) : null;
    const tlViaRole = person.role === "tier_list_admin" || person.role === "super_admin";
    const locViaRole = person.role === "super_admin";

    const [picker, setPicker] = useState<Picker>(null);
    const [level, setLevel] = useState<TierListPermissionLevel>("edit");
    // "Fix access" lands with the picker that fixes the warning already open,
    // once the grants it is derived from have loaded.
    const autoOpened = useRef(false);
    useEffect(() => {
        if (autoOpened.current || !held || !canAssign) return;
        autoOpened.current = true;
        setPicker(pickerForWarning(warning));
    }, [held, canAssign, warning]);

    const togglePicker = (which: Exclude<Picker, null>) => {
        setPicker((open) => (open === which ? null : which));
        setLevel("edit");
    };

    const officialQuery = useQuery({ ...browseTierListsQueryOptions(useGamedataServer()), enabled: canAssign && picker === "tierLists" });
    const tlOptions = (officialQuery.data ?? []).filter((l) => l.listType === "official" && !held?.tierLists.some((g) => g.slug === l.slug)).sort((a, b) => f.collator.compare(a.title, b.title));
    const locOptions = locales.filter((l) => !l.isSource && !held?.locales.some((g) => g.locale === l.code));

    const setRole = useMutation({
        mutationFn: (role: UserRole) => setUserRoleFn({ data: { userId: person.id, role } }),
        onSuccess: (_data, role) => {
            void queryClient.invalidateQueries({ queryKey: ["admin", "users"] });
            void queryClient.invalidateQueries({ queryKey: ["admin", "stats"] });
            toastSuccess("role-set", t("users.toast.roleUpdated"), t("users.toast.roleUpdated.desc", { name, role: roleLabel(role) }));
        },
        onError: (err: unknown) => toastError("role-set-err", t("users.toast.roleFailed"), roleErrorMessage(err, t, describeError)),
    });

    const { grantTl, revokeTl, grantLoc, revokeLoc } = usePersonGrantMutations({ personId: person.id, name, localeName, onGranted: () => setPicker(null) });

    // A grant names its granter; null when the CLI wrote it or the account is gone.
    const grantedBy = (byName: string | null, at: string) => (byName ? t("users.sheet.grantedBy", { name: byName, when: f.relative(at) }) : t("users.sheet.grantedAt", { when: f.relative(at) }));

    const scores: [string, number | null][] = [
        [t("users.sheet.score.total"), person.totalScore === null ? null : Math.round(person.totalScore)],
        [t("users.sheet.score.operators"), person.operatorCount],
        [t("users.sheet.score.items"), person.itemCount],
        [t("users.sheet.score.skins"), person.skinCount],
    ];

    const tlRows = [...(held?.tierLists ?? [])].sort((a, b) => f.collator.compare(a.title, b.title));
    const locRows = [...(held?.locales ?? [])].sort((a, b) => f.collator.compare(localeName(a.locale), localeName(b.locale)));

    return (
        <>
            <SheetHeader className="flex-row items-center gap-3">
                <PersonAvatar avatarId={person.avatarId} name={name} className="size-11 text-[13px]" />
                <div className="flex min-w-0 flex-col gap-1">
                    <SheetTitle>{name}</SheetTitle>
                    <SheetDescription>{person.level === null ? t("users.sheet.metaNoLevel", { uid: person.uid, server: person.server }) : t("users.sheet.meta", { uid: person.uid, server: person.server, level: f.number(person.level) })}</SheetDescription>
                </div>
            </SheetHeader>
            <SheetPanel>
                <div className="@container flex flex-col gap-[22px]">
                    <div className="flex flex-wrap gap-1.5">
                        <Badge variant={roleVariant(person.role)}>{roleLabel(person.role)}</Badge>
                        {person.grade ? <Badge variant="outline">{t("users.sheet.grade", { grade: person.grade })}</Badge> : null}
                        <Badge variant={person.publicProfile ? "success" : "warning"}>{person.publicProfile ? t("users.sheet.publicProfile") : t("users.sheet.privateProfile")}</Badge>
                    </div>

                    <div className="grid @sm:grid-cols-4 grid-cols-2 gap-2 rounded-[10px] bg-muted p-3">
                        {scores.map(([label, value]) => (
                            <div key={label} className="flex flex-col gap-0.5">
                                <span className="text-[11.5px] text-muted-foreground">{label}</span>
                                <span className="font-semibold text-[14px] tabular-nums">{value === null ? "—" : f.number(value)}</span>
                            </div>
                        ))}
                    </div>

                    {warning ? <div className="rounded-[10px] border border-[color-mix(in_oklch,var(--warning)_45%,transparent)] bg-[color-mix(in_oklch,var(--warning)_10%,transparent)] px-3 py-2.5 text-[13px] text-warning-foreground leading-[1.45]">{warningText(t, warning, name)}</div> : null}

                    <div className="flex flex-col gap-2">
                        <span className={SECTION_LABEL}>{t("users.sheet.section.role")}</span>
                        {canAssign ? (
                            <div className="flex flex-wrap gap-1.5">
                                {ROLE_ORDER.map((role) => (
                                    <Button key={role} size="xs" variant={role === person.role ? "default" : "outline"} className="pointer-coarse:h-9 pointer-coarse:px-3" disabled={(isSelf && role !== person.role) || setRole.isPending} onClick={() => role !== person.role && !isSelf && setRole.mutate(role)}>
                                        {roleLabel(role)}
                                    </Button>
                                ))}
                            </div>
                        ) : null}
                        <p className="m-0 text-pretty text-[13px] text-muted-foreground leading-normal">{roleDescription(person.role)}</p>
                        {isSelf ? <span className="text-[12.5px] text-muted-foreground">{t("users.sheet.roleSelf")}</span> : null}
                    </div>

                    <div className="flex flex-col gap-2">
                        <div className="flex items-center justify-between">
                            <span className={SECTION_LABEL}>{t("users.sheet.section.tierLists")}</span>
                            {canAssign ? (
                                <Button size="xs" variant="ghost" aria-expanded={picker === "tierLists"} onClick={() => togglePicker("tierLists")}>
                                    {t("users.sheet.grant")}
                                </Button>
                            ) : null}
                        </div>
                        {tlViaRole ? <div className={MUTED_LINE}>{t("users.sheet.tl.viaRole")}</div> : null}
                        {held === undefined ? <div className={MUTED_LINE}>{t("users.sheet.loadingGrants")}</div> : null}
                        {tlRows.map((g) => (
                            <GrantRow
                                key={`${g.slug}:${g.permission}`}
                                label={g.title}
                                by={grantedBy(g.grantedByNickname, g.grantedAt)}
                                level={g.permission}
                                levelLabel={levelLabel(g.permission)}
                                onRevoke={canAssign ? () => revokeTl.mutate({ slug: g.slug, title: g.title, permission: g.permission as TierListPermissionLevel }) : undefined}
                                revoking={revokeTl.isPending}
                                revokeLabel={t("users.sheet.revoke")}
                            />
                        ))}
                        {held && tlRows.length === 0 && !tlViaRole ? <div className={MUTED_LINE}>{t("users.sheet.tl.empty")}</div> : null}
                        {canAssign && picker === "tierLists" ? (
                            <GrantPicker
                                level={level}
                                onLevel={setLevel}
                                levelLabel={levelLabel}
                                options={tlOptions.map((l) => ({ id: l.slug, label: l.title }))}
                                loading={officialQuery.isPending}
                                emptyLabel={t("users.sheet.picker.noLists")}
                                busy={grantTl.isPending}
                                onPick={(slug, label) => grantTl.mutate({ slug, title: label, permission: level })}
                            />
                        ) : null}
                    </div>

                    <div className="flex flex-col gap-2">
                        <div className="flex items-center justify-between">
                            <span className={SECTION_LABEL}>{t("users.sheet.section.locales")}</span>
                            {canAssign ? (
                                <Button size="xs" variant="ghost" aria-expanded={picker === "locales"} onClick={() => togglePicker("locales")}>
                                    {t("users.sheet.grant")}
                                </Button>
                            ) : null}
                        </div>
                        {locViaRole ? <div className={MUTED_LINE}>{t("users.sheet.loc.viaRole")}</div> : null}
                        {locRows.map((g) => (
                            <GrantRow
                                key={`${g.locale}:${g.permission}`}
                                label={localeName(g.locale)}
                                by={grantedBy(g.granted_by_nickname, g.granted_at)}
                                level={g.permission}
                                levelLabel={levelLabel(g.permission)}
                                onRevoke={canAssign ? () => revokeLoc.mutate({ locale: g.locale, permission: g.permission as TierListPermissionLevel }) : undefined}
                                revoking={revokeLoc.isPending}
                                revokeLabel={t("users.sheet.revoke")}
                            />
                        ))}
                        {held && locRows.length === 0 && !locViaRole ? <div className={MUTED_LINE}>{t("users.sheet.loc.empty")}</div> : null}
                        {canAssign && picker === "locales" ? (
                            <GrantPicker
                                level={level}
                                onLevel={setLevel}
                                levelLabel={levelLabel}
                                options={locOptions.map((l) => ({ id: l.code, label: l.englishName }))}
                                loading={false}
                                emptyLabel={t("users.sheet.picker.noLocales")}
                                busy={grantLoc.isPending}
                                onPick={(code) => grantLoc.mutate({ locale: code, permission: level })}
                            />
                        ) : null}
                    </div>
                </div>
            </SheetPanel>
            <SheetFooter className="sm:items-center">
                <span className="flex-1 self-center text-[12.5px] text-muted-foreground">{t("users.sheet.footer")}</span>
                <Button size="sm" variant="outline" render={<Link to="/user/$id" params={{ id: person.uid }} target="_blank" />}>
                    {t("users.sheet.profile")}
                </Button>
            </SheetFooter>
        </>
    );
}
