import { Link } from "@tanstack/react-router";
import { ChevronDown, Database, Eye, KeyRound, Lock, Mail, Settings as SettingsIcon, ShieldIcon, UserCog } from "lucide-react";
import type { ReactNode } from "react";
import { LegalContainer, LegalDivider, RelatedDocLink, RelatedLinksFooter } from "#/components/legal/LegalShell";
import { Alert, AlertDescription, AlertTitle } from "#/components/ui/alert";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";

const EFFECTIVE_DATE = "Jan 12, 2026";
const VERSION = "Version 2.0";
const REPO_URL = "https://github.com/Eltik/myrtle";
const CONTACT_EMAIL = "contact@myrtle.moe";
const CONTACT_MAILTO = `mailto:${CONTACT_EMAIL}`;
const PRIVACY_EMAIL = "privacy@myrtle.moe";
const PRIVACY_MAILTO = `mailto:${PRIVACY_EMAIL}?subject=GDPR%20data%20request`;

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
    return (
        <LegalContainer ambient>
            {/* Hero */}
            <header className="relative z-1 mb-12 flex flex-col items-center text-center">
                <div className="mb-5.5 inline-flex size-18 items-center justify-center rounded-full bg-primary/10 text-primary">
                    <ShieldIcon className="size-10" strokeWidth={1.8} />
                </div>
                <h1 className="m-0 mb-3.5 text-balance font-extrabold text-[clamp(36px,5vw,52px)] text-foreground leading-[1.05] tracking-[-0.02em]">Privacy Policy</h1>
                <p className="m-0 max-w-[56ch] font-sans text-[20px] text-muted-foreground leading-[1.55]">Your privacy matters. Here's how we protect and handle your personal information.</p>
                <div className="mt-5.5 flex flex-wrap items-center justify-center gap-2">
                    <Badge variant="secondary" size="lg">
                        Effective: {EFFECTIVE_DATE}
                    </Badge>
                    <Badge variant="outline" size="lg">
                        {VERSION}
                    </Badge>
                </div>
            </header>

            {/* TL;DR */}
            <Alert className="mb-10 border-[color-mix(in_srgb,var(--primary)_32%,transparent)] bg-[color-mix(in_srgb,var(--primary)_5%,transparent)] [&>svg]:text-primary">
                <Eye strokeWidth={2} />
                <AlertTitle className="text-[15px] text-foreground">TL;DR - Quick Summary</AlertTitle>
                <AlertDescription className="text-[15px] text-muted-foreground leading-[1.55]">
                    We use Yostar OAuth to sync your game data - we never see your password and don't store your email address. We collect only what's needed to provide our tools. We never sell your information. You control your profile visibility and leaderboard participation. You can delete your account and all data
                    anytime.
                </AlertDescription>
            </Alert>

            {/* Principles */}
            <section className="my-12">
                <h2 className="m-0 mb-5 text-center font-semibold text-[26px] text-foreground leading-[1.2] tracking-[-0.02em]">Our Privacy Principles</h2>
                <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-2 lg:grid-cols-3">
                    <Tile icon={<Lock strokeWidth={1.8} />} title="Minimal Collection" desc="We only collect what we absolutely need to provide our services." />
                    <Tile icon={<Eye strokeWidth={1.8} />} title="Full Transparency" desc="We're upfront about what data we collect and why we collect it." />
                    <Tile icon={<UserCog strokeWidth={1.8} />} title="Your Control" desc="You decide what's visible and can delete your data at any time." />
                </div>
            </section>

            <LegalDivider />

            {/* Yostar OAuth */}
            <Section id="yostar">
                <SectionHead icon={<KeyRound className="size-5" strokeWidth={1.8} />} title="Yostar Account Integration" subtitle="How we sync your Arknights game data" />
                <div className="pl-13 max-sm:pl-0">
                    <P>Myrtle uses Yostar's official OAuth system to sync your Arknights account data. Here's how it works:</P>

                    <Callout tone="primary">
                        <CalloutTitle>How Authentication Works</CalloutTitle>
                        <CalloutBody>
                            When you log in, you enter your email address in our app. Yostar sends a verification code directly to your email. You enter that code to authenticate. We never see or store your Yostar password - authentication is handled entirely by Yostar's servers. Your email address is only used during
                            the authentication process and is not stored in our database.
                        </CalloutBody>
                    </Callout>

                    <Callout>
                        <CalloutTitle>What We Receive from Yostar</CalloutTitle>
                        <CalloutBody>
                            Upon successful authentication, Yostar provides us with a session token that allows us to fetch your public game data: operator roster, levels, promotions, skill masteries, modules, stage progress, base layout, inventory, and account statistics. This is the same data visible in your in-game
                            profile.
                        </CalloutBody>
                    </Callout>

                    <p className="m-0 mt-4 font-sans text-[14px] text-muted-foreground leading-[1.6]">
                        We are not affiliated with Hypergryph or Yostar. We access your data through the same APIs the official game client uses. You can revoke our access at any time by requesting deletion of your Myrtle account - email <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>.
                    </p>
                </div>
            </Section>

            <LegalDivider />

            {/* Information We Collect */}
            <Section id="information-we-collect">
                <SectionHead icon={<Database className="size-5" strokeWidth={1.8} />} title="Information We Collect" subtitle="Understanding what data flows through our system" />
                <div className="pl-13 max-sm:pl-0">
                    <H3>Account Information</H3>
                    <p className="m-0 mb-2 font-sans text-[15px] text-muted-foreground leading-[1.6]">When you create an account via Yostar OAuth, we store:</p>
                    <BulletList
                        items={[
                            { label: "Arknights UID and nickname", body: "- your in-game identifier and display name" },
                            { label: "Profile settings", body: "- your Myrtle preferences: theme, accent color, profile visibility, leaderboard opt-in, and notification settings" },
                        ]}
                    />
                    <Callout tone="success">
                        <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.6]">
                            <strong className="text-success-foreground">Note:</strong> We do not store your email address. It is only used during the Yostar authentication process to receive your verification code and is not saved to our database.
                        </p>
                    </Callout>

                    <H3>Game Data (Synced from Arknights)</H3>
                    <p className="m-0 mb-2 font-sans text-[15px] text-muted-foreground leading-[1.6]">When you sync your account, we fetch and store:</p>
                    <BulletList
                        items={[
                            { label: "Operator roster", body: "- all operators you own, including level, promotion, trust, potential, skill levels, masteries, and equipped modules" },
                            { label: "Stage progress", body: "- mainline, sidestory, and activity stage completion status" },
                            { label: "Roguelike & Sandbox progress", body: "- Integrated Strategies themes, endings, buffs, and Reclamation Algorithm data" },
                            { label: "Base layout", body: "- RIIC building configuration and efficiency data" },
                            { label: "Medals and achievements", body: "- your collection of in-game medals" },
                        ]}
                    />

                    <H3>Saved Configurations</H3>
                    <BulletList items={[{ label: "DPS calculator configurations", body: "- saved operator setups and comparison configurations" }]} />

                    <H3>Technical Data</H3>
                    <BulletList items={[{ label: "Error logs", body: "- diagnostic data to help us fix bugs, stored temporarily" }]} />
                    <Callout tone="success">
                        <p className="m-0 font-sans text-[13.5px] text-muted-foreground leading-[1.6]">
                            <strong className="text-success-foreground">Note:</strong> Your IP address is not collected or stored. All API requests are routed through our server, which handles communication with our backend internally. Rate limiting is applied at the server level, not based on individual user IPs.
                        </p>
                    </Callout>
                </div>
            </Section>

            <LegalDivider />

            {/* How we use info */}
            <Section id="how-we-use">
                <SectionHead icon={<Eye className="size-5" strokeWidth={1.8} />} title="How We Use Your Information" subtitle="The purposes behind our data collection" />
                <div className="pl-13 max-sm:pl-0">
                    <Disclosure summary="Service Provision" defaultOpen>
                        To operate and maintain your account, sync your data across devices, and provide the core functionality of our tools and calculators. This includes displaying your operator roster, calculating account scores, and enabling profile features.
                    </Disclosure>
                    <Disclosure summary="Leaderboards & Community Features">
                        If you opt in to public visibility, your account scores and rankings may appear on our leaderboards. This allows the community to compare account progress across multiple dimensions (operators, stages, roguelike, sandbox, medals, base). You can opt out at any time in your settings.
                    </Disclosure>
                    <Disclosure summary="Improvement & Development">To understand how users interact with features, identify bugs, optimize performance, and develop new functionality based on usage patterns. We use aggregated, anonymized data for these purposes.</Disclosure>
                    <Disclosure summary="Security & Abuse Prevention">To protect against unauthorized access, detect suspicious activity, and maintain the integrity of our platform. Rate limiting is applied at the server level to prevent abuse - your individual IP address is not tracked or stored.</Disclosure>
                </div>
            </Section>

            <LegalDivider />

            {/* Settings & visibility */}
            <Section id="settings">
                <SectionHead icon={<SettingsIcon className="size-5" strokeWidth={1.8} />} title="User Settings & Profile Visibility" subtitle="Control how your information is displayed" />
                <div className="pl-13 max-sm:pl-0">
                    <P>Your settings page gives you full control over how your data is shared with others:</P>
                    <div className="my-4 grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                        <Callout>
                            <CalloutTitle withIcon={<Eye className="size-4 text-primary" strokeWidth={1.8} />}>Profile Visibility</CalloutTitle>
                            <CalloutBody>Choose whether your profile is public (anyone can view), friends-only, or completely private.</CalloutBody>
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
                                Leaderboard Participation
                            </CalloutTitle>
                            <CalloutBody>Opt in or out of appearing on public leaderboards. Your scores are still calculated but won't be displayed publicly if you opt out.</CalloutBody>
                        </Callout>
                    </div>
                    <Callout tone="primary">
                        <CalloutTitle>What's Publicly Visible (when profile is public)</CalloutTitle>
                        <ul className="m-0 list-disc pl-5 font-sans text-[13.5px] text-muted-foreground leading-[1.65]">
                            <li className="my-1">Your Arknights nickname and UID</li>
                            <li className="my-1">Your operator roster and account scores</li>
                            <li className="my-1">Your leaderboard rankings (if opted in)</li>
                            <li className="my-1">Your profile avatar and selected assistant</li>
                        </ul>
                    </Callout>
                    <Callout>
                        <CalloutTitle>What's Always Private</CalloutTitle>
                        <ul className="m-0 list-disc pl-5 font-sans text-[13.5px] text-muted-foreground leading-[1.65]">
                            <li className="my-1">Your authentication tokens and session data</li>
                            <li className="my-1">Your settings and preferences</li>
                            <li className="my-1">Your saved DPS calculator configurations</li>
                            <li className="my-1">Your email and authentication methods</li>
                        </ul>
                    </Callout>
                </div>
            </Section>

            <LegalDivider />

            {/* Security */}
            <Section id="security">
                <SectionHead icon={<Lock className="size-5" strokeWidth={1.8} />} title="Data Security & Storage" subtitle="How we protect your information" />
                <div className="pl-13 max-sm:pl-0">
                    <P>We implement security measures to protect your personal information:</P>
                    <div className="my-4 grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                        <Callout tone="primary">
                            <CalloutTitle>Encryption</CalloutTitle>
                            <CalloutBody>All data transmitted over HTTPS with TLS encryption</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>JWT Authentication</CalloutTitle>
                            <CalloutBody>Secure token-based sessions with automatic expiration</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>Server-Side Rate Limiting</CalloutTitle>
                            <CalloutBody>Rate limits applied at server level without tracking user IPs</CalloutBody>
                        </Callout>
                        <Callout tone="primary">
                            <CalloutTitle>Redis Caching</CalloutTitle>
                            <CalloutBody>Static data cached with 1-hour TTL for performance</CalloutBody>
                        </Callout>
                    </div>
                    <Callout>
                        <CalloutTitle>Database</CalloutTitle>
                        <CalloutBody>Your data is stored in PostgreSQL with proper constraints and transactions. Game data is fetched fresh from Yostar's servers when you sync - we store a copy so you can access your profile and leaderboard features without re-authenticating.</CalloutBody>
                    </Callout>
                    <p className="m-0 mt-4 font-sans text-[14px] text-muted-foreground leading-[1.6]">
                        While we strive to protect your information, no method of transmission over the internet is 100% secure. Our code is open source on{" "}
                        <A href={REPO_URL} external>
                            GitHub
                        </A>
                        , so you can review our security practices.
                    </p>
                </div>
            </Section>

            <LegalDivider />

            {/* Rights */}
            <Section id="rights">
                <SectionHead icon={<UserCog className="size-5" strokeWidth={1.8} />} title="Your Rights & Choices" subtitle="Control over your personal data" />
                <div className="pl-13 max-sm:pl-0">
                    <P>You have the following rights regarding your personal information:</P>
                    <Disclosure summary="Access & Export">Request a copy of all data we hold about you in a portable format. You can view most of your data directly on your profile page, including your synced game data, scores, and settings.</Disclosure>
                    <Disclosure summary="Correction & Re-sync">Update your preferences through your account settings. If your game data is out of date, you can re-sync at any time to fetch the latest information from Yostar's servers.</Disclosure>
                    <Disclosure summary="Visibility Control">Choose who can see your profile and roster (public, friends-only, or private). Control whether you appear on public leaderboards. All visibility settings can be changed at any time in your settings page.</Disclosure>
                    <Disclosure summary="Account Deletion">
                        Request permanent deletion of your account and all associated data. This removes your profile, synced game data, scores, and any saved configurations. This action cannot be undone. To request deletion, email <A href={CONTACT_MAILTO}>{CONTACT_EMAIL}</A>.
                    </Disclosure>
                </div>
            </Section>

            <LegalDivider />

            {/* Third-party */}
            <Section id="third-party">
                <H2Plain>Third-Party Services</H2Plain>
                <P>We interact with the following third-party services:</P>
                <UL>
                    <LI>
                        <strong>Yostar / Hypergryph:</strong> We use Yostar's OAuth system to authenticate you and fetch your game data. We are not affiliated with Yostar or Hypergryph - Myrtle is an independent fan project.
                    </LI>
                    <LI>
                        <strong>Arknights game servers:</strong> We fetch game data from official servers (EN, JP, KR, CN, TW, Bilibili) to provide up-to-date operator information, assets, and your synced account data.
                    </LI>
                    <LI>
                        <strong>Hosting infrastructure:</strong> Our servers and databases are hosted on secure cloud infrastructure.
                    </LI>
                </UL>
                <P muted>We do not sell, trade, or rent your personal information to third parties. We do not run ads or use your data for marketing.</P>
            </Section>

            <LegalDivider />

            {/* Children */}
            <Section id="children">
                <H2Plain>Children's Privacy</H2Plain>
                <P>Our Service is not directed to individuals under the age of 13. We do not knowingly collect personal information from children under 13. If you become aware that a child has provided us with personal information, please contact us and we will take steps to delete such information.</P>
            </Section>

            <LegalDivider />

            {/* Changes */}
            <Section id="changes">
                <H2Plain>Changes to This Policy</H2Plain>
                <P>We may update this Privacy Policy from time to time to reflect changes in our practices or legal requirements. We will notify you of any material changes by posting the new policy on this page and updating the "Effective Date" at the top.</P>
                <P>We encourage you to review this Privacy Policy periodically. Your continued use of the Service after changes are posted constitutes your acceptance of the updated policy.</P>
            </Section>

            <LegalDivider />

            {/* Contact */}
            <Section id="contact">
                <SectionHead icon={<Mail className="size-5" strokeWidth={1.8} />} title="Contact Us" subtitle="Questions about your privacy?" />
                <div className="pl-13 max-sm:pl-0">
                    <div className="rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_50%,var(--card))] px-6 py-5">
                        <p className="m-0 font-sans font-semibold text-[15px] text-foreground">Get in Touch</p>
                        <p className="m-0 mt-1.5 mb-4 font-sans text-[14px] text-muted-foreground leading-[1.55]">If you have any questions, concerns, or requests regarding this Privacy Policy or your personal data, please contact us:</p>
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
                            <Link to="/settings" className="font-sans text-[14px] text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
                                Manage your privacy in settings
                            </Link>
                        </div>
                    </div>
                </div>
            </Section>

            {/* Footer CTA */}
            <div className="my-12 rounded-2xl border border-border bg-linear-to-b from-[color-mix(in_srgb,var(--muted)_50%,transparent)] to-transparent p-8 text-center">
                <h3 className="m-0 mb-2 font-semibold text-[22px] text-foreground leading-[1.3] tracking-[-0.01em]">Your Privacy is Our Priority</h3>
                <p className="m-0 mx-auto max-w-[56ch] text-balance font-sans text-[15px] text-muted-foreground leading-[1.55]">We're committed to transparency and giving you control over your data. If you have any questions or concerns, we're here to help.</p>
                <div className="mt-5 flex flex-wrap items-center justify-center gap-2.5">
                    <Button size="lg" render={<Link to="/settings" />}>
                        Manage your privacy settings
                    </Button>
                    <Button variant="outline" size="lg" render={<Link to="/" />}>
                        Return home
                    </Button>
                </div>
            </div>

            <RelatedLinksFooter>
                <RelatedDocLink to="/terms" label="Terms of Service" />
            </RelatedLinksFooter>
        </LegalContainer>
    );
}
