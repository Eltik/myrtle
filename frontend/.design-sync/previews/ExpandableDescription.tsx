import { ExpandableDescription } from "frontend";

const LONG_INTRO =
    "Ranked for CC#13 Fake Wave under Risk 18, assuming max potential is not available and modules sit at stage 2. The top of the list is dominated by operators who let you skip a whole risk category on their own — Wiš'adel deletes the Sarkaz Ritualist wave from off-screen, Młynar clears the left lane without a single healer, and Logos covers the drone rush that otherwise forces a second caster. Everything below A tier is a comfort pick: it clears, but it costs you deployment slots you would rather spend on the boss lane.";

const SHORT_NOTE = "Solid all-rounder for early Annihilation. Bring her if you are short on DP generation.";

const MARKDOWN_NOTE =
    "**S1 is the one you want** for Risk 18 — the burst window lines up with the second Ritualist spawn. Skip S3 unless you are running the *no-healer* variant, where the extra sustain matters more than the raw damage. Module `SNP-Y` at stage 2 is enough; stage 3 is a nice-to-have, not a requirement for this clear.";

const PLACEMENT_NOTE = "Placed here purely for the drone lane. Outside of that one wave she drops to B tier and Firewatch does the job for half the DP.";

export const Clamped = () => (
    <div className="max-w-160 font-sans text-muted-foreground text-sm leading-relaxed">
        <ExpandableDescription text={LONG_INTRO} />
    </div>
);

export const ShortDescription = () => (
    <div className="max-w-160 font-sans text-muted-foreground text-sm leading-relaxed">
        <ExpandableDescription text={SHORT_NOTE} />
    </div>
);

export const MarkdownBody = () => (
    <div className="max-w-160 font-sans text-muted-foreground text-sm leading-relaxed">
        <ExpandableDescription text={MARKDOWN_NOTE} markdown />
    </div>
);

export const TightPlacementNote = () => (
    <div className="max-w-100 rounded-lg border border-border bg-card p-3">
        <p className="m-0 mb-1.5 font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">Why Rosmontis is in A</p>
        <ExpandableDescription text={PLACEMENT_NOTE} markdown clampLines={2} threshold={140} className="font-sans text-[12.5px] text-muted-foreground leading-[1.55]" />
    </div>
);
