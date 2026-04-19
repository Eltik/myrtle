import { useState } from "react";
import { Kicker } from "#/components/ui/kicker";
import { TIER_LISTS, type Operator } from "./data";
import TierListCard from "./TierListCard";

export default function TierLists({ onSelect }: { onSelect: (op: Operator) => void }) {
    const [filter, setFilter] = useState("All");
    const filters = ["All", "Endgame", "Beginner", "Roguelike", "Niche"];
    const list = TIER_LISTS.filter((t) => filter === "All" || t.tag === filter);

    return (
        <section className="operators page-wrap">
            <div className="section-head">
                <div className="section-head-row">
                    <div>
                        <Kicker>Community</Kicker>
                        <h2 className="section-title">Tier lists, at a glance.</h2>
                        <p className="section-lead">Previews from the 24 most-watched lists. Click any card to open the full ranking.</p>
                    </div>
                    <div className="op-filters" role="tablist">
                        {filters.map((f) => (
                            <button key={f} role="tab" type="button" className={`op-chip ${filter === f ? "is-active" : ""}`} onClick={() => setFilter(f)}>
                                {f}
                            </button>
                        ))}
                    </div>
                </div>
            </div>

            <div className="tl-grid">
                {list.map((tl) => (
                    <TierListCard key={tl.id} tl={tl} onSelect={onSelect} />
                ))}
            </div>

            <a className="tl-browse" href="#">
                <span className="tl-browse-dot" aria-hidden="true" />
                <span className="tl-browse-text">Browse all 24 tier lists</span>
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M5 12h14" />
                    <path d="m12 5 7 7-7 7" />
                </svg>
            </a>
        </section>
    );
}
