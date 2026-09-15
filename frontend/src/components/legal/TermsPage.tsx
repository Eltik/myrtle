import { Link } from "@tanstack/react-router";
import { Calendar, FileText, ScaleIcon, ShieldIcon } from "lucide-react";
import { LegalContainer, LegalDivider, RelatedDocLink, RelatedLinksFooter } from "#/components/legal/LegalShell";
import { Alert, AlertDescription } from "#/components/ui/alert";
import { Badge } from "#/components/ui/badge";
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
    const t: TypedT<typeof messages> = useT("legal");
    const rt: TypedRichT<typeof messages> = useRichT("legal");

    return (
        <LegalContainer>
            {/* Page head */}
            <header className="mb-12 border-border border-b pb-8">
                <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                        <h1 className="m-0 mb-3.5 text-balance font-extrabold text-[clamp(36px,5vw,52px)] text-foreground leading-[1.05] tracking-[-0.02em]">{t("terms.hero.title")}</h1>
                        <p className="m-0 max-w-[56ch] font-sans text-[20px] text-muted-foreground leading-[1.55]">{t("terms.hero.subtitle")}</p>
                    </div>
                    <div className="flex shrink-0 gap-2 pt-2">
                        <Badge variant="outline" size="lg">
                            {t("terms.hero.version", { version: VERSION })}
                        </Badge>
                    </div>
                </div>
                <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 font-sans text-[14px] text-muted-foreground">
                    <span className="inline-flex items-center gap-2">
                        <Calendar className="size-4" aria-hidden="true" />
                        {t("terms.hero.effective", { date: EFFECTIVE_DATE })}
                    </span>
                    <span className="hidden h-4 w-px bg-border sm:inline-block" aria-hidden="true" />
                    <span className="inline-flex items-center gap-2">
                        <FileText className="size-4" aria-hidden="true" />
                        {t("terms.hero.lastUpdated", { date: LAST_UPDATED })}
                    </span>
                </div>
            </header>

            {/* TL;DR alert */}
            <Alert className="mb-10 border-[color-mix(in_srgb,var(--primary)_32%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] [&>svg]:text-primary">
                <ShieldIcon strokeWidth={1.8} />
                <AlertDescription className="text-[15px] text-muted-foreground leading-[1.55]">{t("terms.tldr")}</AlertDescription>
            </Alert>

            {/* Table of contents */}
            <nav aria-label={t("terms.toc.nav")} className="mb-12 rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] p-6">
                <h2 className="m-0 mb-3.5 font-semibold text-[18px] text-foreground leading-[1.3]">{t("terms.toc.heading")}</h2>
                <ol className="m-0 list-decimal pl-6 font-sans text-[14.5px] leading-[1.6]">
                    {TOC.map((entry) => (
                        <li key={entry.id} className="my-1.5">
                            <a href={`#${entry.id}`} className="text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
                                {t(entry.labelKey)}
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
                    <H2>{t("terms.s1.heading")}</H2>
                </div>
                <div className="pl-13 max-sm:pl-0">
                    <P>{t("terms.s1.p1")}</P>
                    <P>{t("terms.s1.p2")}</P>
                    <P>{t("terms.s1.p3")}</P>
                </div>
            </Section>

            <LegalDivider />

            {/* 2. Account */}
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

            {/* 3. Usage */}
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

            {/* 4. Content */}
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

            {/* 5. IP */}
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

            {/* 6. Termination */}
            <Section id="termination">
                <H2>{t("terms.s6.heading")}</H2>
                <P>{t("terms.s6.p1")}</P>
                <P>{t("terms.s6.p2")}</P>
                <P>{t("terms.s6.p3")}</P>
            </Section>

            <LegalDivider />

            {/* 7. Liability */}
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

            {/* 8. Changes */}
            <Section id="changes">
                <H2>{t("terms.s8.heading")}</H2>
                <P>{t("terms.s8.p1")}</P>
                <P>{t("terms.s8.p2")}</P>
            </Section>

            <LegalDivider />

            {/* 9. Contact */}
            <Section id="contact">
                <H2>{t("terms.s9.heading")}</H2>
                <P>{t("terms.s9.intro")}</P>
                <div className="rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] px-6 py-5">
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
                        <Link to="/" className="font-sans text-[14px] text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
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
