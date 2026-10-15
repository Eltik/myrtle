import { Eraser, Maximize2, Minimize2, MoreHorizontal, Save, Share2, Trash2, Undo2 } from "lucide-react";
import * as React from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Input } from "#/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuTrigger } from "#/components/ui/menu";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { NAME_MAX_CHARS, SQUAD_SIZE } from "../model";
import type { messages } from "./SquadToolbar.messages";

type ToolbarT = TypedT<typeof messages>;

export interface ISquadToolbarProps {
    name: string;
    filled: number;
    hasSupport: boolean;
    slot: number | null;
    /** The active slot has a saved copy on the server, so it can be deleted. */
    slotSaved: boolean;
    dirty: boolean;
    compact: boolean;
    readOnly: boolean;
    signedIn: boolean;
    saving: boolean;
    onName: (name: string) => void;
    onToggleCompact: () => void;
    onRevert: () => void;
    onClear: () => void;
    onShare: () => void;
    onSave: () => void;
    onDelete: () => void;
}

export function SquadToolbar(props: ISquadToolbarProps): React.ReactElement {
    const t: ToolbarT = useT("tools");
    const { name, filled, hasSupport, slot, slotSaved, dirty, compact, readOnly, signedIn, saving } = props;
    const [confirmOpen, setConfirmOpen] = React.useState(false);
    // Kept after close so the dialog does not swap its text while it animates out.
    const [confirm, setConfirmKind] = React.useState<"clear" | "delete">("clear");
    const setConfirm = (kind: "clear" | "delete") => {
        setConfirmKind(kind);
        setConfirmOpen(true);
    };
    const empty = filled === 0 && !hasSupport;

    return (
        <div className="flex flex-col gap-3 rounded-xl border border-border bg-card p-3 md:flex-row md:items-center">
            <div className="flex min-w-0 flex-1 flex-wrap items-center gap-x-3 gap-y-1.5">
                <Input value={name} maxLength={NAME_MAX_CHARS} disabled={readOnly} onChange={(e) => props.onName(e.target.value)} placeholder={t("squad.toolbar.name")} aria-label={t("squad.toolbar.name")} className="w-full max-w-72 font-semibold" />
                <span className="font-mono text-[12px] text-muted-foreground tabular-nums">
                    {t("squad.toolbar.count", { count: filled, total: SQUAD_SIZE })}
                    {hasSupport && <> {t("squad.toolbar.plusSupport")}</>}
                </span>
                {!readOnly && <Badge variant="outline">{slot === null ? t("squad.toolbar.scratch") : t("squad.toolbar.inSlot", { slot })}</Badge>}
            </div>
            <div className="flex flex-wrap items-center gap-1.5">
                <Button variant="ghost" size="sm" onClick={props.onToggleCompact}>
                    {compact ? <Maximize2 /> : <Minimize2 />}
                    {compact ? t("squad.toolbar.expand") : t("squad.toolbar.minimize")}
                </Button>
                {!readOnly && (
                    <>
                        <Button variant="ghost" size="sm" disabled={!dirty} onClick={props.onRevert}>
                            <Undo2 />
                            {t("squad.toolbar.revert")}
                        </Button>
                        <Button variant="ghost" size="sm" disabled={empty} onClick={() => setConfirm("clear")}>
                            <Eraser />
                            {t("squad.toolbar.clear")}
                        </Button>
                    </>
                )}
                <Button variant="outline" size="sm" disabled={empty} onClick={props.onShare}>
                    <Share2 />
                    {t("squad.toolbar.share")}
                </Button>
                {!readOnly && (
                    <Button size="sm" disabled={signedIn && !dirty} loading={saving} onClick={props.onSave}>
                        <Save />
                        {signedIn ? t("squad.toolbar.save") : t("squad.toolbar.signInToSave")}
                    </Button>
                )}
                {!readOnly && slot !== null && slotSaved && (
                    <Menu>
                        <MenuTrigger render={(p) => <Button {...p} variant="ghost" size="icon-sm" aria-label={t("squad.toolbar.more")} />}>
                            <MoreHorizontal />
                        </MenuTrigger>
                        <MenuPopup align="end">
                            <MenuItem variant="destructive" onClick={() => setConfirm("delete")}>
                                <Trash2 />
                                {t("squad.toolbar.delete", { slot })}
                            </MenuItem>
                        </MenuPopup>
                    </Menu>
                )}
            </div>

            <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
                <AlertDialogPopup>
                    <AlertDialogHeader>
                        <AlertDialogTitle>{confirm === "delete" ? t("squad.toolbar.deleteTitle", { slot: slot ?? 0 }) : t("squad.toolbar.clearTitle")}</AlertDialogTitle>
                        <AlertDialogDescription>{confirm === "delete" ? t("squad.toolbar.deleteDesc") : t("squad.toolbar.clearDesc")}</AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogClose render={(p) => <Button {...p} variant="outline" />}>{t("squad.toolbar.cancel")}</AlertDialogClose>
                        <AlertDialogClose render={(p) => <Button {...p} variant="destructive" onClick={confirm === "delete" ? props.onDelete : props.onClear} />}>{confirm === "delete" ? t("squad.toolbar.deleteConfirm") : t("squad.toolbar.clear")}</AlertDialogClose>
                    </AlertDialogFooter>
                </AlertDialogPopup>
            </AlertDialog>
        </div>
    );
}
