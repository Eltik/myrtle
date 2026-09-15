import { Link } from "@tanstack/react-router";
import { ChevronDown, Database, Eye, KeyRound, Lock, Mail, Settings as SettingsIcon, ShieldIcon, UserCog } from "lucide-react";
import type { ReactNode } from "react";
import { LegalContainer, LegalDivider, RelatedDocLink, RelatedLinksFooter } from "#/components/legal/LegalShell";
import { Alert, AlertDescription, AlertTitle } from "#/components/ui/alert";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { CONTACT_EMAIL, CONTACT_MAILTO, PRIVACY_EMAIL, PRIVACY_MAILTO, REPO_URL } from "#/lib/constants";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import type { messages } from "./PrivacyPage.messages";

const EFFECTIVE_DATE = "Jan 12, 2026";
const VERSION = "2.0";

function Section({ id, children }: { id: string; children: ReactNode }) {
    return (
        <section id={id} className="mb-10 scroll-mt-22">
            {children}
        </section>
    );
}

function SectionHead({ icon, title, subtitle }: { icon: ReactNode; title: string; subtitle?: string }) {
    return (
        <div className="mb-4 flex items-center gap-3">
            <div className="inline-flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">{icon}</div>
            <div className="min-w-0">
                <h2 className="m-0 font-(--font-heading) text-[26px] text-foreground leading-[1.2] tracking-[-0.02em] sm:text-[30px]">{title}</h2>
                {subtitle ? <p className="m-0 mt-1 font-sans text-[14px] text-muted-foreground leading-normal">{subtitle}</p> : null}
            </div>
        </div>
    );
}

function H2Plain({ children }: { children: ReactNode }) {
    return <h2 className="m-0 mb-4 font-(--font-heading) text-[26px] text-foreground leading-[1.2] tracking-[-0.02em] sm:text-[30px]">{children}</h2>;
}

function H3({ children }: { children: ReactNode }) {
    return <h3 className="m-0 mt-6 mb-2 font-(--font-heading) text-[20px] text-foreground leading-[1.3] tracking-[-0.01em] first:mt-0">{children}</h3>;
}

function P({ children, muted }: { children: ReactNode; muted?: boolean }) {
    return <p className={`m-0 mb-4 font-sans text-[16px] leading-[1.75] last:mb-0 ${muted ? "text-muted-foreground" : "text-foreground"}`}>{children}</p>;
}

function UL({ children }: { children: ReactNode }) {
    return <ul className="m-0 mt-3 mb-4 list-disc pl-6 font-sans text-[16px] text-foreground leading-[1.75]">{children}</ul>;
}

function LI({ children }: { children: ReactNode }) {
    return <li className="my-2">{children}</li>;
}

function A({ href, children, external }: { href: string; children: ReactNode; external?: boolean }) {
    return (
        <a href={href} {...(external ? { target: "_blank", rel: "noreferrer" } : {})} className="text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
            {children}
        </a>
    );
}

function Tile({ icon, title, desc }: { icon: ReactNode; title: string; desc: string }) {
    return (
        <div className="relative rounded-xl border border-border bg-card p-4.5 shadow-xs/5">
            <div className="mb-2.5 text-primary [&_svg]:size-5.5">{icon}</div>
            <p className="m-0 mb-1.5 font-sans font-semibold text-[14.5px] text-foreground leading-[1.3]">{title}</p>
            <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.55]">{desc}</p>
        </div>
    );
}

function Callout({ children, tone }: { children: ReactNode; tone?: "primary" | "success" | "default" }) {
    const toneCls =
        tone === "primary"
            ? "border-[color-mix(in_srgb,var(--primary)_20%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)]"
            : tone === "success"
              ? "border-[color-mix(in_srgb,var(--success)_20%,transparent)] bg-[color-mix(in_srgb,var(--success)_5%,transparent)]"
              : "border-border bg-[color-mix(in_srgb,var(--muted)_40%,transparent)]";
    return <div className={`my-4 rounded-xl border px-4.5 py-4 ${toneCls}`}>{children}</div>;
}

function CalloutTitle({ children, withIcon }: { children: ReactNode; withIcon?: ReactNode }) {
    return (
        <h4 className="m-0 mb-2 inline-flex items-center gap-2 font-sans font-semibold text-[14.5px] text-foreground leading-[1.4]">
            {withIcon}
            {children}
        </h4>
    );
}

function CalloutBody({ children }: { children: ReactNode }) {
    return <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.6]">{children}</p>;
}

function BulletList({ items }: { items: { label: ReactNode; body: ReactNode }[] }) {
    return (
        <ul className="m-0 mt-3 mb-4 list-none p-0">
            {items.map((it, i) => (
                // biome-ignore lint/suspicious/noArrayIndexKey: static content
                <li key={i} className="flex items-start gap-2.5 py-1.5">
                    <span className="mt-1.75 inline-flex size-3.5 shrink-0 items-center justify-center rounded-full bg-primary/20">
                        <span className="size-1.5 rounded-full bg-primary" />
                    </span>
                    <span className="font-sans text-[15px] text-foreground leading-[1.6]">
                        <strong className="font-semibold">{it.label}</strong> {it.body}
                    </span>
                </li>
            ))}
        </ul>
    );
}

function Disclosure({ summary, children, defaultOpen }: { summary: string; children: ReactNode; defaultOpen?: boolean }) {
    return (
        <details open={defaultOpen} className="group my-2 rounded-lg border border-border bg-[color-mix(in_srgb,var(--muted)_40%,transparent)] px-4.5 [&_summary::-webkit-details-marker]:hidden">
            <summary className="flex cursor-pointer list-none items-center justify-between gap-4 py-3.5 font-medium font-sans text-[15px] text-foreground outline-none">
                <span>{summary}</span>
                <ChevronDown className="size-4 text-muted-foreground transition-transform duration-200 group-open:rotate-180" strokeWidth={2} />
            </summary>
            <div className="pt-0 pb-3.5 font-sans text-[14px] text-muted-foreground leading-[1.6]">{children}</div>
        </details>
    );
}

export function PrivacyPage() {
    const t: TypedT<typeof messages> = useT("legal");
    const rt: TypedRichT<typeof messages> = useRichT("legal");

    return (
        <LegalContainer ambient>
            {/* Hero */}
            <header className="relative z-1 mb-12 flex flex-col items-center text-center">
                <div className="mb-5.5 inline-flex size-18 items-center justify-center rounded-full bg-primary/10 text-primary">
                    <ShieldIcon className="size-10" strokeWidth={1.8} />
                </div>
                <h1 className="m-0 mb-3.5 text-balance font-extrabold text-[clamp(36px,5vw,52px)] text-foreground leading-[1.05] tracking-[-0.02em]">{t("privacy.hero.title")}</h1>
                <p className="m-0 max-w-[56ch] font-sans text-[20px] text-muted-foreground leading-[1.55]">{t("privacy.hero.subtitle")}</p>
                <div className="mt-5.5 flex flex-wrap items-center justify-center gap-2">
                    <Badge variant="secondary" size="lg">
                        {t("privacy.hero.effective", { date: EFFECTIVE_DATE })}
                    </Badge>
                    <Badge variant="outline" size="lg">
                        {t("privacy.hero.version", { version: VERSION })}
                    </Badge>
                </div>
            </header>

            {/* TL;DR */}
            <Alert className="mb-10 border-[color-mix(in_srgb,var(--primary)_32%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] [&>svg]:text-primary">
                <Eye strokeWidth={2} />
                <AlertTitle className="text-[15px] text-foreground">{t("privacy.tldr.title")}</AlertTitle>
                <AlertDescription className="text-[15px] text-muted-foreground leading-[1.55]">{t("privacy.tldr.body")}</AlertDescription>
            </Alert>

            {/* Principles */}
            <section className="my-12">
                <h2 className="m-0 mb-5 text-center font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em]">{t("privacy.principles.heading")}</h2>
                <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-2 lg:grid-cols-3">
                    <Tile icon={<Lock strokeWidth={1.8} />} title={t("privacy.principles.minimal.title")} desc={t("privacy.principles.minimal.desc")} />
                    <Tile icon={<Eye strokeWidth={1.8} />} title={t("privacy.principles.transparency.title")} desc={t("privacy.principles.transparency.desc")} />
                    <Tile icon={<UserCog strokeWidth={1.8} />} title={t("privacy.principles.control.title")} desc={t("privacy.principles.control.desc")} />
                </div>
            </section>

            <LegalDivider />

            {/* Yostar OAuth */}
            <Section id="yostar">
                <SectionHead icon={<KeyRound className="size-5" strokeWidth={1.8} />} title={t("privacy.yostar.heading")} subtitle={t("privacy.yostar.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <P>{t("privacy.yostar.intro")}</P>

                    <Callout tone="primary">
                        <CalloutTitle>{t("privacy.yostar.auth.title")}</CalloutTitle>
                        <CalloutBody>{t("privacy.yostar.auth.body")}</CalloutBody>
                    </Callout>

                    <Callout>
                        <CalloutTitle>{t("privacy.yostar.receive.title")}</CalloutTitle>
                        <CalloutBody>
                            {t("privacy.yostar.receive.p1")}
                            <br />
                            <br />
                            {t("privacy.yostar.receive.p2")}
                        </CalloutBody>
                    </Callout>

                    <p className="m-0 mt-4 font-sans text-[14px] text-muted-foreground leading-[1.6]">
                        {rt("privacy.yostar.revoke.body", {
                            path: <strong className="text-foreground">{t("privacy.yostar.revoke.path")}</strong>,
                            email: <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>,
                        })}
                    </p>
                </div>
            </Section>

            <LegalDivider />

            {/* Information We Collect */}
            <Section id="information-we-collect">
                <SectionHead icon={<Database className="size-5" strokeWidth={1.8} />} title={t("privacy.collect.heading")} subtitle={t("privacy.collect.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <H3>{t("privacy.collect.account.heading")}</H3>
                    <p className="m-0 mb-2 font-sans text-[15px] text-muted-foreground leading-[1.6]">{t("privacy.collect.account.intro")}</p>
                    <BulletList
                        items={[
                            { label: t("privacy.collect.account.uid.label"), body: t("privacy.collect.account.uid.body") },
                            { label: t("privacy.collect.account.settings.label"), body: t("privacy.collect.account.settings.body") },
                        ]}
                    />
                    <Callout tone="success">
                        <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.6]">
                            <strong className="text-success-foreground">{t("privacy.collect.note")}</strong> {t("privacy.collect.emailNote")}
                        </p>
                    </Callout>

                    <H3>{t("privacy.collect.game.heading")}</H3>
                    <p className="m-0 mb-2 font-sans text-[15px] text-muted-foreground leading-[1.6]">{t("privacy.collect.game.intro")}</p>
                    <BulletList
                        items={[
                            { label: t("privacy.collect.game.roster.label"), body: t("privacy.collect.game.roster.body") },
                            { label: t("privacy.collect.game.stages.label"), body: t("privacy.collect.game.stages.body") },
                            { label: t("privacy.collect.game.roguelike.label"), body: t("privacy.collect.game.roguelike.body") },
                            { label: t("privacy.collect.game.base.label"), body: t("privacy.collect.game.base.body") },
                            { label: t("privacy.collect.game.medals.label"), body: t("privacy.collect.game.medals.body") },
                        ]}
                    />

                    <H3>{t("privacy.collect.saved.heading")}</H3>
                    <BulletList items={[{ label: t("privacy.collect.saved.dps.label"), body: t("privacy.collect.saved.dps.body") }]} />

                    <H3>{t("privacy.collect.technical.heading")}</H3>
                    <BulletList items={[{ label: t("privacy.collect.technical.logs.label"), body: t("privacy.collect.technical.logs.body") }]} />
                    <Callout tone="success">
                        <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.6]">
                            <strong className="text-success-foreground">{t("privacy.collect.note")}</strong> {t("privacy.collect.ipNote")}
                        </p>
                    </Callout>
                </div>
            </Section>

            <LegalDivider />

            {/* How we use info */}
            <Section id="how-we-use">
                <SectionHead icon={<Eye className="size-5" strokeWidth={1.8} />} title={t("privacy.use.heading")} subtitle={t("privacy.use.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <Disclosure summary={t("privacy.use.service.summary")} defaultOpen>
                        {t("privacy.use.service.body")}
                    </Disclosure>
                    <Disclosure summary={t("privacy.use.leaderboards.summary")}>{t("privacy.use.leaderboards.body")}</Disclosure>
                    <Disclosure summary={t("privacy.use.improvement.summary")}>{t("privacy.use.improvement.body")}</Disclosure>
                    <Disclosure summary={t("privacy.use.security.summary")}>{t("privacy.use.security.body")}</Disclosure>
                </div>
            </Section>

            <LegalDivider />

            {/* Settings & visibility */}
            <Section id="settings">
                <SectionHead icon={<SettingsIcon className="size-5" strokeWidth={1.8} />} title={t("privacy.settings.heading")} subtitle={t("privacy.settings.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <P>{t("privacy.settings.intro")}</P>
                    <div className="my-4 grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                        <Callout>
                            <CalloutTitle withIcon={<Eye className="size-4 text-primary" strokeWidth={1.8} />}>{t("privacy.settings.visibility.title")}</CalloutTitle>
                            <CalloutBody>{t("privacy.settings.visibility.body")}</CalloutBody>
                        </Callout>
                        <Callout>
                            <CalloutTitle
                                withIcon={
                                    <svg className="size-4 text-primary" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={1.8} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
                                        <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
                                        <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
                                        <path d="M4 22h16" />
                                        <path d="M10 14.66V17c0 .55-.47.98-.97 1.21C7.85 18.75 7 20.24 7 22" />
                                        <path d="M14 14.66V17c0 .55.47.98.97 1.21C16.15 18.75 17 20.24 17 22" />
                                        <path d="M18 2H6v7a6 6 0 0 0 12 0V2z" />
                                    </svg>
                                }
                            >
                                {t("privacy.settings.leaderboard.title")}
                            </CalloutTitle>
                            <CalloutBody>{t("privacy.settings.leaderboard.body")}</CalloutBody>
                        </Callout>
                    </div>
                    <Callout tone="primary">
                        <CalloutTitle>{t("privacy.settings.public.title")}</CalloutTitle>
                        <ul className="m-0 list-disc pl-5 font-sans text-[13.5px] text-muted-foreground leading-[1.65]">
                            <li className="my-1">{t("privacy.settings.public.nickname")}</li>
                            <li className="my-1">{t("privacy.settings.public.roster")}</li>
                            <li className="my-1">{t("privacy.settings.public.rankings")}</li>
                            <li className="my-1">{t("privacy.settings.public.avatar")}</li>
                        </ul>
                    </Callout>
                    <Callout>
                        <CalloutTitle>{t("privacy.settings.private.title")}</CalloutTitle>
                        <ul className="m-0 list-disc pl-5 font-sans text-[13.5px] text-muted-foreground leading-[1.65]">
                            <li className="my-1">{t("privacy.settings.private.tokens")}</li>
                            <li className="my-1">{t("privacy.settings.private.settings")}</li>
                            <li className="my-1">{t("privacy.settings.private.dps")}</li>
                            <li className="my-1">{t("privacy.settings.private.email")}</li>
                        </ul>
                    </Callout>
                </div>
            </Section>

            <LegalDivider />

            {/* Security */}
            <Section id="security">
                <SectionHead icon={<Lock className="size-5" strokeWidth={1.8} />} title={t("privacy.security.heading")} subtitle={t("privacy.security.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <P>{t("privacy.security.intro")}</P>
                    <div className="my-4 grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                        <Callout tone="primary">
                            <CalloutTitle>{t("privacy.security.encryption.title")}</CalloutTitle>
                            <CalloutBody>{t("privacy.security.encryption.body")}</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>{t("privacy.security.jwt.title")}</CalloutTitle>
                            <CalloutBody>{t("privacy.security.jwt.body")}</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>{t("privacy.security.rateLimit.title")}</CalloutTitle>
                            <CalloutBody>{t("privacy.security.rateLimit.body")}</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>{t("privacy.security.redis.title")}</CalloutTitle>
                            <CalloutBody>{t("privacy.security.redis.body")}</CalloutBody>
                        </Callout>
                    </div>
                    <Callout>
                        <CalloutTitle>{t("privacy.security.database.title")}</CalloutTitle>
                        <CalloutBody>{t("privacy.security.database.body")}</CalloutBody>
                    </Callout>
                    <p className="m-0 mt-4 font-sans text-[14px] text-muted-foreground leading-[1.6]">
                        {rt("privacy.security.openSource", {
                            link: (
                                <A href={REPO_URL} external>
                                    {t("privacy.security.githubLink")}
                                </A>
                            ),
                        })}
                    </p>
                </div>
            </Section>

            <LegalDivider />

            {/* Rights */}
            <Section id="rights">
                <SectionHead icon={<UserCog className="size-5" strokeWidth={1.8} />} title={t("privacy.rights.heading")} subtitle={t("privacy.rights.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <P>{t("privacy.rights.intro")}</P>
                    <Disclosure summary={t("privacy.rights.access.summary")}>{t("privacy.rights.access.body")}</Disclosure>
                    <Disclosure summary={t("privacy.rights.correction.summary")}>{t("privacy.rights.correction.body")}</Disclosure>
                    <Disclosure summary={t("privacy.rights.visibility.summary")}>{t("privacy.rights.visibility.body")}</Disclosure>
                    <Disclosure summary={t("privacy.rights.deletion.summary")}>{rt("privacy.rights.deletion.body", { email: <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A> })}</Disclosure>
                </div>
            </Section>

            <LegalDivider />

            {/* Third-party */}
            <Section id="third-party">
                <H2Plain>{t("privacy.thirdParty.heading")}</H2Plain>
                <P>{t("privacy.thirdParty.intro")}</P>
                <UL>
                    <LI>
                        <strong>{t("privacy.thirdParty.yostar.label")}</strong> {t("privacy.thirdParty.yostar.body")}
                    </LI>
                    <LI>
                        <strong>{t("privacy.thirdParty.servers.label")}</strong> {t("privacy.thirdParty.servers.body")}
                    </LI>
                    <LI>
                        <strong>{t("privacy.thirdParty.hosting.label")}</strong> {t("privacy.thirdParty.hosting.body")}
                    </LI>
                </UL>
                <P muted>{t("privacy.thirdParty.noSelling")}</P>
            </Section>

            <LegalDivider />

            {/* Children */}
            <Section id="children">
                <H2Plain>{t("privacy.children.heading")}</H2Plain>
                <P>{t("privacy.children.body")}</P>
            </Section>

            <LegalDivider />

            {/* Changes */}
            <Section id="changes">
                <H2Plain>{t("privacy.changes.heading")}</H2Plain>
                <P>{t("privacy.changes.p1")}</P>
                <P>{t("privacy.changes.p2")}</P>
            </Section>

            <LegalDivider />

            {/* Contact */}
            <Section id="contact">
                <SectionHead icon={<Mail className="size-5" strokeWidth={1.8} />} title={t("privacy.contact.heading")} subtitle={t("privacy.contact.subheading")} />
                <div className="pl-13 max-sm:pl-0">
                    <div className="rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] px-6 py-5">
                        <p className="m-0 font-sans font-semibold text-[15px] text-foreground">{t("privacy.contact.getInTouch")}</p>
                        <p className="m-0 mt-1.5 mb-4 font-sans text-[14px] text-muted-foreground leading-[1.55]">{t("privacy.contact.intro")}</p>
                        <div className="my-3.5 first:mt-0 last:mb-0">
                            <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("privacy.contact.gdpr.label")}</p>
                            <A href={PRIVACY_MAILTO}>{PRIVACY_EMAIL}</A>
                            <p className="m-0 mt-1 font-sans text-[12.5px] text-muted-foreground leading-normal">{t("privacy.contact.gdpr.note")}</p>
                        </div>
                        <div className="my-3.5 first:mt-0 last:mb-0">
                            <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("privacy.contact.general.label")}</p>
                            <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>
                        </div>
                        <div className="my-3.5 first:mt-0 last:mb-0">
                            <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("privacy.contact.source.label")}</p>
                            <A href={REPO_URL} external>
                                github.com/Eltik/myrtle
                            </A>
                        </div>
                        <div className="my-3.5 first:mt-0 last:mb-0">
                            <p className="m-0 mb-1 font-medium font-sans text-[14px] text-foreground">{t("privacy.contact.inApp.label")}</p>
                            <Link to="/settings" className="font-sans text-[14px] text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
                                {t("privacy.contact.inApp.link")}
                            </Link>
                        </div>
                    </div>
                </div>
            </Section>

            {/* Footer CTA */}
            <div className="my-12 rounded-2xl border border-border bg-linear-to-b from-[color-mix(in_srgb,var(--muted)_50%,transparent)] to-transparent p-8 text-center">
                <h3 className="m-0 mb-2 font-semibold text-[22px] text-foreground leading-[1.3] tracking-[-0.01em]">{t("privacy.cta.title")}</h3>
                <p className="m-0 mx-auto max-w-[56ch] text-balance font-sans text-[15px] text-muted-foreground leading-[1.55]">{t("privacy.cta.body")}</p>
                <div className="mt-5 flex flex-wrap items-center justify-center gap-2.5">
                    <Button size="lg" render={<Link to="/settings" />}>
                        {t("privacy.cta.manageSettings")}
                    </Button>
                    <Button variant="outline" size="lg" render={<Link to="/" />}>
                        {t("privacy.cta.returnHome")}
                    </Button>
                </div>
            </div>

            <RelatedLinksFooter>
                <RelatedDocLink to="/terms" label={t("privacy.related.terms")} />
            </RelatedLinksFooter>
        </LegalContainer>
    );
}
