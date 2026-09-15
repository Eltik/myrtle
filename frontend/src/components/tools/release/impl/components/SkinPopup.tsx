import { useQuery } from "@tanstack/react-query";
import * as React from "react";
import { SkinDetailContent } from "#/components/skins/SkinDetailDialog";
import { Dialog, DialogContent, DialogTitle } from "#/components/ui/dialog";
import { DynamicArtProvider } from "#/components/user/profile/impl/dynamic-art";
import { operatorsIndexQueryOptions } from "#/lib/api/operators";
import { type ISkinIndexEntry, skinsIndexQueryOptions } from "#/lib/api/skins";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, getAvatarById } from "#/lib/utils";
import type { AutoName } from "#/types/generated/AutoName";
import { useAutoTranslate } from "../autoTranslate";
import type { messages } from "./SkinPopup.messages";
import { buildOperatorLookup, CnName, type OperatorLookup, operatorLabel, useArt } from "./shared";

type PopupT = TypedT<typeof messages>;

export interface ISkinSelection {
    charId: string;
    skinId: string;
    skinName: string;
    charName: AutoName | null;
}

const SkinPopupContext = React.createContext<(sel: ISkinSelection) => void>(() => {});

export function useSkinPopup(): (sel: ISkinSelection) => void {
    return React.useContext(SkinPopupContext);
}

export function SkinPopupProvider({ children }: { children: React.ReactNode }): React.ReactElement {
    const index = useQuery(operatorsIndexQueryOptions(useGamedataServer()));
    const lookup = React.useMemo(() => buildOperatorLookup(index.data), [index.data]);
    const [selected, setSelected] = React.useState<ISkinSelection | null>(null);
    const close = React.useCallback((open: boolean) => {
        if (!open) setSelected(null);
    }, []);
    return (
        <SkinPopupContext.Provider value={setSelected}>
            <Dialog open={selected !== null} onOpenChange={close}>
                {selected && <SkinPopup selection={selected} lookup={lookup} />}
            </Dialog>
            {children}
        </SkinPopupContext.Provider>
    );
}

function SkinPopup({ selection, lookup }: { selection: ISkinSelection; lookup: OperatorLookup }): React.ReactElement {
    const t: PopupT = useT("tools");
    const autoOn = useAutoTranslate();
    const en = useQuery(skinsIndexQueryOptions("en"));
    const cn = useQuery(skinsIndexQueryOptions("cn"));
    const entry: { skin: ISkinIndexEntry; server: "en" | "cn" } | null = React.useMemo(() => {
        const e = en.data?.[selection.skinId];
        if (e) return { skin: e, server: "en" };
        const c = cn.data?.[selection.skinId];
        if (c) return { skin: c, server: "cn" };
        return null;
    }, [en.data, cn.data, selection.skinId]);
    const opName = operatorLabel(selection.charId, lookup.get(selection.charId), selection.charName, autoOn).text;
    if (!entry) {
        const pending = en.isPending || cn.isPending;
        return (
            <DialogContent bottomStickOnMobile={false} className="w-full max-w-120 p-6" showCloseButton>
                <DialogTitle className="font-heading font-semibold text-base">{selection.skinName}</DialogTitle>
                <p className="font-sans text-[13px] text-muted-foreground">{pending ? t("release.popup.loading") : t("release.popup.missing")}</p>
            </DialogContent>
        );
    }
    return (
        <DynamicArtProvider server={entry.server}>
            <SkinDetailContent skin={entry.skin} server={entry.server} opName={opName} skinName={entry.skin.displaySkin?.skinName ?? selection.skinName} avatarURL={getAvatarById(selection.charId, entry.server === "cn" ? "cn" : undefined)} closeLabel={t("release.popup.close")} />
        </DynamicArtProvider>
    );
}

export interface ISkinTileViewProps {
    skinId: string;
    charId: string;
    charName: AutoName | null;
    skinName: string;
    skinNameEn?: string | null;
    skinNameAuto?: AutoName | null;
    portraitPath: string | null;
    lookup: OperatorLookup;
    title?: string;
}

export function SkinTileView({ skinId, charId, charName, skinName, skinNameEn, skinNameAuto, portraitPath, lookup, title }: ISkinTileViewProps): React.ReactElement {
    const t: PopupT = useT("tools");
    const autoOn = useAutoTranslate();
    const openPopup = useSkinPopup();
    const art = useArt(portraitPath);
    const entry = charId ? lookup.get(charId) : undefined;
    const op = operatorLabel(charId, entry, charName, autoOn);
    const initial = (entry ? op.text : op.text.replace(/^char_\d+_/, "")).trim().charAt(0).toUpperCase() || "?";
    const body = (
        <>
            <span className="relative block aspect-[3/5] w-full overflow-hidden rounded-md bg-zinc-900">
                {art.src ? (
                    <img src={art.src} alt={t("release.popup.tileAlt", { operator: op.text, skin: skinName })} loading="lazy" onError={art.onError} className="absolute inset-0 h-full w-full object-cover object-top" />
                ) : (
                    <span className="absolute inset-0 flex items-center justify-center font-bold font-sans text-[28px] text-zinc-600">{initial}</span>
                )}
            </span>
            <span className={cn("mt-1 block truncate font-medium font-sans text-[12px] leading-tight", entry ? "text-foreground" : "text-muted-foreground")}>{op.text}</span>
            <CnName cn={skinName} en={skinNameEn} auto={skinNameAuto} compact primaryClassName="font-sans text-[11px] text-muted-foreground leading-tight" />
        </>
    );
    const className = "block w-22 shrink-0 sm:w-27.5";
    if (charId) {
        return (
            <button type="button" onClick={() => openPopup({ charId, skinId, skinName, charName })} className={cn(className, "cursor-pointer rounded-md text-left hover:opacity-90")} title={title}>
                {body}
            </button>
        );
    }
    return (
        <span className={className} title={title}>
            {body}
        </span>
    );
}
