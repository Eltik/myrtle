import { Clock, Coins, Flag, Gauge, Heart, Shield, Skull, Sparkles, Star, Timer, Zap } from "lucide-react";
import type { ILevel } from "#/lib/api/level";
import type { IStage } from "#/types/stages";
import { SectionHead, StatCard } from "./primitives";

export function OverviewSection({ stage, level }: { stage: IStage; level: ILevel | null }) {
    const opts = level?.options;
    return (
        <section>
            <SectionHead>Overview</SectionHead>
            <div className="grid grid-cols-2 gap-2.5 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-6">
                <StatCard icon={<Zap />} label="Sanity Cost" value={String(stage.apCost)} accent="var(--primary)" />
                <StatCard icon={<Sparkles />} label="EXP Gain" value={stage.expGain.toLocaleString()} accent="var(--info)" />
                <StatCard icon={<Coins />} label="LMD Gain" value={stage.goldGain.toLocaleString()} accent="var(--warning)" />
                {stage.dangerLevel && <StatCard icon={<Skull />} label="Risk Level" value={stage.dangerLevel} accent="var(--destructive)" />}
                {stage.dangerPoint > 0 && <StatCard icon={<Flag />} label="Danger Point" value={String(stage.dangerPoint)} accent="var(--destructive)" />}
                {opts?.maxLifePoint != null && <StatCard icon={<Heart />} label="Life Points" value={String(opts.maxLifePoint)} accent="var(--destructive)" />}
                {opts?.characterLimit != null && <StatCard icon={<Shield />} label="Unit Limit" value={String(opts.characterLimit)} accent="var(--primary)" />}
                {opts?.initialCost != null && <StatCard icon={<Star />} label="Initial DP" value={String(opts.initialCost)} accent="var(--info)" />}
                {opts?.maxCost != null && <StatCard icon={<Star />} label="Max DP" value={String(opts.maxCost)} accent="var(--info)" />}
                {opts?.costIncreaseTime != null && <StatCard icon={<Timer />} label="DP / Tick" value={`${opts.costIncreaseTime}s`} accent="var(--muted-foreground)" />}
                {opts?.moveMultiplier != null && opts.moveMultiplier !== 1 && <StatCard icon={<Gauge />} label="Move Speed" value={`×${opts.moveMultiplier}`} accent="var(--warning)" />}
                {opts?.maxPlayTime != null && opts.maxPlayTime > 0 && <StatCard icon={<Clock />} label="Time Limit" value={`${opts.maxPlayTime}s`} accent="var(--warning)" />}
            </div>
        </section>
    );
}
