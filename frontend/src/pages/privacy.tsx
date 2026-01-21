"use client";

import { ChevronDown, Database, Eye, Gamepad2, Globe, Lock, Mail, Settings, Shield, Trophy, UserX } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import { SEO } from "~/components/seo";
import { TextEffect } from "~/components/ui/motion-primitives/text-effect";
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "~/components/ui/shadcn/accordion";
import { Alert, AlertDescription, AlertTitle } from "~/components/ui/shadcn/alert";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "~/components/ui/shadcn/collapsible";
import { Separator } from "~/components/ui/shadcn/separator";

export default function PrivacyPage() {
    const [openRights, setOpenRights] = useState<string[]>([]);

    const toggleRight = (id: string) => {
        setOpenRights((prev) => (prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id]));
    };

    return (
        <>
            <SEO
                description="Learn how myrtle.moe protects your privacy. We use Yostar OAuth for secure authentication, never store your email or password, and give you full control over your data."
                keywords={["privacy policy", "data protection", "Yostar OAuth", "account security"]}
                path="/privacy"
                title="Privacy Policy"
            />
            <article className="mx-auto max-w-4xl">
                {/* Hero Section */}
                <header className="mb-12 text-center">
                    <div className="mb-6 inline-flex items-center justify-center rounded-full bg-primary/10 p-4">
                        <Shield className="h-12 w-12 text-primary" />
                    </div>
                    <TextEffect as="h1" className="mb-4 font-extrabold text-5xl tracking-tight md:text-6xl" per="word">
                        Privacy Policy
                    </TextEffect>
                    <p className="mx-auto max-w-2xl text-balance text-muted-foreground text-xl leading-relaxed">Your privacy matters. Here's how we protect and handle your personal information.</p>
                    <div className="mt-6 flex items-center justify-center gap-3">
                        <Badge variant="secondary">Effective: Jan 12, 2026</Badge>
                        <Badge variant="outline">Version 2.0</Badge>
                    </div>
                </header>

                {/* Quick Summary Alert */}
                <Alert className="mb-10 border-primary/50 bg-primary/5">
                    <Eye className="h-4 w-4" />
                    <AlertTitle className="font-semibold text-base">TL;DR - Quick Summary</AlertTitle>
                    <AlertDescription className="mt-2 leading-relaxed">
                        We use Yostar OAuth to sync your game data - we never see your password and don't store your email address. We collect only what's needed to provide our tools. We never sell your information. You control your profile visibility and leaderboard participation. You can delete your account and all
                        data anytime.
                    </AlertDescription>
                </Alert>

                {/* Key Principles */}
                <section className="mb-12">
                    <h2 className="mb-6 text-center font-semibold text-2xl tracking-tight">Our Privacy Principles</h2>
                    <div className="grid gap-4 sm:grid-cols-3">
                        <Card className="border-border/50 bg-linear-to-br from-background to-muted/30">
                            <CardHeader className="pb-3">
                                <Lock className="mb-2 h-6 w-6 text-primary" />
                                <CardTitle className="text-base">Minimal Collection</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <CardDescription className="text-sm leading-relaxed">We only collect what we absolutely need to provide our services.</CardDescription>
                            </CardContent>
                        </Card>
                        <Card className="border-border/50 bg-linear-to-br from-background to-muted/30">
                            <CardHeader className="pb-3">
                                <Eye className="mb-2 h-6 w-6 text-primary" />
                                <CardTitle className="text-base">Full Transparency</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <CardDescription className="text-sm leading-relaxed">We're upfront about what data we collect and why we collect it.</CardDescription>
                            </CardContent>
                        </Card>
                        <Card className="border-border/50 bg-linear-to-br from-background to-muted/30">
                            <CardHeader className="pb-3">
                                <UserX className="mb-2 h-6 w-6 text-primary" />
                                <CardTitle className="text-base">Your Control</CardTitle>
                            </CardHeader>
                            <CardContent>
                                <CardDescription className="text-sm leading-relaxed">You decide what's visible and can delete your data at any time.</CardDescription>
                            </CardContent>
                        </Card>
                    </div>
                </section>

                <Separator className="my-10" />

                {/* Main Content */}
                <div className="space-y-12">
                    {/* Yostar OAuth Section */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Gamepad2 className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">Yostar Account Integration</h2>
                                <p className="text-muted-foreground text-sm">How we sync your Arknights game data</p>
                            </div>
                        </div>

                        <div className="space-y-4 pl-14">
                            <p className="leading-7">Myrtle uses Yostar's official OAuth system to sync your Arknights account data. Here's how it works:</p>
                            <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                <h4 className="mb-2 font-semibold">How Authentication Works</h4>
                                <p className="mb-3 text-muted-foreground text-sm leading-relaxed">
                                    When you log in, you enter your email address in our app. Yostar sends a verification code directly to your email. You enter that code to authenticate. We never see or store your Yostar password - authentication is handled entirely by Yostar's servers. Your email address is only used
                                    during the authentication process and is not stored in our database.
                                </p>
                            </div>
                            <div className="rounded-lg border border-border/50 bg-muted/20 p-4">
                                <h4 className="mb-2 font-semibold">What We Receive from Yostar</h4>
                                <p className="text-muted-foreground text-sm leading-relaxed">
                                    Upon successful authentication, Yostar provides us with a session token that allows us to fetch your public game data: operator roster, levels, promotions, skill masteries, modules, stage progress, base layout, inventory, and account statistics. This is the same data visible in your
                                    in-game profile.
                                </p>
                            </div>
                            <p className="text-muted-foreground text-sm leading-relaxed">
                                We are not affiliated with Hypergryph or Yostar. We access your data through the same APIs the official game client uses. You can revoke our access at any time by requesting a deletion of your Myrtle account via the{" "}
                                <Link className="text-blue-400 hover:underline" href={"/discord"} target="_blank">
                                    Discord
                                </Link>{" "}
                                server.
                            </p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 1 - Information We Collect */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Database className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">Information We Collect</h2>
                                <p className="text-muted-foreground text-sm">Understanding what data flows through our system</p>
                            </div>
                        </div>

                        <div className="space-y-6 pl-14">
                            <div>
                                <h3 className="mb-2 font-semibold text-lg tracking-tight">Account Information</h3>
                                <p className="mb-3 text-muted-foreground leading-7">When you create an account via Yostar OAuth, we store:</p>
                                <ul className="space-y-2">
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Arknights UID and nickname</strong> - Your in-game identifier and display name
                                        </span>
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Profile settings</strong> - Your Myrtle preferences: theme, accent color, profile visibility, leaderboard opt-in, and notification settings
                                        </span>
                                    </li>
                                </ul>
                                <div className="mt-3 rounded-lg border border-green-500/20 bg-green-500/5 p-3">
                                    <p className="text-muted-foreground text-sm leading-relaxed">
                                        <strong className="text-green-600 dark:text-green-400">Note:</strong> We do not store your email address. It is only used during the Yostar authentication process to receive your verification code and is not saved to our database.
                                    </p>
                                </div>
                            </div>

                            <div>
                                <h3 className="mb-2 font-semibold text-lg tracking-tight">Game Data (Synced from Arknights)</h3>
                                <p className="mb-3 text-muted-foreground leading-7">When you sync your account, we fetch and store:</p>
                                <ul className="space-y-2">
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Operator roster</strong> - All operators you own, including level, promotion, trust, potential, skill levels, masteries, and equipped modules
                                        </span>
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Stage progress</strong> - Mainline, sidestory, and activity stage completion status
                                        </span>
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Roguelike & Sandbox progress</strong> - Integrated Strategies themes, endings, buffs, and Reclamation Algorithm data
                                        </span>
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Base layout</strong> - RIIC building configuration and efficiency data
                                        </span>
                                    </li>
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Medals and achievements</strong> - Your collection of in-game medals
                                        </span>
                                    </li>
                                </ul>
                            </div>

                            <div>
                                <h3 className="mb-2 font-semibold text-lg tracking-tight">Saved Configurations</h3>
                                <p className="mb-3 text-muted-foreground leading-7">Content you save on Myrtle:</p>
                                <ul className="space-y-2">
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>DPS calculator configurations</strong> - Saved operator configurations and comparison setups
                                        </span>
                                    </li>
                                </ul>
                            </div>

                            <div>
                                <h3 className="mb-2 font-semibold text-lg tracking-tight">Technical Data</h3>
                                <p className="mb-3 text-muted-foreground leading-7">We may temporarily process:</p>
                                <ul className="space-y-2">
                                    <li className="flex items-start gap-2">
                                        <div className="mt-0.5 flex h-4 w-4 shrink-0 items-center justify-center rounded-full bg-primary/20">
                                            <div className="h-1.5 w-1.5 rounded-full bg-primary" />
                                        </div>
                                        <span className="leading-relaxed">
                                            <strong>Error logs</strong> - Diagnostic data to help us fix bugs, stored temporarily
                                        </span>
                                    </li>
                                </ul>
                                <div className="mt-3 rounded-lg border border-green-500/20 bg-green-500/5 p-3">
                                    <p className="text-muted-foreground text-sm leading-relaxed">
                                        <strong className="text-green-600 dark:text-green-400">Note:</strong> Your IP address is not collected or stored. All API requests are routed through our Next.js server, which handles communication with our backend internally. Rate limiting is applied at the server level, not
                                        based on individual user IPs.
                                    </p>
                                </div>
                            </div>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 2 - How We Use Your Information (Accordion) */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Eye className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">How We Use Your Information</h2>
                                <p className="text-muted-foreground text-sm">The purposes behind our data collection</p>
                            </div>
                        </div>

                        <div className="pl-14">
                            <Accordion className="space-y-2" type="multiple">
                                <AccordionItem className="rounded-lg border border-border/50 bg-muted/20 px-4" value="service">
                                    <AccordionTrigger className="hover:no-underline">
                                        <span className="font-medium">Service Provision</span>
                                    </AccordionTrigger>
                                    <AccordionContent>
                                        <p className="text-muted-foreground text-sm leading-relaxed">
                                            To operate and maintain your account, sync your data across devices, and provide the core functionality of our tools and calculators. This includes displaying your operator roster, calculating account scores, and enabling profile features.
                                        </p>
                                    </AccordionContent>
                                </AccordionItem>

                                <AccordionItem className="rounded-lg border border-border/50 bg-muted/20 px-4" value="leaderboards">
                                    <AccordionTrigger className="hover:no-underline">
                                        <span className="font-medium">Leaderboards & Community Features</span>
                                    </AccordionTrigger>
                                    <AccordionContent>
                                        <p className="text-muted-foreground text-sm leading-relaxed">
                                            If you opt in to public visibility, your account scores and rankings may appear on our leaderboards. This allows the community to compare account progress across multiple dimensions (operators, stages, roguelike, sandbox, medals, base). You can opt out at any time in your
                                            settings.
                                        </p>
                                    </AccordionContent>
                                </AccordionItem>

                                <AccordionItem className="rounded-lg border border-border/50 bg-muted/20 px-4" value="improvement">
                                    <AccordionTrigger className="hover:no-underline">
                                        <span className="font-medium">Improvement & Development</span>
                                    </AccordionTrigger>
                                    <AccordionContent>
                                        <p className="text-muted-foreground text-sm leading-relaxed">To understand how users interact with features, identify bugs, optimize performance, and develop new functionality based on usage patterns. We use aggregated, anonymized data for these purposes.</p>
                                    </AccordionContent>
                                </AccordionItem>

                                <AccordionItem className="rounded-lg border border-border/50 bg-muted/20 px-4" value="security">
                                    <AccordionTrigger className="hover:no-underline">
                                        <span className="font-medium">Security & Abuse Prevention</span>
                                    </AccordionTrigger>
                                    <AccordionContent>
                                        <p className="text-muted-foreground text-sm leading-relaxed">
                                            To protect against unauthorized access, detect suspicious activity, and maintain the integrity of our platform. Rate limiting is applied at the server level to prevent abuse - your individual IP address is not tracked or stored.
                                        </p>
                                    </AccordionContent>
                                </AccordionItem>
                            </Accordion>
                        </div>
                    </section>

                    <Separator />

                    {/* User Settings & Visibility Section */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Settings className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">User Settings & Profile Visibility</h2>
                                <p className="text-muted-foreground text-sm">Control how your information is displayed</p>
                            </div>
                        </div>

                        <div className="space-y-4 pl-14">
                            <p className="leading-7">Your settings page gives you full control over how your data is shared with others:</p>

                            <div className="grid gap-3 sm:grid-cols-2">
                                <div className="rounded-lg border border-border/50 bg-muted/20 p-4">
                                    <div className="mb-2 flex items-center gap-2">
                                        <Globe className="h-4 w-4 text-primary" />
                                        <p className="font-medium text-sm">Profile Visibility</p>
                                    </div>
                                    <p className="text-muted-foreground text-sm">Choose whether your profile is public (anyone can view), friends-only, or completely private.</p>
                                </div>
                                <div className="rounded-lg border border-border/50 bg-muted/20 p-4">
                                    <div className="mb-2 flex items-center gap-2">
                                        <Trophy className="h-4 w-4 text-primary" />
                                        <p className="font-medium text-sm">Leaderboard Participation</p>
                                    </div>
                                    <p className="text-muted-foreground text-sm">Opt in or out of appearing on public leaderboards. Your scores are still calculated but won't be displayed publicly if you opt out.</p>
                                </div>
                            </div>

                            <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                <h4 className="mb-2 font-semibold text-sm">What's Publicly Visible (When Profile is Public)</h4>
                                <ul className="space-y-1 text-muted-foreground text-sm">
                                    <li>• Your Arknights nickname and UID</li>
                                    <li>• Your operator roster and account scores</li>
                                    <li>• Your leaderboard rankings (if opted in)</li>
                                    <li>• Your profile avatar and selected assistant</li>
                                </ul>
                            </div>

                            <div className="rounded-lg border border-border/50 bg-muted/20 p-4">
                                <h4 className="mb-2 font-semibold text-sm">What's Always Private</h4>
                                <ul className="space-y-1 text-muted-foreground text-sm">
                                    <li>• Your authentication tokens and session data</li>
                                    <li>• Your settings and preferences</li>
                                    <li>• Your saved DPS calculator configurations</li>
                                </ul>
                            </div>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 3 - Data Security */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Lock className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">Data Security & Storage</h2>
                                <p className="text-muted-foreground text-sm">How we protect your information</p>
                            </div>
                        </div>

                        <div className="space-y-4 pl-14">
                            <p className="leading-7">We implement security measures to protect your personal information:</p>
                            <div className="grid gap-3 sm:grid-cols-2">
                                <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                    <p className="mb-1 font-medium text-sm">Encryption</p>
                                    <p className="text-muted-foreground text-sm">All data transmitted over HTTPS with TLS encryption</p>
                                </div>
                                <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                    <p className="mb-1 font-medium text-sm">JWT Authentication</p>
                                    <p className="text-muted-foreground text-sm">Secure token-based sessions with automatic expiration</p>
                                </div>
                                <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                    <p className="mb-1 font-medium text-sm">Server-Side Rate Limiting</p>
                                    <p className="text-muted-foreground text-sm">Rate limits applied at server level without tracking user IPs</p>
                                </div>
                                <div className="rounded-lg border border-primary/20 bg-primary/5 p-4">
                                    <p className="mb-1 font-medium text-sm">Redis Caching</p>
                                    <p className="text-muted-foreground text-sm">Static data cached with 1-hour TTL for performance</p>
                                </div>
                            </div>
                            <div className="rounded-lg border border-border/50 bg-muted/20 p-4">
                                <h4 className="mb-2 font-semibold text-sm">Database</h4>
                                <p className="text-muted-foreground text-sm leading-relaxed">
                                    Your data is stored in PostgreSQL with proper constraints and transactions. Game data is fetched fresh from Yostar's servers when you sync - we store a copy so you can access your profile and leaderboard features without re-authenticating.
                                </p>
                            </div>
                            <p className="text-muted-foreground text-sm leading-relaxed">While we strive to protect your information, no method of transmission over the internet is 100% secure. Our code is open source on GitHub, so you can review our security practices.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 4 - Your Rights & Choices (Collapsible with different design) */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <UserX className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">Your Rights & Choices</h2>
                                <p className="text-muted-foreground text-sm">Control over your personal data</p>
                            </div>
                        </div>

                        <div className="space-y-3 pl-14">
                            <p className="mb-4 leading-7">You have the following rights regarding your personal information:</p>

                            <Collapsible onOpenChange={() => toggleRight("access")} open={openRights.includes("access")}>
                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg bg-muted/50 p-4 transition-colors hover:bg-muted/70">
                                    <h4 className="font-semibold text-base">Access & Export</h4>
                                    <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openRights.includes("access") ? "rotate-180" : ""}`} />
                                </CollapsibleTrigger>
                                <CollapsibleContent className="px-4 pt-2 pb-4">
                                    <p className="text-muted-foreground text-sm leading-relaxed">Request a copy of all data we hold about you in a portable format. You can view most of your data directly on your profile page, including your synced game data, scores, and settings.</p>
                                </CollapsibleContent>
                            </Collapsible>

                            <Collapsible onOpenChange={() => toggleRight("correction")} open={openRights.includes("correction")}>
                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg bg-muted/50 p-4 transition-colors hover:bg-muted/70">
                                    <h4 className="font-semibold text-base">Correction & Re-sync</h4>
                                    <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openRights.includes("correction") ? "rotate-180" : ""}`} />
                                </CollapsibleTrigger>
                                <CollapsibleContent className="px-4 pt-2 pb-4">
                                    <p className="text-muted-foreground text-sm leading-relaxed">Update your preferences through your account settings. If your game data is out of date, you can re-sync at any time to fetch the latest information from Yostar's servers.</p>
                                </CollapsibleContent>
                            </Collapsible>

                            <Collapsible onOpenChange={() => toggleRight("visibility")} open={openRights.includes("visibility")}>
                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg bg-muted/50 p-4 transition-colors hover:bg-muted/70">
                                    <h4 className="font-semibold text-base">Visibility Control</h4>
                                    <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openRights.includes("visibility") ? "rotate-180" : ""}`} />
                                </CollapsibleTrigger>
                                <CollapsibleContent className="px-4 pt-2 pb-4">
                                    <p className="text-muted-foreground text-sm leading-relaxed">Choose who can see your profile and roster (public, friends-only, or private). Control whether you appear on public leaderboards. All visibility settings can be changed at any time in your settings page.</p>
                                </CollapsibleContent>
                            </Collapsible>

                            <Collapsible onOpenChange={() => toggleRight("deletion")} open={openRights.includes("deletion")}>
                                <CollapsibleTrigger className="flex w-full items-center justify-between rounded-lg bg-muted/50 p-4 transition-colors hover:bg-muted/70">
                                    <h4 className="font-semibold text-base">Account Deletion</h4>
                                    <ChevronDown className={`h-4 w-4 text-muted-foreground transition-transform duration-200 ${openRights.includes("deletion") ? "rotate-180" : ""}`} />
                                </CollapsibleTrigger>
                                <CollapsibleContent className="px-4 pt-2 pb-4">
                                    <p className="text-muted-foreground text-sm leading-relaxed">
                                        Request permanent deletion of your account and all associated data. This removes your profile, synced game data, scores, and any saved configurations. This action cannot be undone. To request deletion, contact us via{" "}
                                        <Link className="text-blue-400 hover:underline" href="/discord" target="_blank">
                                            Discord
                                        </Link>
                                        .
                                    </p>
                                </CollapsibleContent>
                            </Collapsible>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 5 */}
                    <section>
                        <h2 className="mb-4 scroll-m-20 font-semibold text-2xl tracking-tight">Third-Party Services</h2>
                        <div className="space-y-4">
                            <p className="leading-7">We interact with the following third-party services:</p>
                            <ul className="ml-6 list-disc space-y-2 [&>li]:leading-relaxed">
                                <li>
                                    <strong>Yostar/Hypergryph:</strong> We use Yostar's OAuth system to authenticate you and fetch your game data. We are not affiliated with Yostar or Hypergryph - Myrtle is an independent fan project.
                                </li>
                                <li>
                                    <strong>Arknights Game Servers:</strong> We fetch game data from official servers (EN, JP, KR, CN, TW, Bilibili) to provide up-to-date operator information, assets, and your synced account data.
                                </li>
                                <li>
                                    <strong>Hosting Infrastructure:</strong> Our servers and databases are hosted on secure cloud infrastructure.
                                </li>
                            </ul>
                            <p className="text-muted-foreground text-sm leading-7">We do not sell, trade, or rent your personal information to third parties. We do not run ads or use your data for marketing.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 6 */}
                    <section>
                        <h2 className="mb-4 scroll-m-20 font-semibold text-2xl tracking-tight">Children's Privacy</h2>
                        <p className="leading-7">
                            Our Service is not directed to individuals under the age of 13. We do not knowingly collect personal information from children under 13. If you become aware that a child has provided us with personal information, please contact us and we will take steps to delete such information.
                        </p>
                    </section>

                    <Separator />

                    {/* Section 7 */}
                    <section>
                        <h2 className="mb-4 scroll-m-20 font-semibold text-2xl tracking-tight">Changes to This Policy</h2>
                        <p className="mb-4 leading-7">We may update this Privacy Policy from time to time to reflect changes in our practices or legal requirements. We will notify you of any material changes by posting the new policy on this page and updating the "Effective Date" at the top.</p>
                        <p className="leading-7">We encourage you to review this Privacy Policy periodically. Your continued use of the Service after changes are posted constitutes your acceptance of the updated policy.</p>
                    </section>

                    <Separator />

                    {/* Contact Section */}
                    <section>
                        <div className="mb-6 flex items-start gap-4">
                            <div className="mt-1 flex h-10 w-10 shrink-0 items-center justify-center rounded-lg bg-primary/10">
                                <Mail className="h-5 w-5 text-primary" />
                            </div>
                            <div>
                                <h2 className="mb-2 scroll-m-20 font-semibold text-2xl tracking-tight">Contact Us</h2>
                                <p className="text-muted-foreground text-sm">Questions about your privacy?</p>
                            </div>
                        </div>

                        <div className="pl-14">
                            <Card>
                                <CardHeader>
                                    <CardTitle className="text-base">Get in Touch</CardTitle>
                                    <CardDescription>If you have any questions, concerns, or requests regarding this Privacy Policy or your personal data, please contact us:</CardDescription>
                                </CardHeader>
                                <CardContent className="space-y-2">
                                    <div>
                                        <p className="mb-1 font-medium text-sm">Discord</p>
                                        <Link className="text-primary transition-colors hover:underline" href="/discord" target="_blank">
                                            Join our Discord server
                                        </Link>
                                    </div>
                                    <div>
                                        <p className="mb-1 font-medium text-sm">Source Code</p>
                                        <Link className="text-primary transition-colors hover:underline" href="https://github.com/Eltik/myrtle.moe" target="_blank">
                                            github.com/Eltik/myrtle.moe
                                        </Link>
                                    </div>
                                </CardContent>
                            </Card>
                        </div>
                    </section>
                </div>

                {/* Footer CTA */}
                <div className="mt-16 text-center">
                    <div className="mb-6 rounded-2xl border border-border bg-linear-to-br from-muted/50 to-background p-8">
                        <h3 className="mb-2 font-semibold text-xl">Your Privacy is Our Priority</h3>
                        <p className="mx-auto max-w-2xl text-balance text-muted-foreground leading-relaxed">We're committed to transparency and giving you control over your data. If you have any questions or concerns, we're here to help.</p>
                    </div>
                    <div className="flex flex-col items-center justify-center gap-3 sm:flex-row">
                        <Button asChild size="lg">
                            <Link href="/my/settings">Manage Your Privacy Settings</Link>
                        </Button>
                        <Button asChild size="lg" variant="outline">
                            <Link href="/">Return Home</Link>
                        </Button>
                    </div>
                </div>
            </article>
        </>
    );
}
