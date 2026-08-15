import { Package } from "lucide-react";
import { Meta, StagesDetailSectionHead } from "frontend";

export const Overview = () => (
    <section className="max-w-4xl">
        <StagesDetailSectionHead>Overview</StagesDetailSectionHead>
        <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-4">
            {[
                { label: "Initial DP", value: "10" },
                { label: "DP / Tick", value: "1s" },
                { label: "Unit Limit", value: "10" },
                { label: "Move Speed", value: "×0.5" },
            ].map((s) => (
                <div key={s.label} className="flex flex-col gap-1 rounded-[10px] border border-border bg-card p-3">
                    <span className="font-medium font-mono text-[10px] text-muted-foreground uppercase leading-none tracking-[0.12em]">{s.label}</span>
                    <span className="font-mono font-semibold text-[15px] text-foreground tabular-nums leading-none">{s.value}</span>
                </div>
            ))}
        </div>
    </section>
);

/** Section heads carry the item count inline — "Enemies · 7", "Waves · 5". */
export const WithCount = () => (
    <div className="flex max-w-2xl flex-col gap-6">
        <section>
            <StagesDetailSectionHead>Enemies · 7</StagesDetailSectionHead>
            <p className="m-0 font-sans text-[12.5px] text-muted-foreground">FrostNova, Butcher, Heavy Defender Leader, Soldier, Bladed Fighter, Caster Leader, Shielded Soldier.</p>
        </section>
        <section>
            <StagesDetailSectionHead>Identifiers</StagesDetailSectionHead>
            <div className="grid grid-cols-2 gap-2">
                <Meta label="Stage ID" value="main_04-10" />
                <Meta label="Zone ID" value="main_4" />
            </div>
        </section>
    </div>
);

/** DropsSection hangs a total in the `aside` slot, right of the rule. */
export const WithAside = () => (
    <section className="max-w-4xl">
        <StagesDetailSectionHead
            aside={
                <span className="inline-flex items-center gap-1.5 font-medium font-mono text-[10px] text-muted-foreground">
                    <Package className="h-3 w-3" /> 13
                </span>
            }
        >
            Drops
        </StagesDetailSectionHead>
        <div className="flex flex-col gap-2">
            <span className="font-medium font-mono text-[9.5px] text-muted-foreground uppercase leading-none tracking-[0.12em]">Regular Drops</span>
            <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
                {[
                    { name: "Integrated Device", icon: "MTL_SL_BOSS3" },
                    { name: "Optimized Device", icon: "MTL_SL_BOSS4" },
                    { name: "Orirock Cube", icon: "MTL_SL_G2" },
                ].map((item) => (
                    <div key={item.name} className="flex items-center gap-3 rounded-[10px] border border-border bg-card p-2.5">
                        <img src={`https://api.myrtle.moe/api/item-icon/${item.icon}`} alt="" className="h-11 w-11 rounded-md object-contain" />
                        <span className="truncate font-sans font-semibold text-[12.5px] text-foreground leading-tight">{item.name}</span>
                    </div>
                ))}
            </div>
        </div>
    </section>
);
