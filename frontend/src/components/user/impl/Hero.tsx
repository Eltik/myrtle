import { MoreHorizontal, Share2 } from "lucide-react";
import { useState } from "react";
import { Button } from "#/components/ui/button";
import { toastManager } from "#/components/ui/toast";
import { getAvatarSkinId } from "#/lib/utils";
import type { IUserProfile } from "#/types/user";
import shared from "./shared.module.css";

interface IHeroProps {
    profile: IUserProfile;
}

export function Hero({ profile }: IHeroProps) {
    const [avatarErrored, setAvatarErrored] = useState(false);

    const handleShare = async () => {
        const url = window.location.href;
        try {
            await navigator.clipboard.writeText(url);
            toastManager.add({
                id: "share-success",
                title: "Link copied",
                description: "Profile link copied to clipboard.",
                type: "success",
            });
        } catch {
            toastManager.add({
                id: "share-error",
                title: "Couldn't copy link",
                description: "Clipboard access was denied.",
                type: "error",
            });
        }
    };

    return (
        <section className="relative overflow-hidden rounded-3xl border border-[oklch(0.28_0.005_285)] bg-card shadow-sm">
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
            <div className="relative grid grid-cols-[auto_1fr_auto] items-center gap-7 px-8 pt-8 pb-7">
                <div className="relative">
                    <div className="relative flex h-30 w-30 items-center justify-center overflow-hidden rounded-[26px] shadow-[inset_0_1px_0_rgb(255_255_255/0.45),inset_0_-2px_0_rgb(0_0_0/0.18),0_0_0_1px_rgb(255_255_255/0.06),0_8px_24px_rgb(0_0_0/0.12)]" aria-hidden="true">
                        {avatarErrored ? (
                            <span className="flex size-full items-center justify-center bg-muted">{profile.nickname?.slice(0, 1)}</span>
                        ) : (
                            <img
                                alt={profile.nickname ?? profile.uid}
                                src={getAvatarSkinId(profile)}
                                onLoad={(e) => {
                                    if (e.currentTarget.naturalWidth === 0) setAvatarErrored(true);
                                }}
                                onError={() => setAvatarErrored(true)}
                                className="size-full object-cover"
                            />
                        )}
                    </div>
                </div>
                <div className="flex min-w-0 flex-col">
                    <div className="mb-1.5 flex flex-wrap items-center gap-2.5">
                        <span className="font-mono text-[11px] font-medium uppercase leading-none tracking-widest text-muted-foreground">{profile.uid}</span>
                        <span className="inline-flex items-center gap-1.25 rounded-full border border-[oklch(0.28_0.005_285)] bg-muted px-2.25 py-0.75 font-mono text-[11px] font-medium leading-none text-muted-foreground">{profile.server} server</span>
                    </div>

                    <h1 className="mb-1 font-sans text-[36px] font-bold leading-[1.05] tracking-tight">{profile.nickname ?? `Doctor ${profile.uid}`}</h1>

                    {profile.resume && <p className="mb-3.5 max-w-135 font-sans text-[14.5px] font-normal leading-normal text-muted-foreground">{profile.resume}</p>}

                    <div className="mt-1.5 inline-flex items-center gap-2 font-sans text-xs font-medium leading-none text-muted-foreground">
                        <span className={shared.dotPulse} aria-hidden="true" />
                        <span>
                            Registered · <b>{new Date((profile.register_ts ?? 0) * 1000).toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" })}</b>
                        </span>
                    </div>
                </div>
                <div className="flex items-center gap-2 self-start pt-1">
                    <Button variant="outline" size="sm" onClick={handleShare} className="gap-1.5">
                        <Share2 className="size-3.5" />
                        Share
                    </Button>
                    <Button variant="ghost" size="icon" className="size-8" aria-label="More options">
                        <MoreHorizontal className="size-3.5" />
                    </Button>
                </div>
            </div>
        </section>
    );
}
