import { Link } from "@tanstack/react-router";
import { Avatar, AvatarFallback, AvatarImage } from "#/components/ui/avatar";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { ROLE_CHIP_GRADIENT, ROLE_SOLID, type Role, tlAccentVars } from "#/lib/role-styles";
import { formatNumberCompact, getAvatarById } from "#/lib/utils";
import type { ITierList } from "./data";
import styles from "./TierListCard.module.css";

const MAX_TOP_OPS = 4;
const MAX_GHOST_OPS = 6;
const MAX_GHOST_TIER_NAMES = 3;

export default function TierListCard({ tl, onOpen }: { tl: ITierList; onOpen?: (slug: string) => void }) {
    const [topTier, ...restTiers] = tl.tiers;
    const topOps = (topTier?.operators ?? []).slice(0, MAX_TOP_OPS);
    const ghostOps = restTiers.flatMap((t) => t.operators).slice(0, MAX_GHOST_OPS);
    const totalOps = tl.tiers.reduce((n, t) => n + t.operators.length, 0);
    const restCount = totalOps - topOps.length;

    const ghostNames = restTiers.slice(0, MAX_GHOST_TIER_NAMES).map((t) => t.name);
    const extraGhost = Math.max(0, restTiers.length - MAX_GHOST_TIER_NAMES);
    const ghostLabel = ghostNames.length > 0 ? `+ ${ghostNames.join(", ")}${extraGhost > 0 ? ` +${extraGhost}` : ""} ${restTiers.length > 1 ? "tiers" : "tier"}` : null;

    return (
        <Link to="/tier-lists/$id" params={{ id: tl.slug }} aria-labelledby={`home-tl-${tl.id}-title`} className={`${styles.tlCard} group`} style={tlAccentVars(tl.accent)} onClick={() => onOpen?.(tl.slug)}>
            <div className="flex flex-col gap-1.5 px-4 pt-3.5 pb-2 pl-4.5">
                <div className="inline-flex w-max max-w-full items-center gap-1.5 rounded-full border border-border bg-muted px-2 py-0.75 pl-2 font-medium font-mono text-[10.5px] text-muted-foreground uppercase leading-none tracking-wider">
                    <span className="h-1.5 w-1.5 rounded-full bg-(--tl-a,var(--primary)) shadow-[0_0_6px_var(--tl-a,var(--primary))]" aria-hidden="true" />
                    <span>{tl.tag}</span>
                    {tl.hot && <span className="ml-0.5 font-semibold text-primary normal-case tracking-tight">· trending</span>}
                </div>
                <h3 id={`home-tl-${tl.id}-title`} className="m-0 line-clamp-2 font-sans font-semibold text-base text-foreground leading-snug tracking-tight">
                    {tl.title}
                </h3>
            </div>

            <div className="flex flex-col gap-2.5 px-4 pt-0.5 pb-3 pl-4.5">
                {topTier && (
                    <>
                        <div className="flex items-center justify-between gap-2">
                            <span className="inline-flex items-center rounded-[5px] bg-(--tl-a,var(--primary)) px-2 py-0.75 font-bold font-sans text-[10.5px] text-primary-foreground leading-none tracking-tight">{topTier.name}</span>
                            <span className="font-medium font-mono text-[11px] text-muted-foreground leading-none tracking-wide">
                                {topTier.operators.length} {topTier.operators.length === 1 ? "pick" : "picks"}
                            </span>
                        </div>
                        <div className="flex min-w-0 flex-wrap gap-1.5">
                            {topOps.map((op) => (
                                <span key={op.id} className="inline-flex items-center gap-1.5 rounded-full border border-border bg-muted py-0.75 pr-2.5 pl-0.75 font-medium font-sans text-[11.5px] text-foreground leading-none" title={`${op.name} · ${op.role}`}>
                                    <span className="inline-flex h-5 w-5 items-center justify-center rounded-full font-bold font-sans text-[9.5px] text-white leading-none tracking-tight" style={{ background: ROLE_CHIP_GRADIENT[op.role as Role] }}>
                                        <OperatorAvatar charId={op.id} name={op.name} />
                                    </span>
                                    <span>{op.name}</span>
                                </span>
                            ))}
                        </div>
                    </>
                )}

                {ghostLabel && (
                    <div className="flex items-center gap-2.5 pt-0.5">
                        <span className="font-mono text-[11px] text-muted-foreground leading-none tracking-wide">{ghostLabel}</span>
                        <div className="flex items-center">
                            {ghostOps.map((op) => (
                                <span key={`ghost-${op.id}`} className="-ml-1 h-3.5 w-3.5 rounded-full border-[1.5px] border-card opacity-60 first:ml-0" style={{ background: ROLE_SOLID[op.role as Role] }} title={op.name}>
                                    <OperatorAvatar charId={op.id} name={op.name} />
                                </span>
                            ))}
                            {restCount > MAX_GHOST_OPS && <span className="ml-1.5 font-medium font-mono text-[10.5px] text-muted-foreground leading-none tracking-tight">+{restCount - MAX_GHOST_OPS}</span>}
                        </div>
                    </div>
                )}
            </div>

            <div className="mt-auto flex items-center justify-between gap-2.5 border-border border-t bg-muted/30 px-4 pt-2.5 pb-3 pl-4.5">
                <div className="flex min-w-0 items-center gap-2">
                    <Avatar className="h-5.5 w-5.5 shrink-0 border border-border bg-linear-to-br from-muted to-border font-sans font-semibold text-[9.5px] text-foreground tracking-tight">
                        {tl.author.avatarId && <AvatarImage src={getAvatarById(tl.author.avatarId)} alt={tl.author.name} />}
                        <AvatarFallback>{tl.author.name.charAt(0).toUpperCase()}</AvatarFallback>
                    </Avatar>
                    <span className="inline-flex min-w-0 items-center gap-1.5 overflow-hidden text-ellipsis whitespace-nowrap font-sans text-[11.5px] text-muted-foreground leading-none">
                        <span className="font-medium text-foreground">{tl.author.name}</span>
                        <span className="opacity-50">·</span>
                        <span className="font-mono text-[11px] tracking-tight">{tl.updated}</span>
                    </span>
                </div>
                <div className="inline-flex shrink-0 items-center gap-2.5">
                    <span className="inline-flex items-center gap-1 font-medium font-mono text-[11.5px] text-muted-foreground leading-none tracking-tight [&>svg]:h-3 [&>svg]:w-3 [&>svg]:opacity-80" title={`${tl.views} views`}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Views">
                            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                        {formatNumberCompact(tl.views)}
                    </span>
                    <span className="inline-flex items-center gap-1 font-medium font-mono text-[11.5px] text-muted-foreground leading-none tracking-tight [&>svg]:h-2.75 [&>svg]:w-2.75 [&>svg]:opacity-80" title={`${tl.votes} favorites`}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Votes">
                            <path d="m5 15 7-7 7 7" />
                        </svg>
                        {formatNumberCompact(tl.votes)}
                    </span>
                    <span className="inline-flex shrink-0 items-center gap-1 whitespace-nowrap font-medium font-sans text-foreground text-xs leading-none tracking-tight transition-[color,gap] group-hover:text-primary [&>svg]:h-3.25 [&>svg]:w-3.25 [&>svg]:transition-transform group-hover:[&>svg]:translate-x-0.75">
                        Open
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" role="image" aria-label="Right arrow">
                            <path d="M5 12h14" />
                            <path d="m12 5 7 7-7 7" />
                        </svg>
                    </span>
                </div>
            </div>
        </Link>
    );
}
