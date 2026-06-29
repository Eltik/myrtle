import { Link } from "@tanstack/react-router";
import { Package } from "lucide-react";
import { useState } from "react";
import { RARITY_HEX_MUTED } from "#/lib/utils";
import { SectionHead } from "./primitives";
import type { IDropGroup, IResolvedDrop } from "./types";

function RateMeter({ occ }: { occ: { label: string; level: number; tone: string } }) {
    return (
        <span className="inline-flex items-center gap-1.5" title={`Drop rate: ${occ.label}`}>
            <span aria-hidden="true" className="flex items-center gap-0.5">
                {[0, 1, 2, 3, 4].map((i) => (
                    <span key={i} className="h-2.5 w-1 rounded-[1px]" style={{ background: i < occ.level ? occ.tone : "color-mix(in oklch, var(--muted-foreground) 28%, transparent)" }} />
                ))}
            </span>
            <span className="font-medium font-mono text-[10px] uppercase leading-none tracking-[0.08em]" style={{ color: occ.tone }}>
                {occ.label}
            </span>
        </span>
    );
}

function DropCard({ drop }: { drop: IResolvedDrop }) {
    const [errored, setErrored] = useState(false);
    const color = RARITY_HEX_MUTED[drop.rarity] ?? "#b5b5b5";
    const initials =
        drop.name
            .split(/[\s-]+/)
            .filter(Boolean)
            .slice(0, 2)
            .map((w) => w[0])
            .join("")
            .toUpperCase() || "?";

    const inner = (
        <div className="group flex h-full items-center gap-3 rounded-[10px] border border-border bg-card p-2.5 transition-colors hover:border-[color-mix(in_oklch,var(--primary)_50%,transparent)]">
            <span
                className="relative flex h-11 w-11 shrink-0 items-center justify-center overflow-hidden rounded-md border"
                style={{
                    borderColor: `color-mix(in oklch, ${color} 45%, var(--border))`,
                    background: `radial-gradient(ellipse at 30% 30%, ${color}33, transparent 70%), color-mix(in oklch, var(--muted) 40%, transparent)`,
                }}
            >
                {!errored ? (
                    <img src={drop.iconUrl} alt={drop.name} loading="lazy" decoding="async" className="h-full w-full object-contain" onError={() => setErrored(true)} />
                ) : (
                    <span className="font-bold font-mono text-[12px]" style={{ color }}>
                        {initials}
                    </span>
                )}
            </span>
            <div className="flex min-w-0 flex-1 flex-col gap-1.5">
                <span className="truncate font-sans font-semibold text-[12.5px] text-foreground leading-tight">{drop.name}</span>
                <RateMeter occ={drop.occ} />
            </div>
        </div>
    );

    return drop.isChar ? (
        <Link to="/operators/$id" params={{ id: drop.reward.id }} className="block">
            {inner}
        </Link>
    ) : (
        inner
    );
}

export function DropsSection({ groups }: { groups: IDropGroup[] }) {
    if (groups.length === 0) return null;
    const total = groups.reduce((sum, g) => sum + g.drops.length, 0);
    return (
        <section className="flex flex-col gap-4">
            <SectionHead
                aside={
                    <span className="inline-flex items-center gap-1.5 font-medium font-mono text-[10px] text-muted-foreground">
                        <Package className="h-3 w-3" /> {total}
                    </span>
                }
            >
                Drops
            </SectionHead>
            {groups.map((group) => (
                <div key={group.type} className="flex flex-col gap-2">
                    <span className="font-medium font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{group.label}</span>
                    <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 xl:grid-cols-3">
                        {group.drops.map((drop) => (
                            <DropCard key={`${group.type}-${drop.reward.id}`} drop={drop} />
                        ))}
                    </div>
                </div>
            ))}
        </section>
    );
}
