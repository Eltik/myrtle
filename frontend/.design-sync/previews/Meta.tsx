import { Swords } from "lucide-react";
import { Meta, StagesDetailSectionHead } from "frontend";

/** PropertiesSection's identifier grid for 4-10 — Extinguished Flames. */
export const Identifiers = () => (
    <section className="max-w-lg">
        <StagesDetailSectionHead>Identifiers</StagesDetailSectionHead>
        <div className="grid grid-cols-2 gap-2">
            <Meta label="Stage ID" value="main_04-10" />
            <Meta label="Level ID" value="Obt/Main/level_main_04-10" />
            <Meta label="Zone ID" value="main_4" />
            <Meta label="Challenge ID" value="main_04-10#f#" />
        </div>
    </section>
);

/** WavesSection uses the same tile for the wave's timing numbers. */
export const WaveTimings = () => (
    <div className="max-w-md rounded-[10px] border border-border bg-card p-3.5">
        <div className="flex items-center justify-between gap-2">
            <span className="inline-flex items-center gap-2 font-medium font-mono text-[10.5px] text-primary uppercase leading-none tracking-[0.14em]">
                <Swords className="h-3.5 w-3.5" /> Wave 2
            </span>
            <span className="font-mono text-[11px] text-muted-foreground tabular-nums">62 spawns</span>
        </div>
        <div className="mt-3 grid grid-cols-3 gap-2">
            <Meta label="Pre-Delay" value="5.0s" />
            <Meta label="Post-Delay" value="0.0s" />
            <Meta label="Fragments" value="4" />
        </div>
    </div>
);

/** Long ids wrap rather than truncate — the whole value has to stay copyable. */
export const LongValue = () => (
    <div className="grid max-w-sm grid-cols-1 gap-2">
        <Meta label="Level ID" value="Obt/Rogue/Rogue_5/level_rogue5_b-4" />
        <Meta label="Zone ID" value="rogue_5_zone_bloodline" />
    </div>
);
