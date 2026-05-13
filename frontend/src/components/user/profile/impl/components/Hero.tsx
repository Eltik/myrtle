import { Copy, Share2 } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { Tooltip, TooltipPopup, TooltipTrigger } from "#/components/ui/tooltip";
import { getLevelProgress, type ILevelProgress, MAX_PLAYER_LEVEL } from "#/lib/registry/player-level";
import { DEFAULT_AVATAR_ID, getAvatarById } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import shared from "./shared.module.css";

interface IHeroProps {
    profile: IUserProfile;
}

async function copyToClipboard(text: string, successTitle: string, successDescription: string, errorTitle: string) {
    try {
        await navigator.clipboard.writeText(text);
        toastManager.add({
            id: `copy-${Date.now()}`,
            title: successTitle,
            description: successDescription,
            type: "success",
        });
    } catch {
        toastManager.add({
            id: `copy-error-${Date.now()}`,
            title: errorTitle,
            description: "Clipboard access was denied.",
            type: "error",
        });
    }
}

export function Hero({ profile }: IHeroProps) {
    const [avatarErrored, setAvatarErrored] = useState(false);
    const levelProgress = getLevelProgress(profile.level, profile.exp);
    const avatarSrc = getAvatarById(profile.avatar_id ?? DEFAULT_AVATAR_ID);

    const handleShare = () => copyToClipboard(window.location.href, "Link copied", "Profile link copied to clipboard.", "Couldn't copy link");

    const handleCopyUid = () => copyToClipboard(profile.uid, "UID copied", `${profile.uid} copied to clipboard.`, "Couldn't copy UID");

    const displayNickname = profile.nickname ?? `Doctor ${profile.uid}`;
    const usernameWithDiscriminator = profile.nick_number ? `${displayNickname}#${profile.nick_number}` : displayNickname;
    const handleCopyUsername = () => copyToClipboard(usernameWithDiscriminator, "Username copied", `${usernameWithDiscriminator} copied to clipboard.`, "Couldn't copy username");

    return (
        <section className="relative overflow-hidden rounded-2xl border border-[oklch(0.28_0.005_285)] bg-card shadow-sm sm:rounded-3xl">
            <div
                className="pointer-events-none absolute inset-0"
                style={{
                    background: "radial-gradient(ellipse 80% 60% at 60% 40%, oklch(0.58 0.22 25 / 0.07), transparent 70%), radial-gradient(ellipse 60% 80% at 80% 20%, oklch(0.696 0.17 162 / 0.06), transparent 70%)",
                }}
            />
            <div
                className="pointer-events-none absolute inset-0 opacity-[0.06]"
                style={{
                    backgroundImage: "linear-gradient(var(--border) 1px, transparent 1px), linear-gradient(90deg, var(--border) 1px, transparent 1px)",
                    backgroundSize: "44px 44px",
                    maskImage: "radial-gradient(ellipse 60% 50% at 30% 50%, black 0%, transparent 80%)",
                    WebkitMaskImage: "radial-gradient(ellipse 60% 50% at 30% 50%, black 0%, transparent 80%)",
                }}
            />

            {/* Mobile-first: stacked grid; on sm+ becomes the original 3-column row */}
            <div className="relative grid grid-cols-[auto_1fr] items-start gap-4 px-5 pt-5 pb-5 sm:grid-cols-[auto_1fr_auto] sm:items-center sm:gap-6 sm:px-7 sm:pt-7 sm:pb-6 lg:gap-7 lg:px-8 lg:pt-8 lg:pb-7">
                {/* Avatar */}
                <div className="relative">
                    <div
                        className="relative flex h-20 w-20 items-center justify-center overflow-hidden rounded-2xl shadow-[inset_0_1px_0_rgb(255_255_255/0.45),inset_0_-2px_0_rgb(0_0_0/0.18),0_0_0_1px_rgb(255_255_255/0.06),0_8px_24px_rgb(0_0_0/0.12)] sm:h-24 sm:w-24 sm:rounded-[22px] lg:h-30 lg:w-30 lg:rounded-[26px]"
                        aria-hidden="true"
                    >
                        {avatarErrored ? (
                            <span className="flex size-full items-center justify-center bg-muted">{profile.nickname?.slice(0, 1)}</span>
                        ) : (
                            <img
                                alt={profile.nickname ?? profile.uid}
                                src={avatarSrc}
                                onLoad={(e) => {
                                    if (e.currentTarget.naturalWidth === 0) setAvatarErrored(true);
                                }}
                                onError={() => setAvatarErrored(true)}
                                className="size-full object-cover"
                            />
                        )}
                    </div>
                </div>

                {/* Action buttons - shown top-right on mobile via grid placement */}
                <div className="flex items-center justify-end gap-2 self-start sm:order-3 sm:pt-1">
                    <Button variant="outline" size="sm" onClick={handleShare} className="gap-1.5">
                        <Share2 className="size-3.5" />
                        <span className="hidden xs:inline sm:inline">Share</span>
                    </Button>
                    <Tooltip>
                        <TooltipTrigger
                            render={(triggerProps) => (
                                <Button {...triggerProps} variant="outline" size="icon" onClick={handleCopyUid} aria-label="Copy UID" className="size-8">
                                    <Copy className="size-3.5" />
                                </Button>
                            )}
                        />
                        <TooltipPopup>Copy UID</TooltipPopup>
                    </Tooltip>
                </div>

                {/* Main info: spans full width on mobile, normal column on sm+ */}
                <div className="col-span-2 flex min-w-0 flex-col sm:col-span-1 sm:order-2">
                    <div className="mb-1.5 flex flex-wrap items-center gap-2">
                        <span className="font-mono text-[11px] font-medium uppercase leading-none tracking-widest text-muted-foreground">{profile.uid}</span>
                        <span className="inline-flex items-center gap-1 rounded-full border border-[oklch(0.28_0.005_285)] bg-muted px-2 py-0.5 font-mono text-[11px] font-medium leading-none text-muted-foreground">{profile.server} server</span>
                    </div>
                    <h1 className="mb-1 flex flex-wrap items-baseline gap-x-1.5 wrap-break-word font-sans text-2xl font-bold leading-[1.1] tracking-tight sm:text-3xl lg:text-[36px] lg:leading-[1.05]">
                        <Tooltip>
                            <TooltipTrigger
                                render={(triggerProps) => (
                                    <button {...triggerProps} type="button" onClick={handleCopyUsername} aria-label="Copy username" className="cursor-pointer rounded-sm text-left transition-colors hover:text-primary focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring">
                                        {displayNickname}
                                        {profile.nick_number && <span className="ml-1 font-mono font-medium text-muted-foreground text-base sm:text-lg lg:text-xl">#{profile.nick_number}</span>}
                                    </button>
                                )}
                            />
                            <TooltipPopup>Copy username</TooltipPopup>
                        </Tooltip>
                    </h1>
                    {profile.resume && <p className="mb-3.5 max-w-prose font-sans text-sm font-normal leading-normal text-muted-foreground sm:text-[14.5px] lg:max-w-135">{profile.resume}</p>}
                    {levelProgress && <LevelProgressBar progress={levelProgress} />}
                    <div className="mt-1.5 inline-flex items-center gap-2 font-sans text-xs font-medium leading-none text-muted-foreground">
                        <span className={shared.dotPulse} aria-hidden="true" />
                        <span>
                            Registered ·{" "}
                            <b>
                                {new Date((profile.register_ts ?? 0) * 1000).toLocaleDateString("en-US", {
                                    month: "short",
                                    day: "numeric",
                                    year: "numeric",
                                })}
                            </b>
                        </span>
                    </div>
                </div>
            </div>
        </section>
    );
}

function LevelProgressBar({ progress }: { progress: ILevelProgress }) {
    const { level, isMax, currentExp, requiredExp, ratio } = progress;
    const percent = Math.round(ratio * 100);
    const ariaLabel = isMax ? `Level ${level} (max)` : `Level ${level}, ${currentExp.toLocaleString()} of ${requiredExp?.toLocaleString()} EXP to level ${level + 1}`;

    return (
        <div className="mb-3.5 w-full max-w-prose lg:max-w-135">
            <div className="mb-1.5 flex flex-wrap items-baseline justify-between gap-x-3 gap-y-1 font-mono text-[11px] font-medium uppercase leading-none tracking-widest text-muted-foreground">
                <span>
                    Lv <span className="tabular-nums text-foreground">{level}</span>
                    {!isMax && (
                        <>
                            {"/"}
                            <span className="tabular-nums text-foreground">120</span>
                        </>
                    )}
                </span>
                <span className="tabular-nums">{isMax ? `Max · ${MAX_PLAYER_LEVEL}` : `${currentExp.toLocaleString()} / ${requiredExp?.toLocaleString()} EXP`}</span>
            </div>
            <div role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} aria-label={ariaLabel} className="h-1.5 overflow-hidden rounded-full bg-muted">
                <div className="h-full rounded-full bg-linear-to-r from-primary/70 to-primary transition-[width] duration-500 ease-out" style={{ width: `${percent}%` }} />
            </div>
        </div>
    );
}
