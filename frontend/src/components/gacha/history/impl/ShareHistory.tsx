import { useMutation } from "@tanstack/react-query";
import { Link, useRouter } from "@tanstack/react-router";
import { Share2Icon } from "lucide-react";
import { useId, useState } from "react";
import { AlertDialog, AlertDialogClose, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogPopup, AlertDialogTitle } from "#/components/ui/alert-dialog";
import { Button } from "#/components/ui/button";
import { useErrorMessage } from "#/components/ui/error-message";
import { Switch } from "#/components/ui/switch";
import { toastManager } from "#/components/ui/toast";
import { useInvalidateProfile } from "#/hooks/use-resync-roster";
import { updateUserSettingsFn } from "#/lib/api/auth";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IUserProfile } from "#/types/user";
import { formatBannerParam } from "./banner-filter";
import type { messages } from "./ShareHistory.messages";

type ShareT = TypedT<typeof messages>;

/**
 * Whether visitors can open the owner's shared history: a public profile AND the opt-in
 * `public_gacha` switch. Only an explicit `true` counts for either, as the backend's gate
 * reads them (a missing settings row is a private profile there).
 */
export function historyShareable(user: Pick<IUserProfile, "public_profile" | "public_gacha">): boolean {
    return user.public_profile === true && user.public_gacha === true;
}

/** Saves `public_gacha` and refreshes the profile it lives on, with a toast either way. */
function usePublicHistoryMutation() {
    const t: ShareT = useT("gacha");
    const describeError = useErrorMessage();
    const invalidateProfile = useInvalidateProfile();
    return useMutation({
        mutationFn: (on: boolean) => updateUserSettingsFn({ data: { public_gacha: on } }),
        onSuccess: (_, on) => {
            void invalidateProfile();
            toastManager.add({ id: `gacha-public-${Date.now()}`, title: on ? t("history.public.savedOn") : t("history.public.savedOff"), type: "success" });
        },
        onError: (err: unknown) => {
            toastManager.add({ id: `gacha-public-err-${Date.now()}`, title: t("history.public.saveFailed"), description: describeError(err), type: "error" });
        },
    });
}

/** The owner's opt-in switch, with a caption saying who can see the pulls now. */
export function PublicHistorySwitch({ user }: { user: IUserProfile }) {
    const t: ShareT = useT("gacha");
    const id = useId();
    const mutation = usePublicHistoryMutation();
    // Shows the value being saved, so the switch does not flick back while the profile refetches.
    const on = mutation.isPending ? mutation.variables === true : user.public_gacha === true;
    return (
        <div className="flex items-start gap-2.5">
            <Switch id={id} checked={on} onCheckedChange={(v) => mutation.mutate(v)} disabled={mutation.isPending} aria-describedby={`${id}-caption`} className="mt-0.5" />
            <div className="flex flex-col">
                <label htmlFor={id} className="cursor-pointer font-medium font-sans text-foreground text-sm">
                    {t("history.public.label")}
                </label>
                <span id={`${id}-caption`} className="font-sans text-muted-foreground text-xs">
                    {on ? t("history.public.on") : t("history.public.off")}
                </span>
            </div>
        </div>
    );
}

/**
 * Copies a link to the owner's profile Pulls tab with `pools` (the banner filter) in the
 * URL. While the history is private it first asks to make it public, and while the
 * whole profile is private it explains that no link can open and points to Settings,
 * since making a profile public shares far more than pulls.
 */
export function ShareHistoryButton({ user, pools }: { user: IUserProfile; pools: readonly string[] }) {
    const t: ShareT = useT("gacha");
    const router = useRouter();
    const [dialog, setDialog] = useState<"history" | "profile" | null>(null);
    const mutation = usePublicHistoryMutation();

    const copy = async () => {
        const href = router.buildLocation({ to: "/user/$id", params: { id: user.uid }, search: { tab: "pulls", banners: formatBannerParam(pools) } }).href;
        try {
            await navigator.clipboard.writeText(`${window.location.origin}${href}`);
            toastManager.add({ id: `gacha-share-${Date.now()}`, title: pools.length > 0 ? t("history.share.copiedSome", { count: pools.length }) : t("history.share.copiedAll"), type: "success" });
        } catch {
            toastManager.add({ id: `gacha-share-err-${Date.now()}`, title: t("history.share.copyFailed"), type: "error" });
        }
    };

    const onShare = () => {
        if (user.public_profile !== true) setDialog("profile");
        else if (user.public_gacha !== true) setDialog("history");
        else void copy();
    };

    const onConfirm = () => {
        mutation.mutate(true, {
            onSuccess: () => {
                setDialog(null);
                void copy();
            },
        });
    };

    return (
        <>
            <Button type="button" variant="outline" onClick={onShare}>
                <Share2Icon />
                {t("history.share.action")}
            </Button>
            <AlertDialog
                open={dialog !== null}
                onOpenChange={(open) => {
                    // A save in flight finishes before the dialog can go.
                    if (!open && !mutation.isPending) setDialog(null);
                }}
            >
                <AlertDialogPopup>
                    {dialog === "profile" ? (
                        <>
                            <AlertDialogHeader>
                                <AlertDialogTitle>{t("history.share.profilePrivate.title")}</AlertDialogTitle>
                                <AlertDialogDescription>{t("history.share.profilePrivate.body")}</AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogClose render={<Button type="button" variant="outline" />}>{t("history.share.cancel")}</AlertDialogClose>
                                <Button render={<Link to="/settings" />}>{t("history.share.profilePrivate.action")}</Button>
                            </AlertDialogFooter>
                        </>
                    ) : (
                        <>
                            <AlertDialogHeader>
                                <AlertDialogTitle>{t("history.share.private.title")}</AlertDialogTitle>
                                <AlertDialogDescription>{t("history.share.private.body")}</AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogClose render={<Button type="button" variant="outline" disabled={mutation.isPending} />}>{t("history.share.cancel")}</AlertDialogClose>
                                <Button type="button" loading={mutation.isPending} onClick={onConfirm}>
                                    {t("history.share.private.confirm")}
                                </Button>
                            </AlertDialogFooter>
                        </>
                    )}
                </AlertDialogPopup>
            </AlertDialog>
        </>
    );
}
