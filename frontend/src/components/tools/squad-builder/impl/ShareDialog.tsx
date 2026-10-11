import { useRouter } from "@tanstack/react-router";
import { Copy, Download, ImageIcon, Link2 } from "lucide-react";
import * as React from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { Input } from "#/components/ui/input";
import { Label } from "#/components/ui/label";
import { Skeleton } from "#/components/ui/skeleton";
import { Switch } from "#/components/ui/switch";
import { toastManager } from "#/components/ui/toast";
import { useFormatters, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { downloadBlob, getPortraitById } from "#/lib/utils";
import type { IOperatorIndexEntry } from "#/types/operators";
import { SQUAD_SIZE } from "../constants";
import { encodeShare, type IResolvedSeat, type ISquadDraft } from "../model";
import { canvasToBlob, type IShareImageData, type IShareImageSeat, renderSquadImage } from "../shareImage";
import type { messages } from "./ShareDialog.messages";

type ShareT = TypedT<typeof messages>;

export interface IShareDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    draft: ISquadDraft;
    seats: ReadonlyArray<IResolvedSeat | null>;
    support: IResolvedSeat | null;
    operators: ReadonlyMap<string, IOperatorIndexEntry>;
    /** Professions playable operators have (`ThemeStats.class_total`, the server's `SquadIndex.professions`); `undefined` before the analysis loads. */
    classTotal: number | undefined;
}

const PAGE_PATH = "/tools/squad-builder";

export function ShareDialog({ open, onOpenChange, draft, seats, support, operators, classTotal }: IShareDialogProps): React.ReactElement {
    const t: ShareT = useT("tools");
    const fmt = useFormatters();
    const router = useRouter();
    const [includeBuilds, setIncludeBuilds] = React.useState(true);
    const title = draft.name.trim() || t("squad.share.untitled");

    const url = React.useMemo(() => {
        if (!open || typeof window === "undefined") return "";
        const search = encodeShare(draft, includeBuilds, { members: seats, support });
        // The router's own encoder, so a name like "123" comes back as a string.
        const href = router.buildLocation({ to: PAGE_PATH, search: search as never }).href;
        return `${window.location.origin}${href}`;
    }, [open, draft, includeBuilds, seats, support, router]);

    const copyLink = async () => {
        try {
            await navigator.clipboard.writeText(url);
            toastManager.add({ id: `squad-share-${Date.now()}`, title: t("squad.share.copied"), type: "success" });
        } catch {
            toastManager.add({ id: `squad-share-err-${Date.now()}`, title: t("squad.share.copyFailed"), type: "error" });
        }
    };

    // ---- image
    const canvasRef = React.useRef<HTMLCanvasElement | null>(null);
    const [preview, setPreview] = React.useState<string | null>(null);
    const [failed, setFailed] = React.useState(false);

    const imageData = React.useMemo((): IShareImageData => {
        const toSeat = (r: IResolvedSeat | null): IShareImageSeat | null => {
            const op = r ? operators.get(r.member.char_id) : undefined;
            if (!r || !op) return null;
            return { name: op.name, rarity: op.rarity, phase: r.build.phase, level: r.build.level, portraitUrl: getPortraitById(op.id) };
        };
        const ops = seats.map((s) => (s ? operators.get(s.member.char_id) : undefined)).filter((o): o is IOperatorIndexEntry => o !== undefined);
        const classes = new Set(ops.map((o) => o.profession));
        // The top rarity is the highest any operator in the index has; the
        // class total falls back to the index's own professions until the
        // analysis answers.
        let topRarity = 0;
        const allClasses = new Set<string>();
        for (const o of operators.values()) {
            topRarity = Math.max(topRarity, o.rarity);
            allClasses.add(o.profession);
        }
        return {
            title,
            seats: Array.from({ length: SQUAD_SIZE }, (_, i) => toSeat(seats[i] ?? null)),
            support: toSeat(support),
            facts: [
                { label: t("squad.share.fact.rarity"), value: fmt.number(ops.reduce((sum, o) => sum + o.rarity, 0)) },
                { label: t("squad.share.fact.topRarity", { stars: topRarity }), value: fmt.number(ops.filter((o) => o.rarity === topRarity).length) },
                { label: t("squad.share.fact.medics"), value: fmt.number(ops.filter((o) => o.profession === "MEDIC").length) },
                { label: t("squad.share.fact.classes"), value: t("squad.share.fact.classesValue", { count: classes.size, total: classTotal ?? allClasses.size }) },
            ],
            labels: {
                leader: t("squad.share.leader"),
                empty: t("squad.share.empty"),
                support: t("squad.share.support"),
                build: (phase, level) => t("squad.share.build", { phase, level }),
                facts: t("squad.share.facts"),
            },
            footer: typeof window === "undefined" ? PAGE_PATH : `${window.location.host}${PAGE_PATH}`,
        };
    }, [title, seats, support, operators, classTotal, t, fmt]);

    React.useEffect(() => {
        if (!open) return;
        let cancelled = false;
        setPreview(null);
        setFailed(false);
        const canvas = canvasRef.current ?? document.createElement("canvas");
        canvasRef.current = canvas;
        renderSquadImage(canvas, imageData)
            .then(() => {
                if (!cancelled) setPreview(canvas.toDataURL("image/png"));
            })
            .catch((err: unknown) => {
                console.error("Squad image failed:", err);
                if (!cancelled) setFailed(true);
            });
        return () => {
            cancelled = true;
        };
    }, [open, imageData]);

    const fileName = `${title.replace(/[\\/:*?"<>|]+/g, "").trim() || "squad"}.png`;

    const copyImage = async () => {
        const canvas = canvasRef.current;
        try {
            if (!canvas || typeof ClipboardItem === "undefined" || !navigator.clipboard?.write) throw new Error("unsupported");
            const blob = await canvasToBlob(canvas);
            await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
            toastManager.add({ id: `squad-img-${Date.now()}`, title: t("squad.share.imageCopied"), type: "success" });
        } catch {
            toastManager.add({ id: `squad-img-err-${Date.now()}`, title: t("squad.share.imageCopyFailed"), type: "error" });
        }
    };

    const saveImage = async () => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        downloadBlob(await canvasToBlob(canvas), fileName);
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogPopup className="max-w-2xl">
                <DialogHeader>
                    <DialogTitle>{t("squad.share.title")}</DialogTitle>
                </DialogHeader>
                <DialogPanel>
                    <div className="flex flex-col gap-6">
                        <section className="flex flex-col gap-2.5">
                            <div>
                                <h3 className="flex items-center gap-1.5 font-semibold text-[14px]">
                                    <Link2 className="size-4 text-muted-foreground" aria-hidden="true" />
                                    {t("squad.share.link")}
                                </h3>
                                <p className="text-[12.5px] text-muted-foreground">{t("squad.share.linkDesc")}</p>
                            </div>
                            <div className="flex gap-2">
                                <Input readOnly value={url} aria-label={t("squad.share.linkField")} onFocus={(e) => e.currentTarget.select()} className="min-w-0 flex-1 font-mono text-[12px]" />
                                <Button onClick={copyLink} disabled={!url}>
                                    <Copy />
                                    {t("squad.share.copyLink")}
                                </Button>
                            </div>
                            <Label className="flex cursor-pointer items-start gap-2.5">
                                <Switch checked={includeBuilds} onCheckedChange={setIncludeBuilds} className="mt-0.5" />
                                <span>
                                    <span className="block font-medium text-[13px]">{t("squad.share.builds")}</span>
                                    <span className="block font-normal text-[12px] text-muted-foreground">{t("squad.share.buildsDesc")}</span>
                                </span>
                            </Label>
                        </section>

                        <section className="flex flex-col gap-2.5">
                            <div>
                                <h3 className="flex items-center gap-1.5 font-semibold text-[14px]">
                                    <ImageIcon className="size-4 text-muted-foreground" aria-hidden="true" />
                                    {t("squad.share.image")}
                                </h3>
                                <p className="text-[12.5px] text-muted-foreground">{t("squad.share.imageDesc")}</p>
                            </div>
                            <div className="overflow-hidden rounded-lg border border-border bg-muted">
                                {preview ? (
                                    <img alt={t("squad.share.imageAlt", { name: title })} src={preview} className="block aspect-[16/9] w-full" />
                                ) : failed ? (
                                    <p className="flex aspect-[16/9] items-center justify-center text-[13px] text-muted-foreground">{t("squad.share.renderFailed")}</p>
                                ) : (
                                    <div className="relative aspect-[16/9]">
                                        <Skeleton className="absolute inset-0 rounded-none" />
                                        <span className="absolute inset-0 flex items-center justify-center text-[12.5px] text-muted-foreground">{t("squad.share.rendering")}</span>
                                    </div>
                                )}
                            </div>
                            <div className="flex flex-wrap gap-2">
                                <Button variant="outline" onClick={copyImage} disabled={!preview}>
                                    <Copy />
                                    {t("squad.share.copyImage")}
                                </Button>
                                <Button variant="outline" onClick={saveImage} disabled={!preview}>
                                    <Download />
                                    {t("squad.share.saveImage")}
                                </Button>
                            </div>
                        </section>
                    </div>
                </DialogPanel>
            </DialogPopup>
        </Dialog>
    );
}
