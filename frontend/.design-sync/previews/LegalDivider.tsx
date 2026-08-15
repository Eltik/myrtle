import { LegalContainer, LegalDivider, SectionHeading } from "frontend";
import { Eye, Lock } from "lucide-react";

// A `Separator` with the document rhythm baked in (`my-10`). Both legal pages
// use it to break the long clause run into chapters — never on its own, so the
// preview is the two blocks it sits between.

const P = ({ children }: { children: React.ReactNode }) => <p className="m-0 mb-4 font-sans text-[16px] text-foreground leading-[1.75] last:mb-0">{children}</p>;

export const BetweenClauses = () => (
    <div className="max-w-3xl">
        <SectionHeading icon={<Lock className="size-5" strokeWidth={1.8} />} title="Account Security" />
        <P>Sessions are bound to a rotating token and expire after 30 days of inactivity. Revoking a session from Settings invalidates it immediately on every device.</P>
        <LegalDivider />
        <SectionHeading icon={<Eye className="size-5" strokeWidth={1.8} />} title="Profile Visibility" />
        <P>A private profile is excluded from search, the leaderboard and the community gacha statistics. Switching back to public re-indexes it on the next sync.</P>
    </div>
);

export const InDocumentFlow = () => (
    <LegalContainer>
        <h2 className="m-0 mb-4 font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em]">Acceptable Use</h2>
        <P>Scraping the public API for bulk redistribution, or automating account linking on behalf of other Doctors, is not permitted.</P>
        <LegalDivider />
        <h2 className="m-0 mb-4 font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em]">User Content</h2>
        <P>Tier lists you publish stay yours. Publishing one grants myrtle.moe a licence to display it on the site and in the community gallery.</P>
        <LegalDivider />
        <h2 className="m-0 mb-4 font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em]">Intellectual Property</h2>
        <P>Arknights, its operators and its artwork belong to Hypergryph and Yostar. myrtle.moe is an unofficial fan project.</P>
    </LegalContainer>
);
