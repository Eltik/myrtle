import { LegalContainer, RelatedDocLink, RelatedLinksFooter, SectionHeading } from "frontend";
import { Mail } from "lucide-react";

// The card that closes both legal documents: a "Related Documents" list of
// sibling policies on the left, a "Return Home" outline button on the right.
// Ported from the tail of `src/components/legal/PrivacyPage.tsx`.

const P = ({ children }: { children: React.ReactNode }) => <p className="m-0 mb-4 font-sans text-[16px] text-foreground leading-[1.75] last:mb-0">{children}</p>;

export const Default = () => (
    <div className="max-w-3xl">
        <RelatedLinksFooter>
            <RelatedDocLink to="/terms" label="Terms of Service" />
            <RelatedDocLink to="/privacy" label="Privacy Policy" />
        </RelatedLinksFooter>
    </div>
);

export const SingleLink = () => (
    <div className="max-w-3xl">
        <RelatedLinksFooter>
            <RelatedDocLink to="/terms" label="Terms of Service" />
        </RelatedLinksFooter>
    </div>
);

export const ClosingADocument = () => (
    <LegalContainer>
        <SectionHeading icon={<Mail className="size-5" strokeWidth={1.8} />} title="Contact" subtitle="Questions about this policy" />
        <P>Reach us at privacy@myrtle.moe for data-deletion requests, or open an issue on the repository for anything else. We answer deletion requests within 24 hours.</P>
        <RelatedLinksFooter>
            <RelatedDocLink to="/terms" label="Terms of Service" />
            <RelatedDocLink to="/privacy" label="Privacy Policy" />
        </RelatedLinksFooter>
    </LegalContainer>
);
