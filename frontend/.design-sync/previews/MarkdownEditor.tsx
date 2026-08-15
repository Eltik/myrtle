import { MarkdownEditor } from "frontend";

const noop = () => {
    /* previews render a fixed value; editing is an interaction state */
};

const DESCRIPTION = `A **generalist clear list** for Chapter 8 — Roaring Flare.

- \`S\` tier operators clear every stage without a second DPS
- Placements assume **E2 lv 60** with module X unlocked
- See [the full methodology](https://myrtle.moe/tier-lists) for how votes are weighted`;

export const TierListDescription = () => (
    <div className="flex w-full max-w-lg flex-col gap-1.5">
        <label className="font-medium font-sans text-foreground text-sm" htmlFor="tier-list-description">
            List description
        </label>
        <MarkdownEditor id="tier-list-description" maxLength={600} onChange={noop} placeholder="Add a short description so viewers know what this list is about." rows={4} value={DESCRIPTION} />
    </div>
);

export const CompactPlacementNote = () => (
    <div className="flex w-full max-w-lg flex-col gap-1.5">
        <label className="font-medium font-sans text-foreground text-sm" htmlFor="placement-note">
            Why does Mlynar land here?
        </label>
        <MarkdownEditor id="placement-note" maxLength={280} onChange={noop} placeholder="Why does this operator land here?" rows={3} showHint={false} size="sm" value="" />
    </div>
);

export const PublishingDisabled = () => (
    <div className="flex w-full max-w-lg flex-col gap-1.5">
        <label className="font-medium font-sans text-foreground text-sm" htmlFor="changelog">
            Version changelog
        </label>
        <MarkdownEditor disabled id="changelog" maxLength={400} onChange={noop} rows={3} value={"Promoted Texas the Omertosa to **S**.\nAdded Wis'adel with a note on her S3 rotation."} />
        <span className="font-sans text-muted-foreground text-xs">Publishing version 12 — the changelog is locked while the request is in flight.</span>
    </div>
);

export const WithoutToolbar = () => (
    <div className="flex w-full max-w-lg flex-col gap-1.5">
        <label className="font-medium font-sans text-foreground text-sm" htmlFor="operator-notes">
            Operator notes
        </label>
        <MarkdownEditor hideToolbar id="operator-notes" onChange={noop} rows={3} showHint={false} size="sm" value={"Strong against Sarkaz-heavy waves; his S3 covers the left lane on 1-7 by himself.\n\nPairs well with Skadi for the ASPD boost."} />
    </div>
);
