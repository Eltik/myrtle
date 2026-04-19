import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { OP_BY_ID, ROLE_CLS, type Operator, type TierList } from "./data";

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
        <article className={`tl-card accent-${tl.accent}`} role="link" tabIndex={0}>
            <div className="tl-head">
                <div className="tl-tag">
                    <span className="tl-tag-dot" aria-hidden="true" />
                    <span>{tl.tag}</span>
                    {tl.hot && <span className="tl-hot-inline">· trending</span>}
                </div>
                <h3 className="tl-title">{tl.title}</h3>
            </div>

            <div className="tl-preview">
                <div className="tl-preview-head">
                    <span className="tl-s-pill">S-tier</span>
                    <span className="tl-s-count">{tl.tiers.S.length} picks</span>
                </div>
                <div className="tl-s-row">
                    {topOps.map((op) => (
                        <button
                            key={op.id}
                            type="button"
                            className={`tl-chip ${ROLE_CLS[op.role] || ""}`}
                            onClick={(e) => {
                                e.stopPropagation();
                                onSelect({ ...op, e: 2, lvl: 90, hp: 2020, atk: 510, def: 225, trust: 200, owned: true });
                            }}
                            title={`${op.name} · ${op.role}`}
                        >
                            <span className="tl-chip-initials">
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                            <span className="tl-chip-name">{op.name}</span>
                        </button>
                    ))}
                </div>

                <div className="tl-ghost">
                    <span className="tl-ghost-label">+ A, B tiers</span>
                    <div className="tl-ghost-stack">
                        {ghostOps.map((op, i) => (
                            <span key={op.id + i} className={`tl-ghost-dot ${ROLE_CLS[op.role] || ""}`} title={op.name}>
                                <OperatorAvatar charId={op.id} name={op.name} />
                            </span>
                        ))}
                        {restCount > 4 && <span className="tl-ghost-more">+{restCount - 4}</span>}
                    </div>
                </div>
            </div>

            <div className="tl-foot">
                <div className="tl-foot-meta">
                    <span className="tl-avatar">{tl.author.initials}</span>
                    <span className="tl-foot-text">
                        <span className="tl-author-name">{tl.author.name}</span>
                        <span className="tl-foot-sep">·</span>
                        <span className="tl-foot-time">{tl.updated}</span>
                    </span>
                </div>
                <div className="tl-foot-cta">
                    <span className="tl-votes" title={`${tl.votes} upvotes`}>
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="m5 15 7-7 7 7" />
                        </svg>
                        {tl.votes > 999 ? (tl.votes / 1000).toFixed(1) + "k" : tl.votes}
                    </span>
                    <span className="tl-cta">
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
