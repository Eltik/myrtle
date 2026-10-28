import { useQuery } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import { useMemo, useState } from "react";
import { BannerFilter } from "#/components/gacha/history/impl/BannerFilter";
import { bannerOptions } from "#/components/gacha/history/impl/banner-filter";
import { HistoryBody, hasAnyPull, useBannersById } from "#/components/gacha/history/impl/HistoryBody";
import { PublicHistorySwitch, ShareHistoryButton } from "#/components/gacha/history/impl/ShareHistory";
import { useErrorMessage } from "#/components/ui/error-message";
import { userGachaRecordsQueryOptions } from "#/lib/api/gacha";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { IUserProfile } from "#/types/user";
import type { messages } from "./PullsTab.messages";

interface IPullsTabProps {
    uid: string;
    profile: IUserProfile;
    isOwner: boolean;
    /** The banners the tab is limited to, from the URL; empty is every banner. */
    pools: readonly string[];
    onPoolsChange: (next: string[]) => void;
}

function Note({ children }: { children: React.ReactNode }) {
    return <div className="flex flex-col items-center justify-center gap-2 rounded-3xl border border-border bg-card px-8 py-12 text-center text-muted-foreground text-sm">{children}</div>;
}

/**
 * A player's pull history on their profile: the history page's panels over
 * `/gacha/public-records`, limited to the banners in the URL, so a link the owner
 * shared opens on exactly those. The owner also gets the public switch and Share.
 */
export function PullsTab({ uid, profile, isOwner, pools, onPoolsChange }: IPullsTabProps) {
    const t: TypedT<typeof messages> = useT("user");
    const describeError = useErrorMessage();
    const records = useQuery(userGachaRecordsQueryOptions(uid, pools));
    // A filtered read carries only the chosen banners, so the menu's full list is read
    // the first time it opens rather than on every shared link.
    const [wantAll, setWantAll] = useState(false);
    const all = useQuery({ ...userGachaRecordsQueryOptions(uid, []), enabled: pools.length > 0 && wantAll });
    const bannersById = useBannersById(profile.server);

    const data = records.data ?? null;
    const optionSource = pools.length === 0 ? data : (all.data ?? data);
    const options = useMemo(() => bannerOptions(optionSource, bannersById), [optionSource, bannersById]);

    if (records.isError) {
        return (
            <Note>
                <span className="font-medium text-foreground">{t("profile.pulls.loadFailed")}</span>
                <span>{describeError(records.error)}</span>
            </Note>
        );
    }
    // `null` is a refusal: the owner made the history private after this page loaded.
    if (!records.isLoading && records.data === null) return <Note>{t("profile.pulls.private")}</Note>;

    const empty = !records.isLoading && !hasAnyPull(data);
    if (empty && pools.length === 0) {
        return (
            <Note>
                <span>{isOwner ? t("profile.pulls.ownerEmpty") : t("profile.pulls.empty")}</span>
                {isOwner ? (
                    <Link to="/gacha/history" className="font-medium text-foreground underline underline-offset-2 hover:text-primary">
                        {t("profile.pulls.historyLink")}
                    </Link>
                ) : null}
            </Note>
        );
    }

    return (
        <div className="flex flex-col gap-6">
            {isOwner && profile.public_gacha !== true ? <p className="m-0 rounded-[14px] border border-border bg-muted/40 px-4 py-3 font-sans text-muted-foreground text-sm">{t("profile.pulls.ownerPrivate")}</p> : null}
            <div className="flex flex-col gap-3 rounded-[14px] border border-border bg-card px-4 py-3 lg:flex-row lg:items-center lg:justify-between">
                <BannerFilter options={options} value={pools} onChange={onPoolsChange} onOpenChange={(open) => open && setWantAll(true)} />
                {isOwner ? (
                    <div className="flex flex-wrap items-center gap-3 sm:gap-4">
                        <PublicHistorySwitch user={profile} />
                        <ShareHistoryButton user={profile} pools={pools} />
                    </div>
                ) : null}
            </div>
            {empty ? <Note>{t("profile.pulls.emptyFiltered")}</Note> : <HistoryBody records={data} bannersById={bannersById} isLoading={records.isLoading} filtered={pools.length > 0} />}
        </div>
    );
}
