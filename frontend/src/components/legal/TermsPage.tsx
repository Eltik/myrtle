import { Link } from "@tanstack/react-router";
import { Calendar, FileText, ScaleIcon, ShieldIcon } from "lucide-react";
import { LegalContainer, LegalDivider, RelatedDocLink, RelatedLinksFooter } from "#/components/legal/LegalShell";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { Badge } from "#/components/ui/badge";

const EFFECTIVE_DATE = "January 12, 2026";
const LAST_UPDATED = "January 12, 2026";
const VERSION = "Version 2.0";
const REPO_URL = "https://github.com/Eltik/myrtle";
const FBS_URL = "https://github.com/MooncellWiki/OpenArknightsFBS";
const CONTACT_EMAIL = "contact@myrtle.moe";
const CONTACT_MAILTO = `mailto:${CONTACT_EMAIL}`;
const PRIVACY_EMAIL = "privacy@myrtle.moe";
const PRIVACY_MAILTO = `mailto:${PRIVACY_EMAIL}?subject=GDPR%20data%20request`;

const TOC = [
    { id: "acceptance", label: "Acceptance of Terms" },
    { id: "account", label: "Account Registration and Security" },
    { id: "usage", label: "Acceptable Use Policy" },
    { id: "content", label: "User Content and Submissions" },
    { id: "intellectual", label: "Intellectual Property Rights" },
    { id: "termination", label: "Termination and Suspension" },
    { id: "liability", label: "Limitation of Liability" },
    { id: "changes", label: "Changes to Terms" },
    { id: "contact", label: "Contact Information" },
];

function Section({ id, children }: { id: string; children: React.ReactNode }) {
    return (
        <section id={id} className="mb-10 scroll-mt-22">
            {children}
        </section>
    );
}

function H2({ children }: { children: React.ReactNode }) {
    return <h2 className="m-0 mb-4 font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em] sm:text-[30px]">{children}</h2>;
}

function H3({ children }: { children: React.ReactNode }) {
    return <h3 className="m-0 mt-6 mb-2 font-semibold text-[20px] text-foreground leading-[1.3] tracking-[-0.01em] first:mt-0">{children}</h3>;
}

function P({ children }: { children: React.ReactNode }) {
    return <p className="m-0 mb-4 font-sans text-[16px] text-foreground leading-[1.75] last:mb-0">{children}</p>;
}

function UL({ children }: { children: React.ReactNode }) {
    return <ul className="m-0 mt-3 mb-4 list-disc pl-6 font-sans text-[16px] text-foreground leading-[1.75]">{children}</ul>;
}

function LI({ children }: { children: React.ReactNode }) {
    return <li className="my-2">{children}</li>;
}

function A({ href, children, external }: { href: string; children: React.ReactNode; external?: boolean }) {
    return (
        <a href={href} {...(external ? { target: "_blank", rel: "noreferrer" } : {})} className="text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
            {children}
        </a>
    );
}

export function TermsPage() {
    return (
        <LegalContainer>
            {/* Page head */}
            <header className="mb-12 border-border border-b pb-8">
                <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                        <h1 className="m-0 mb-3.5 text-balance font-extrabold text-[clamp(36px,5vw,52px)] text-foreground leading-[1.05] tracking-[-0.02em]">Terms of Service</h1>
                        <p className="m-0 max-w-[56ch] font-sans text-[20px] text-muted-foreground leading-[1.55]">Legal agreement governing your use of Myrtle</p>
                    </div>
                    <div className="flex shrink-0 gap-2 pt-2">
                        <Badge variant="outline" size="lg">
                            {VERSION}
                        </Badge>
                    </div>
                </div>
                <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 font-sans text-[14px] text-muted-foreground">
                    <span className="inline-flex items-center gap-2">
                        <Calendar className="size-4" aria-hidden="true" />
                        Effective: {EFFECTIVE_DATE}
                    </span>
                    <span className="hidden h-4 w-px bg-border sm:inline-block" aria-hidden="true" />
                    <span className="inline-flex items-center gap-2">
                        <FileText className="size-4" aria-hidden="true" />
                        Last updated: {LAST_UPDATED}
                    </span>
                </div>
            </header>

            {/* TL;DR alert */}
            <Alert className="mb-10 border-[color-mix(in_srgb,var(--primary)_32%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] [&>svg]:text-primary">
                <ShieldIcon strokeWidth={1.8} />
                <AlertDescription className="text-[15px] text-muted-foreground leading-[1.55]">By accessing or using Myrtle, you agree to be bound by these Terms of Service. Please read them carefully before continuing.</AlertDescription>
            </Alert>

            {/* Table of contents */}
            <nav aria-label="Table of contents" className="mb-12 rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] p-6">
                <h2 className="m-0 mb-3.5 font-semibold text-[18px] text-foreground leading-[1.3]">Table of Contents</h2>
                <ol className="m-0 list-decimal pl-6 font-sans text-[14.5px] leading-[1.6]">
                    {TOC.map((entry) => (
                        <li key={entry.id} className="my-1.5">
                            <a href={`#${entry.id}`} className="text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
                                {entry.label}
                            </a>
                        </li>
                    ))}
                </ol>
            </nav>

            {/* 1. Acceptance */}
            <Section id="acceptance">
                <div className="mb-4 flex items-center gap-3">
                    <div className="inline-flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">
                        <ScaleIcon className="size-5" strokeWidth={1.8} />
                    </div>
                    <H2>1. Acceptance of Terms</H2>
                </div>
                <div className="pl-13 max-sm:pl-0">
                    <P>By accessing, browsing, or using the Myrtle platform ("Service"), you acknowledge that you have read, understood, and agree to be bound by these Terms of Service and all applicable laws and regulations.</P>
                    <P>If you do not agree with any part of these terms, you must not use our Service.</P>
                    <P>These terms constitute a legally binding agreement between you ("User," "you," or "your") and Myrtle ("we," "us," or "our"). Your continued use of the Service signifies your acceptance of these terms and any modifications thereto.</P>
                </div>
            </Section>

            <LegalDivider />

            {/* 2. Account */}
            <Section id="account">
                <H2>2. Account Registration and Security</H2>
                <H3>2.1 Account Creation via Yostar OAuth</H3>
                <P>
                    To access account features like profile sync, tier lists, and leaderboards, you must authenticate using your Yostar account (the same account you use for Arknights). We use Yostar's email verification system - you will receive a verification code from Yostar to confirm your identity. We never see or
                    store your Yostar password.
                </P>
                <H3>2.2 Account Security</H3>
                <P>You are responsible for:</P>
                <UL>
                    <LI>Maintaining the security of your Yostar account and email</LI>
                    <LI>All activities that occur under your Myrtle account</LI>
                    <LI>Notifying us if you believe your account has been compromised</LI>
                </UL>
                <H3>2.3 Account Eligibility</H3>
                <P>You must be at least 13 years old to use this Service. By creating an account, you represent that you meet this age requirement and have the legal capacity to enter into these Terms. You must also have a valid Yostar account with Arknights.</P>
                <H3>2.4 Game Data Sync</H3>
                <P>
                    When you sync your account, we fetch your Arknights game data (operator roster, stage progress, etc.) from Yostar's servers. This data is stored on our servers to power features like your profile page, account scoring, and leaderboards. You can re-sync at any time to update your data, or delete your
                    account to remove all stored data.
                </P>
            </Section>

            <LegalDivider />

            {/* 3. Usage */}
            <Section id="usage">
                <H2>3. Acceptable Use Policy</H2>
                <P>You agree not to engage in any of the following prohibited activities:</P>
                <UL>
                    <LI>Using the Service for any illegal or unauthorized purpose</LI>
                    <LI>Attempting to gain unauthorized access to any portion of the Service or related systems</LI>
                    <LI>Interfering with or disrupting the Service or servers connected to the Service</LI>
                    <LI>Uploading or transmitting viruses, malware, or any other malicious code</LI>
                    <LI>Scraping, crawling, or using automated systems to extract data without permission</LI>
                    <LI>Impersonating any person or entity, or falsely stating your affiliation with any person or entity</LI>
                    <LI>Harassing, threatening, or intimidating other users</LI>
                    <LI>Violating any applicable local, state, national, or international law</LI>
                </UL>
                <P>We reserve the right to investigate and prosecute violations of any of the above to the fullest extent of the law.</P>
            </Section>

            <LegalDivider />

            {/* 4. Content */}
            <Section id="content">
                <H2>4. User Content and Submissions</H2>
                <P>Our Service allows you to create and share content such as tier lists ("User Content"). By creating User Content, you grant us a worldwide, non-exclusive, royalty-free license to display and distribute such content in connection with the Service.</P>
                <H3>4.1 Tier Lists</H3>
                <P>You can create tier lists with full version control - every edit is logged and previous versions are preserved. You control the visibility (public, private, or shared with specific users) and can set permissions for who can edit your tier lists.</P>
                <H3>4.2 Content Guidelines</H3>
                <P>You represent and warrant that your User Content:</P>
                <UL>
                    <LI>Is your original creation or you have the right to share it</LI>
                    <LI>Does not contain offensive, harmful, or inappropriate material</LI>
                    <LI>Does not impersonate other users or misrepresent your identity</LI>
                </UL>
                <P>We reserve the right to remove any User Content that violates these Terms or is otherwise objectionable at our sole discretion.</P>
            </Section>

            <LegalDivider />

            {/* 5. IP */}
            <Section id="intellectual">
                <H2>5. Intellectual Property Rights</H2>
                <H3>5.1 Myrtle Code and Content</H3>
                <P>
                    The Myrtle platform code is open source and available on GitHub. Original features, designs, and documentation created for Myrtle are provided under applicable open source licenses. See our{" "}
                    <A href={REPO_URL} external>
                        GitHub repository
                    </A>{" "}
                    for specific license terms.
                </P>
                <H3>5.2 Arknights Assets and Content</H3>
                <P>
                    Arknights and all related assets, characters, operator data, artwork, voice lines, and imagery are the property of Hypergryph Network Technology Co., Ltd. and Yostar Limited. This Service is a fan-made project and is not officially affiliated with, endorsed by, or sponsored by Hypergryph or Yostar.
                </P>
                <P>We use game assets under fair use for the purpose of providing game companion tools to the community. All operator data, images, Spine animations, and other game content displayed on Myrtle are sourced from the official Arknights game files.</P>
                <H3>5.3 Third-Party Content</H3>
                <P>
                    Our asset processing uses FlatBuffers schemas from the{" "}
                    <A href={FBS_URL} external>
                        OpenArknightsFBS
                    </A>{" "}
                    project (MooncellWiki). DPS calculator implementations are based on community research and reference calculations.
                </P>
            </Section>

            <LegalDivider />

            {/* 6. Termination */}
            <Section id="termination">
                <H2>6. Termination and Suspension</H2>
                <P>We may terminate or suspend your account and access to the Service immediately, without prior notice or liability, for any reason, including but not limited to breach of these Terms.</P>
                <P>Upon termination, your right to use the Service will immediately cease. If you wish to terminate your account, you may discontinue using the Service and contact us to request account deletion.</P>
                <P>All provisions of these Terms which by their nature should survive termination shall survive, including ownership provisions, warranty disclaimers, and limitations of liability.</P>
            </Section>

            <LegalDivider />

            {/* 7. Liability */}
            <Section id="liability">
                <H2>7. Limitation of Liability</H2>
                <P>The Service is provided on an "AS IS" and "AS AVAILABLE" basis without warranties of any kind, either express or implied. We do not warrant that the Service will be uninterrupted, secure, or error-free.</P>
                <P>To the maximum extent permitted by law, Myrtle shall not be liable for any indirect, incidental, special, consequential, or punitive damages, including but not limited to loss of profits, data, or other intangible losses, resulting from:</P>
                <UL>
                    <LI>Your access to or use of (or inability to access or use) the Service</LI>
                    <LI>Any unauthorized access to or use of our servers and/or any personal information stored therein</LI>
                    <LI>Any interruption or cessation of transmission to or from the Service</LI>
                    <LI>Any bugs, viruses, or other harmful code that may be transmitted through the Service</LI>
                </UL>
            </Section>

            <LegalDivider />

            {/* 8. Changes */}
            <Section id="changes">
                <H2>8. Changes to Terms</H2>
                <P>We reserve the right to modify or replace these Terms at any time at our sole discretion. If a revision is material, we will provide at least 30 days' notice prior to any new terms taking effect.</P>
                <P>What constitutes a material change will be determined at our sole discretion. By continuing to access or use our Service after revisions become effective, you agree to be bound by the revised terms.</P>
            </Section>

            <LegalDivider />

            {/* 9. Contact */}
            <Section id="contact">
                <H2>9. Contact Information</H2>
                <P>If you have any questions about these Terms, please contact us:</P>
                <div className="rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] px-6 py-5">
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">Privacy &amp; GDPR requests</p>
                        <A href={PRIVACY_MAILTO}>{PRIVACY_EMAIL}</A>
                        <p className="m-0 mt-1 font-sans text-[12.5px] text-muted-foreground leading-normal">For access, rectification, erasure, portability, or objection requests under the GDPR (or comparable data-protection laws).</p>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">General contact</p>
                        <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">Source code</p>
                        <A href={REPO_URL} external>
                            github.com/Eltik/myrtle
                        </A>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">In-app</p>
                        <Link to="/" className="font-sans text-[14px] text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
                            Manage your account in settings
                        </Link>
                    </div>
                </div>
            </Section>

            <RelatedLinksFooter>
                <RelatedDocLink to="/privacy" label="Privacy Policy" />
            </RelatedLinksFooter>
        </LegalContainer>
    );
}
