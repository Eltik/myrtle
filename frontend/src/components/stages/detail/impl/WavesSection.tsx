import { Swords } from "lucide-react";
import type { ILevel } from "#/lib/api/level";
import { Meta, SectionHead } from "./primitives";

export function WavesSection({ level }: { level: ILevel | null }) {
    const waves = level?.waves ?? [];
    if (waves.length === 0) return null;
    return (
        <section className="flex flex-col gap-3">
            <SectionHead>Waves · {waves.length}</SectionHead>
            {waves.map((wave, wi) => {
                const spawnCount = (wave.fragments ?? []).reduce((sum, frag) => sum + (frag.actions ?? []).reduce((s, a) => s + (a.actionType === "SPAWN" ? (a.count ?? 1) : 0), 0), 0);
                return (
                    // biome-ignore lint/suspicious/noArrayIndexKey: waves are positional ("Wave 1", "Wave 2") and never reorder, so index is their identity
                    <div key={`wave-${wi}`} className="rounded-[10px] border border-border bg-card p-3.5">
                        <div className="flex items-center justify-between gap-2">
                            <span className="inline-flex items-center gap-2 font-medium font-mono text-[10.5px] text-primary uppercase leading-none tracking-[0.14em]">
                                <Swords className="h-3.5 w-3.5" /> Wave {wi + 1}
                            </span>
                            <span className="font-mono text-[11px] text-muted-foreground tabular-nums">{spawnCount} spawns</span>
                        </div>
                        <div className="mt-3 grid grid-cols-3 gap-2">
                            <Meta label="Pre-Delay" value={`${wave.preDelay.toFixed(1)}s`} />
                            <Meta label="Post-Delay" value={`${wave.postDelay.toFixed(1)}s`} />
                            <Meta label="Fragments" value={String((wave.fragments ?? []).length)} />
                        </div>
                    </div>
                );
            })}
        </section>
    );
}
