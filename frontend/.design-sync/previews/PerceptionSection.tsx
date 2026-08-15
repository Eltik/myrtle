import type { CSSProperties, ReactNode } from "react";
import { PerceptionSection } from "frontend";

const ACCENT = { "--imp-accent": "oklch(0.70 0.16 145)" } as CSSProperties;

const op = (operator_id: string, name: string) => ({ operator_id, name });

/** The collapsible body the panel drops this section into. */
const Section = ({ title, count, children }: { title: string; count: string; children: ReactNode }) => (
    <div className="flex max-w-2xl flex-col gap-3" style={ACCENT}>
        <div className="flex items-center justify-between gap-2 border-border/40 border-b pb-1.5">
            <span className="font-mono font-semibold text-[10.5px] uppercase tracking-[0.12em]" style={{ color: "color-mix(in oklch, var(--imp-accent) 60%, var(--foreground))" }}>
                {title}
            </span>
            <span className="rounded-md border border-border/40 bg-muted/30 px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground tabular-nums">{count}</span>
        </div>
        <div className="flex flex-col gap-3 pt-0.5">{children}</div>
    </div>
);

const SUPPORT = [
    { operator: op("char_391_rosmon", "Rosmontis"), room_type: "HIRE" },
    { operator: op("char_4055_bgsnow", "Pozëmka"), room_type: "DORMITORY" },
    { operator: op("char_2013_cerber", "Ceobe"), room_type: "MEETING" },
];

const CONSUMERS = [
    { operator: op("char_2023_ling", "Ling"), room_type: "MANUFACTURE", bonus_pct: 62, sustained_pct: 41 },
    { operator: op("char_2015_dusk", "Dusk"), room_type: "MANUFACTURE", bonus_pct: 55, sustained_pct: 37 },
    { operator: op("char_1028_texas2", "Texas the Omertosa"), room_type: "TRADING", bonus_pct: 38, sustained_pct: 26 },
    { operator: op("char_4064_mlynar", "Mlynar"), room_type: "TRADING", bonus_pct: 30, sustained_pct: 20 },
];

/** The full plan: support crew stationed outside production, the operators they power,
 *  and the morale-swap manager keeping the Ling/Dusk rotation alive. */
export const ResourceEconomy = () => (
    <Section title="Resource economy (max ceiling)" count="+62% peak">
        <PerceptionSection
            plan={{
                support: SUPPORT,
                consumers: CONSUMERS,
                rotation_manager: op("char_300_phenxi", "Fiammetta"),
                needs_rotation_manager: false,
            }}
        />
    </Section>
);

/** The roster can field the plan but has no morale-swap manager — the figures above
 *  are a peak ceiling rather than something sustainable. */
export const NeedsRotationManager = () => (
    <Section title="Resource economy (max ceiling)" count="+62% peak">
        <PerceptionSection
            plan={{
                support: SUPPORT,
                consumers: CONSUMERS,
                rotation_manager: null,
                needs_rotation_manager: true,
            }}
        />
    </Section>
);

/** No support operators owned yet: the powered half of the plan renders alone. */
export const ConsumersOnly = () => (
    <Section title="Resource economy (max ceiling)" count="+38% peak">
        <PerceptionSection
            plan={{
                support: [],
                consumers: [
                    { operator: op("char_1028_texas2", "Texas the Omertosa"), room_type: "TRADING", bonus_pct: 38, sustained_pct: 26 },
                    { operator: op("char_4064_mlynar", "Mlynar"), room_type: "TRADING", bonus_pct: 30, sustained_pct: 20 },
                ],
                rotation_manager: null,
                needs_rotation_manager: false,
            }}
        />
    </Section>
);
