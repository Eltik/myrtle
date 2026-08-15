import { DetailSectionHead } from "frontend";

// The kicker + hairline rule that opens every block on /enemies/$id. Ported
// straight from OverviewTab / DetailStatsTab / AppearsInTab, which are its only
// callers — the head always sits directly above its section body.
export const OverSectionBody = () => (
    <section className="w-full max-w-2xl">
        <DetailSectionHead>Description</DetailSectionHead>
        <p className="m-0 text-pretty font-sans text-[13.5px] text-foreground leading-relaxed">One of Reunion's squad leaders who serves in the Assault Squad. Though armed with an Originium launcher and explosives, reports show that he possesses high mobility and poses a significant threat even in melee combat.</p>
    </section>
);

export const StackedSections = () => (
    <div className="flex w-full max-w-2xl flex-col gap-5">
        <section>
            <DetailSectionHead>Combat · 3 levels</DetailSectionHead>
            <div className="grid grid-cols-3 gap-3">
                {[
                    { level: "L0", hp: "10,500" },
                    { level: "L1", hp: "30,000" },
                    { level: "L2", hp: "80,000" },
                ].map((p) => (
                    <div key={p.level} className="rounded-lg border border-border bg-muted/20 p-3">
                        <div className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Max HP · {p.level}</div>
                        <div className="mt-1.5 font-mono font-semibold text-[15px] text-foreground tabular-nums leading-none">{p.hp}</div>
                    </div>
                ))}
            </div>
        </section>
        <section>
            <DetailSectionHead>Tags</DetailSectionHead>
            <div className="flex flex-wrap gap-1.5">
                {["sarkaz", "boss", "reunion"].map((t) => (
                    <span key={t} className="rounded-full border border-border px-2.5 py-1 font-medium font-sans text-[11px] text-muted-foreground leading-none">
                        {t}
                    </span>
                ))}
            </div>
        </section>
    </div>
);

export const WithCount = () => (
    <section className="w-full max-w-2xl">
        <DetailSectionHead>Story Stages · 4</DetailSectionHead>
        <div className="flex flex-wrap gap-1.5">
            {["1-7", "4-4", "7-18", "10-16"].map((code) => (
                <span key={code} className="inline-flex items-center rounded-md border border-border bg-card px-2 py-1 font-medium font-mono text-[11.5px] text-foreground tabular-nums leading-none">
                    {code}
                </span>
            ))}
        </div>
    </section>
);
