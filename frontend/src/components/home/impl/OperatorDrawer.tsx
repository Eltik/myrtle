import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Kicker } from "#/components/ui/kicker";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { ROLE_CLS, type Operator } from "./data";

export default function OperatorDrawer({ op, onClose }: { op: Operator; onClose: () => void }) {
    return (
        <div className="drawer-backdrop" onClick={onClose}>
            <aside className="drawer" onClick={(e) => e.stopPropagation()}>
                <div className="drawer-head">
                    <Kicker>Operator</Kicker>
                    <button type="button" className="drawer-close" onClick={onClose} aria-label="Close">
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                            <path d="M18 6 6 18M6 6l12 12" />
                        </svg>
                    </button>
                </div>
                <div className={`drawer-hero ${ROLE_CLS[op.role] || ""}`}>
                    <span className="drawer-initials">
                        <OperatorAvatar charId={op.id} name={op.name} />
                    </span>
                    <span className="drawer-rarity">{"★".repeat(op.rarity)}</span>
                </div>
                <h2 className="drawer-title">{op.name}</h2>
                <div className="drawer-tags">
                    <Badge>{op.role}</Badge>
                    <Badge variant="secondary">{op.arch}</Badge>
                    {op.owned ? (
                        <Badge variant="success">
                            E{op.e} · Lv{op.lvl}
                        </Badge>
                    ) : (
                        <Badge variant="outline">Not owned</Badge>
                    )}
                </div>
                <div className="stat-grid">
                    <div>
                        <div className="stat-label">HP</div>
                        <div className="stat-val">2,020</div>
                    </div>
                    <div>
                        <div className="stat-label">ATK</div>
                        <div className="stat-val">510</div>
                    </div>
                    <div>
                        <div className="stat-label">DEF</div>
                        <div className="stat-val">225</div>
                    </div>
                    <div>
                        <div className="stat-label">RES</div>
                        <div className="stat-val">0</div>
                    </div>
                </div>
                <div className="drawer-actions">
                    <Button variant="default">View skills</Button>
                    <Button variant="outline">Add to team</Button>
                </div>
            </aside>
        </div>
    );
}
