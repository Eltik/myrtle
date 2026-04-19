import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { ROLE_CHIP_GRADIENT, ROLE_SOLID, tlAccentVars, type Role } from "#/lib/role-styles";
import { OP_BY_ID, type Operator, type TierList } from "./data";

export default function TierListCard({ tl, onSelect }: { tl: TierList; onSelect: (op: Operator) => void }) {
    const topOps = (tl.tiers.S || [])
        .slice(0, 4)
        .map((id) => OP_BY_ID[id])
        .filter(Boolean);
    const ghostOps = [...(tl.tiers.A || []), ...(tl.tiers.B || [])]
        .slice(0, 6)
        .map((id) => OP_BY_ID[id])
        .filter(Boolean);
    const totalOps = Object.values(tl.tiers).reduce((n, arr) => n + arr.length, 0);
    const restCount = totalOps - topOps.length;

    return (
        <article className="tl-card group" role="link" tabIndex={0} style={tlAccentVars(tl.accent)}>
            <div className="flex flex-col gap-1.5 px-4 pt-3.5 pb-2 pl-4.5">
                <div className="inline-flex w-max max-w-full items-center gap-1.5 rounded-full border border-border bg-muted px-2 py-[3px] pl-2 font-mono text-[10.5px] font-medium uppercase leading-none tracking-wider text-muted-foreground">
                    <span className="h-1.5 w-1.5 rounded-full bg-[var(--tl-a,var(--primary))] shadow-[0_0_6px_var(--tl-a,var(--primary))]" aria-hidden="true" />
                    <span>{tl.tag}</span>
                    {tl.hot && <span className="ml-0.5 font-semibold normal-case tracking-tight text-primary">· trending</span>}
                </div>
                <h3 className="m-0 line-clamp-2 font-sans text-base font-semibold leading-snug tracking-tight text-foreground">{tl.title}</h3>
            </div>

            <div className="flex flex-col gap-2.5 px-4 pt-0.5 pb-3 pl-4.5">
                <div className="flex items-center justify-between gap-2">
                    <span className="inline-flex items-center rounded-[5px] bg-[var(--tl-a,var(--primary))] px-2 py-[3px] font-sans text-[10.5px] font-bold leading-none tracking-tight text-primary-foreground">S-tier</span>
                    <span className="font-mono text-[11px] font-medium leading-none tracking-wide text-muted-foreground">{tl.tiers.S.length} picks</span>
                </div>
                <div className="flex min-w-0 flex-wrap gap-1.5">
                    {topOps.map((op) => (
                        <button
                            key={op.id}
                            type="button"
                            className="inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-border bg-muted py-[3px] pl-[3px] pr-2.5 font-sans text-[11.5px] font-medium leading-none text-foreground transition-[background-color,border-color,transform] hover:-translate-y-px hover:border-[color-mix(in_srgb,var(--primary)_22%,var(--border))]"
                            onClick={(e) => {
                                e.stopPropagation();
                                onSelect({ ...op, e: 2, lvl: 90, hp: 2020, atk: 510, def: 225, trust: 200, owned: true });
                            }}
                            title={`${op.name} · ${op.role}`}
                        >
                            <span className="inline-flex h-5 w-5 items-center justify-center rounded-full font-sans text-[9.5px] font-bold leading-none tracking-tight text-white" style={{ background: ROLE_CHIP_GRADIENT[op.role as Role] }}>
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span>{op.name}</span>
                        </button>
                    ))}
                </div>

                <div className="flex items-center gap-2.5 pt-0.5">
                    <span className="font-mono text-[11px] leading-none tracking-wide text-muted-foreground">+ A, B tiers</span>
                    <div className="flex items-center">
                        {ghostOps.map((op, i) => (
                            <span key={op.id + i} className="-ml-1 h-3.5 w-3.5 rounded-full border-[1.5px] border-card opacity-60 first:ml-0" style={{ background: ROLE_SOLID[op.role as Role] }} title={op.name}>
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                        ))}
                        {restCount > 4 && <span className="ml-1.5 font-mono text-[10.5px] font-medium leading-none tracking-tight text-muted-foreground">+{restCount - 4}</span>}
                    </div>
                </div>
            </div>

            <div className="mt-auto flex items-center justify-between gap-2.5 border-t border-border bg-muted/30 px-4 pt-2.5 pb-3 pl-4.5">
                <div className="flex min-w-0 items-center gap-2">
                    <span className="inline-flex h-5.5 w-5.5 shrink-0 items-center justify-center rounded-full border border-border bg-gradient-to-br from-muted to-border font-sans text-[9.5px] font-semibold leading-none tracking-tight text-foreground">{tl.author.initials}</span>
                    <span className="inline-flex min-w-0 items-center gap-1.5 overflow-hidden text-ellipsis whitespace-nowrap font-sans text-[11.5px] leading-none text-muted-foreground">
                        <span className="font-medium text-foreground">{tl.author.name}</span>
                        <span className="opacity-50">·</span>
                        <span className="font-mono text-[11px] tracking-tight">{tl.updated}</span>
                    </span>
                </div>
                <div className="inline-flex shrink-0 items-center gap-2.5">
                    <span className="inline-flex items-center gap-1 font-mono text-[11.5px] font-medium leading-none tracking-tight text-muted-foreground [&>svg]:h-[11px] [&>svg]:w-[11px] [&>svg]:opacity-80" title={`${tl.votes} upvotes`}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="m5 15 7-7 7 7" />
                        </svg>
                        {tl.votes > 999 ? (tl.votes / 1000).toFixed(1) + "k" : tl.votes}
                    </span>
                    <span className="inline-flex items-center gap-1 font-sans text-xs font-medium leading-none tracking-tight text-foreground transition-[color,gap] group-hover:text-primary [&>svg]:h-[13px] [&>svg]:w-[13px] [&>svg]:transition-transform group-hover:[&>svg]:translate-x-[3px]">
                        Open
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M5 12h14" />
                            <path d="m12 5 7 7-7 7" />
                        </svg>
                    </span>
                </div>
            </div>
        </article>
    );
}
