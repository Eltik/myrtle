import { Link } from "@tanstack/react-router";
import { Calendar, FileText, ShieldIcon } from "lucide-react";
import { LegalContainer, LegalDivider, LegalHeader, LegalHeaderLine, RelatedDocLink, RelatedLinksFooter } from "#/components/legal/LegalShell";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { CONTACT_EMAIL, CONTACT_MAILTO, PRIVACY_EMAIL, PRIVACY_MAILTO, REPO_URL } from "#/lib/constants";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./TermsPage.messages";

const EFFECTIVE_DATE = "January 12, 2026";
const LAST_UPDATED = "January 12, 2026";
const VERSION = "2.0";
const FBS_URL = "https://github.com/MooncellWiki/OpenArknightsFBS";

/** Anchor ids paired with the message key for each entry's visible label. */
const TOC = [
    { id: "acceptance", labelKey: "terms.toc.acceptance" as const },
    { id: "account", labelKey: "terms.toc.account" as const },
    { id: "usage", labelKey: "terms.toc.usage" as const },
    { id: "content", labelKey: "terms.toc.content" as const },
    { id: "intellectual", labelKey: "terms.toc.intellectual" as const },
    { id: "termination", labelKey: "terms.toc.termination" as const },
    { id: "liability", labelKey: "terms.toc.liability" as const },
    { id: "changes", labelKey: "terms.toc.changes" as const },
    { id: "contact", labelKey: "terms.toc.contact" as const },
];

function Section({ id, children }: { id: string; children: React.ReactNode }) {
    return (
        <section id={id} className="mb-10 scroll-mt-22">
            {children}
        </section>
    );
}

function H2({ children }: { children: React.ReactNode }) {
    return <h2 className="m-0 mb-4 font-semibold text-[20px] text-foreground leading-[1.25] tracking-[-0.01em] sm:text-[22px]">{children}</h2>;
}

function H3({ children }: { children: React.ReactNode }) {
    return <h3 className="m-0 mt-6 mb-2 font-semibold text-[17px] text-foreground leading-[1.3] tracking-[-0.01em] first:mt-0">{children}</h3>;
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
        <a
            href={href}
            {...(external ? { target: "_blank", rel: "noreferrer" } : {})}
            className="relative text-primary underline decoration-1 underline-offset-[3px] transition-[text-decoration-thickness] pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:decoration-2"
        >
            {children}
        </a>
    );
}

export function TermsPage() {
    const t: TypedT<typeof messages> = useT("legal");
    const rt: TypedRichT<typeof messages> = useRichT("legal");

    return (
        <LegalContainer>
            <LegalHeader title={t("terms.hero.title")} subtitle={t("terms.hero.subtitle")} version={t("terms.hero.version", { version: VERSION })}>
                <LegalHeaderLine icon={<Calendar className="size-4" aria-hidden="true" />}>{t("terms.hero.effective", { date: EFFECTIVE_DATE })}</LegalHeaderLine>
                <LegalHeaderLine icon={<FileText className="size-4" aria-hidden="true" />} separated>
                    {t("terms.hero.lastUpdated", { date: LAST_UPDATED })}
                </LegalHeaderLine>
            </LegalHeader>

            <Alert className="mb-10 border-[color-mix(in_srgb,var(--primary)_32%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] [&>svg]:text-primary">
                <ShieldIcon strokeWidth={1.8} />
                <AlertDescription className="text-[15px] text-muted-foreground leading-[1.55]">{t("terms.tldr")}</AlertDescription>
            </Alert>

            <nav aria-label={t("terms.toc.nav")} className="mb-12 rounded-2xl border border-border bg-card p-6">
                <h2 className="m-0 mb-3.5 font-semibold text-[18px] text-foreground leading-[1.3]">{t("terms.toc.heading")}</h2>
                <ol className="m-0 list-decimal pl-6 font-sans text-[14.5px] leading-[1.6]">
                    {TOC.map((entry) => (
                        <li key={entry.id} className="my-1.5 pointer-coarse:my-0">
                            <a href={`#${entry.id}`} className="pointer-coarse:inline-flex pointer-coarse:min-h-11 pointer-coarse:items-center text-foreground underline decoration-1 underline-offset-[3px] transition-[text-decoration-thickness] hover:decoration-2">
                                {t(entry.labelKey)}
                            </a>
                        </li>
                    ))}
                </ol>
            </nav>

            <Section id="acceptance">
                <div className="mb-4">
                    <H2>{t("terms.s1.heading")}</H2>
                </div>
                <div>
                    <P>{t("terms.s1.p1")}</P>
                    <P>{t("terms.s1.p2")}</P>
                    <P>{t("terms.s1.p3")}</P>
                </div>
            </Section>

            <LegalDivider />

            <Section id="account">
                <H2>{t("terms.s2.heading")}</H2>
                <H3>{t("terms.s2.oauth.heading")}</H3>
                <P>{t("terms.s2.oauth.body")}</P>
                <H3>{t("terms.s2.security.heading")}</H3>
                <P>{t("terms.s2.security.intro")}</P>
                <UL>
                    <LI>{t("terms.s2.security.item1")}</LI>
                    <LI>{t("terms.s2.security.item2")}</LI>
                    <LI>{t("terms.s2.security.item3")}</LI>
                </UL>
                <H3>{t("terms.s2.eligibility.heading")}</H3>
                <P>{t("terms.s2.eligibility.body")}</P>
                <H3>{t("terms.s2.sync.heading")}</H3>
                <P>{t("terms.s2.sync.body")}</P>
            </Section>

            <LegalDivider />

            <Section id="usage">
                <H2>{t("terms.s3.heading")}</H2>
                <P>{t("terms.s3.intro")}</P>
                <UL>
                    <LI>{t("terms.s3.item1")}</LI>
                    <LI>{t("terms.s3.item2")}</LI>
                    <LI>{t("terms.s3.item3")}</LI>
                    <LI>{t("terms.s3.item4")}</LI>
                    <LI>{t("terms.s3.item5")}</LI>
                    <LI>{t("terms.s3.item6")}</LI>
                    <LI>{t("terms.s3.item7")}</LI>
                    <LI>{t("terms.s3.item8")}</LI>
                </UL>
                <P>{t("terms.s3.outro")}</P>
            </Section>

            <LegalDivider />

            <Section id="content">
                <H2>{t("terms.s4.heading")}</H2>
                <P>{t("terms.s4.p1")}</P>
                <H3>{t("terms.s4.tierLists.heading")}</H3>
                <P>{t("terms.s4.tierLists.body")}</P>
                <H3>{t("terms.s4.guidelines.heading")}</H3>
                <P>{t("terms.s4.guidelines.intro")}</P>
                <UL>
                    <LI>{t("terms.s4.guidelines.item1")}</LI>
                    <LI>{t("terms.s4.guidelines.item2")}</LI>
                    <LI>{t("terms.s4.guidelines.item3")}</LI>
                </UL>
                <P>{t("terms.s4.outro")}</P>
            </Section>

            <LegalDivider />

            <Section id="intellectual">
                <H2>{t("terms.s5.heading")}</H2>
                <H3>{t("terms.s5.code.heading")}</H3>
                <P>
                    {rt("terms.s5.code.body", {
                        link: (
                            <A href={REPO_URL} external>
                                {t("terms.s5.code.link")}
                            </A>
                        ),
                    })}
                </P>
                <H3>{t("terms.s5.assets.heading")}</H3>
                <P>{t("terms.s5.assets.p1")}</P>
                <P>{t("terms.s5.assets.p2")}</P>
                <H3>{t("terms.s5.thirdParty.heading")}</H3>
                <P>
                    {rt("terms.s5.thirdParty.body", {
                        link: (
                            <A href={FBS_URL} external>
                                {t("terms.s5.thirdParty.link")}
                            </A>
                        ),
                    })}
                </P>
            </Section>

            <LegalDivider />

            <Section id="termination">
                <H2>{t("terms.s6.heading")}</H2>
                <P>{t("terms.s6.p1")}</P>
                <P>{t("terms.s6.p2")}</P>
                <P>{t("terms.s6.p3")}</P>
            </Section>

            <LegalDivider />

            <Section id="liability">
                <H2>{t("terms.s7.heading")}</H2>
                <P>{t("terms.s7.p1")}</P>
                <P>{t("terms.s7.p2")}</P>
                <UL>
                    <LI>{t("terms.s7.item1")}</LI>
                    <LI>{t("terms.s7.item2")}</LI>
                    <LI>{t("terms.s7.item3")}</LI>
                    <LI>{t("terms.s7.item4")}</LI>
                </UL>
            </Section>

            <LegalDivider />

            <Section id="changes">
                <H2>{t("terms.s8.heading")}</H2>
                <P>{t("terms.s8.p1")}</P>
                <P>{t("terms.s8.p2")}</P>
            </Section>

            <LegalDivider />

            <Section id="contact">
                <H2>{t("terms.s9.heading")}</H2>
                <P>{t("terms.s9.intro")}</P>
                <div className="rounded-2xl border border-border bg-card px-6 py-5">
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("terms.s9.gdpr.label")}</p>
                        <A href={PRIVACY_MAILTO}>{PRIVACY_EMAIL}</A>
                        <p className="m-0 mt-1 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("terms.s9.gdpr.note")}</p>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("terms.s9.general.label")}</p>
                        <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("terms.s9.source.label")}</p>
                        <A href={REPO_URL} external>
                            github.com/Eltik/myrtle
                        </A>
                    </div>
                    <div className="my-3.5 first:mt-0 last:mb-0">
                        <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("terms.s9.inApp.label")}</p>
                        <Link
                            to="/"
                            className="relative font-sans text-[14px] text-primary underline decoration-1 underline-offset-[3px] transition-[text-decoration-thickness] pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:decoration-2"
                        >
                            {t("terms.s9.inApp.link")}
                        </Link>
                    </div>
                </div>
            </Section>

            <RelatedLinksFooter>
                <RelatedDocLink to="/privacy" label={t("terms.related.privacy")} />
            </RelatedLinksFooter>
        </LegalContainer>
    );
}
