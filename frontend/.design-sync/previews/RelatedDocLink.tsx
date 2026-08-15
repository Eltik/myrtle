import { RelatedDocLink, RelatedLinksFooter } from "frontend";

// A primary-coloured cross-reference between the two legal documents. Its `to`
// is typed to `/privacy | /terms`, so the two stories below are the whole axis.
// It is only ever rendered inside `RelatedLinksFooter`, which supplies the
// wrapping flex row.

export const InFooter = () => (
    <div className="max-w-3xl">
        <RelatedLinksFooter>
            <RelatedDocLink to="/terms" label="Terms of Service" />
            <RelatedDocLink to="/privacy" label="Privacy Policy" />
        </RelatedLinksFooter>
    </div>
);

export const BothTargets = () => (
    <div className="flex flex-col gap-3 rounded-xl border border-border bg-card px-5 py-4">
        <p className="m-0 font-medium font-sans text-[14px] text-foreground">Related Documents</p>
        <div className="inline-flex flex-wrap gap-x-4 gap-y-1.5">
            <RelatedDocLink to="/privacy" label="Privacy Policy" />
            <RelatedDocLink to="/terms" label="Terms of Service" />
        </div>
    </div>
);

export const InlineInProse = () => (
    <p className="m-0 max-w-3xl font-sans text-[16px] text-foreground leading-[1.75]">
        Data you sync is handled as described in our <RelatedDocLink to="/privacy" label="Privacy Policy" />, and your use of the roster tools is additionally governed by the <RelatedDocLink to="/terms" label="Terms of Service" />.
    </p>
);
