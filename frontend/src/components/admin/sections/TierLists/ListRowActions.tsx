import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ExternalLinkIcon, MoreHorizontalIcon, TagIcon, Trash2Icon } from "lucide-react";
import { useState } from "react";
import { toastError, toastSuccess } from "#/components/admin/shell/toast";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Input } from "#/components/ui/input";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "#/components/ui/menu";
import { TableCell, TableRow } from "#/components/ui/table";
import { publishTierListVersionFn, tierListVersionsQueryOptions } from "#/lib/api/tier-lists";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TierListsT } from "./labels";
import { canDeleteList, canSetFlair, type ITierListRow, nextVersionNumber } from "./model";

/** The Lists table's column count: the publish row spans all of them. */
export const COLUMN_COUNT = 8;

export function RowMenu({ row, onFlair, onDelete }: { row: ITierListRow; onFlair: () => void; onDelete: () => void }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const flair = canSetFlair(row.level);
    const del = canDeleteList(row.level);
    return (
        <DropdownMenu>
            <DropdownMenuTrigger render={<Button size="icon-sm" variant="ghost" aria-label={t("tierLists.lists.more", { title: row.title })} />}>
                <MoreHorizontalIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-44">
                <DropdownMenuItem onClick={() => window.open(`/tier-lists/${encodeURIComponent(row.slug)}`, "_blank", "noopener")}>
                    <ExternalLinkIcon />
                    {t("tierLists.menu.viewPublic")}
                </DropdownMenuItem>
                {flair ? (
                    <DropdownMenuItem onClick={onFlair}>
                        <TagIcon />
                        {t("tierLists.menu.changeFlair")}
                    </DropdownMenuItem>
                ) : null}
                {del ? (
                    <>
                        <DropdownMenuSeparator />
                        <DropdownMenuItem variant="destructive" onClick={onDelete}>
                            <Trash2Icon />
                            {t("tierLists.menu.delete")}
                        </DropdownMenuItem>
                    </>
                ) : null}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

/** The inline "what changed?" row under a list (design: at most one open). */
export function PublishRow({ row, onClose }: { row: ITierListRow; onClose: () => void }): React.ReactElement {
    const t: TierListsT = useT("admin");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const server = useGamedataServer();
    const versionsQuery = useQuery(tierListVersionsQueryOptions(row.slug, server));
    const [changelog, setChangelog] = useState("");

    const publish = useMutation({
        mutationFn: () => publishTierListVersionFn({ data: { slug: row.slug, changelog: changelog.trim() || null } }),
        onSuccess: (version) => {
            void queryClient.invalidateQueries({ queryKey: ["tier-lists"] });
            toastSuccess("tier-lists-published", t("tierLists.publish.toast"), t("tierLists.publish.toastDesc", { title: row.title, version: String(version.version) }));
            onClose();
        },
        onError: (err: unknown) => toastError("tier-lists-publish-failed", t("tierLists.publish.failed"), describeError(err)),
    });

    const prompt = versionsQuery.data ? t("tierLists.publish.prompt", { version: String(nextVersionNumber(versionsQuery.data)), title: row.title }) : t("tierLists.publish.promptPending", { title: row.title });

    return (
        <TableRow className="hover:bg-transparent!">
            <TableCell colSpan={COLUMN_COUNT}>
                {/* Pinned to the scroller's left edge and capped to the visible width, so a phone that scrolls the table sideways still sees the whole prompt. */}
                <div className="sticky left-0 flex w-[min(100%,calc(100vw-4rem))] flex-wrap items-center gap-2.5 whitespace-normal pt-1 pr-1.5 pb-1.5">
                    <span className="min-w-0 basis-full text-[13px] text-muted-foreground leading-snug sm:basis-auto">{prompt}</span>
                    <div className="min-w-48 flex-1">
                        <Input
                            size="sm"
                            autoFocus
                            value={changelog}
                            onChange={(e) => setChangelog(e.target.value)}
                            onKeyDown={(e) => {
                                if (e.key === "Enter" && !publish.isPending) publish.mutate();
                                if (e.key === "Escape") onClose();
                            }}
                            placeholder={t("tierLists.publish.changelogPlaceholder")}
                            aria-label={t("tierLists.publish.changelogLabel")}
                        />
                    </div>
                    <Button size="sm" variant="ghost" onClick={onClose}>
                        {t("tierLists.cancel")}
                    </Button>
                    <Button size="sm" loading={publish.isPending} onClick={() => publish.mutate()}>
                        {t("tierLists.publish.confirm")}
                    </Button>
                </div>
            </TableCell>
        </TableRow>
    );
}
