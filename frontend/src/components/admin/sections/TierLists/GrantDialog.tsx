import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { invalidateTierListGrants } from "#/components/admin/shell/invalidate";
import { GRANT_LEVELS } from "#/components/admin/shell/model";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { InputGroup, InputGroupInput } from "#/components/ui/input-group";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "#/components/ui/select";
import { Skeleton } from "#/components/ui/skeleton";
import { Tabs, TabsList, TabsTab } from "#/components/ui/tabs";
import { useDebounce } from "#/hooks/use-debounce";
import { adminUsersQueryOptions, grantTierListPermissionFn, type TierListPermissionLevel } from "#/lib/api/admin";
import { useT } from "#/lib/i18n";
import { cn } from "#/lib/utils";
import { type TierListsT, useLevelLabel } from "./labels";

/** The design's level ladder as a segmented control. */
function LevelPicker({ value, onChange, label }: { value: TierListPermissionLevel; onChange: (level: TierListPermissionLevel) => void; label?: string }): React.ReactElement {
    const levelLabel = useLevelLabel();
    return (
        <Tabs value={value} onValueChange={(v) => onChange(v as TierListPermissionLevel)}>
            <TabsList aria-label={label}>
                {GRANT_LEVELS.map((level) => (
                    <TabsTab key={level} value={level}>
                        {levelLabel(level)}
                    </TabsTab>
                ))}
            </TabsList>
        </Tabs>
    );
}

export interface IGrantListOption {
    slug: string;
    title: string;
}

/** Staff only: grant one player one level on one list. */
export function GrantDialog({ open, onOpenChange, lists }: { open: boolean; onOpenChange: (open: boolean) => void; lists: readonly IGrantListOption[] }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const levelLabel = useLevelLabel();
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const [q, setQ] = useState("");
    const [user, setUser] = useState<{ id: string; name: string } | null>(null);
    const [slug, setSlug] = useState<string | null>(null);
    const [level, setLevel] = useState<TierListPermissionLevel>("edit");
    // One trimmed character is a real query (a CJK nickname prefix is one code unit): debounce, don't gate on length.
    const term = useDebounce(q.trim(), 300);
    const usersEnabled = open && term.length > 0;
    const usersQuery = useQuery({ ...adminUsersQueryOptions({ q: term, limit: 8 }, usersEnabled), enabled: usersEnabled });
    const listTitle = lists.find((l) => l.slug === slug)?.title ?? "";

    const reset = (): void => {
        setQ("");
        setUser(null);
        setSlug(null);
        setLevel("edit");
    };

    const grant = useMutation({
        mutationFn: (input: { slug: string; userId: string; permission: TierListPermissionLevel }) => grantTierListPermissionFn({ data: input }),
        onSuccess: () => {
            invalidateTierListGrants(queryClient);
            toastSuccess("tier-lists-granted", t("tierLists.access.toast.granted"), t("tierLists.access.toast.levelDesc", { name: user?.name ?? "", level: levelLabel(level), title: listTitle }));
            onOpenChange(false);
            reset();
        },
        onError: (err: unknown) => toastError("tier-lists-grant-failed", t("tierLists.access.toast.grantFailed"), describeError(err)),
    });

    const results = usersQuery.data?.users ?? [];
    const searching = q.trim().length > 0 && (term !== q.trim() || usersQuery.isPending);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-lg">
                <DialogHeader>
                    <DialogTitle>{t("tierLists.grant.title")}</DialogTitle>
                    <DialogDescription>{t("tierLists.grant.desc")}</DialogDescription>
                </DialogHeader>
                <DialogPanel className="flex flex-col gap-4">
                    <div className="flex flex-col gap-1.5">
                        <span className="font-medium text-sm">{t("tierLists.grant.player")}</span>
                        <InputGroup>
                            <InputGroupInput
                                size="sm"
                                value={q}
                                onChange={(e) => {
                                    setQ(e.target.value);
                                    setUser(null);
                                }}
                                placeholder={t("tierLists.grant.playerPlaceholder")}
                                aria-label={t("tierLists.grant.player")}
                                autoFocus
                            />
                        </InputGroup>
                        {q.trim().length === 0 ? null : searching ? (
                            <Skeleton className="h-10" />
                        ) : results.length === 0 ? (
                            <div className="text-[13px] text-muted-foreground">{t("tierLists.grant.noMatches")}</div>
                        ) : (
                            <ul className="max-h-52 overflow-auto rounded-lg border border-border">
                                {results.map((r) => {
                                    const name = r.nickname ?? t("tierLists.access.unnamed");
                                    return (
                                        <li key={r.id} className="border-border border-b last:border-0">
                                            <button type="button" aria-pressed={user?.id === r.id} onClick={() => setUser({ id: r.id, name })} className={cn("flex w-full cursor-pointer items-center justify-between gap-3 px-3 py-2 text-left hover:bg-accent", user?.id === r.id && "bg-accent")}>
                                                <span className="flex min-w-0 flex-col gap-0.5">
                                                    <span className="truncate font-medium text-[14px]" title={name}>
                                                        {name}
                                                    </span>
                                                    <span className="font-mono text-[12px] text-muted-foreground">{t("tierLists.access.uid", { uid: r.uid })}</span>
                                                </span>
                                                <span className="font-mono text-[12px] text-muted-foreground">{r.server}</span>
                                            </button>
                                        </li>
                                    );
                                })}
                            </ul>
                        )}
                    </div>
                    <div className="flex flex-col gap-1.5">
                        <span className="font-medium text-sm">{t("tierLists.grant.list")}</span>
                        <Select value={slug} onValueChange={(next: string | null) => setSlug(next)}>
                            <SelectTrigger size="sm" aria-label={t("tierLists.grant.list")}>
                                <SelectValue>{() => listTitle || t("tierLists.grant.listPlaceholder")}</SelectValue>
                            </SelectTrigger>
                            <SelectContent>
                                {lists.map((l) => (
                                    <SelectItem key={l.slug} value={l.slug}>
                                        {l.title}
                                    </SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <div className="flex flex-col gap-1.5">
                        <span className="font-medium text-sm">{t("tierLists.grant.level")}</span>
                        <LevelPicker value={level} onChange={setLevel} label={t("tierLists.grant.level")} />
                    </div>
                </DialogPanel>
                <DialogFooter>
                    <DialogClose render={<Button variant="ghost" />}>{t("tierLists.cancel")}</DialogClose>
                    <Button disabled={!user || !slug} loading={grant.isPending} onClick={() => user && slug && grant.mutate({ slug, userId: user.id, permission: level })}>
                        {t("tierLists.grant.submit", { level: levelLabel(level) })}
                    </Button>
                </DialogFooter>
            </DialogPopup>
        </Dialog>
    );
}
