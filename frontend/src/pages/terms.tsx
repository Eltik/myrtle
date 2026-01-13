"use client";

import { Calendar, FileText, Scale, Shield } from "lucide-react";
import Link from "next/link";
import { SEO } from "~/components/seo";
import { Alert, AlertDescription } from "~/components/ui/shadcn/alert";
import { Badge } from "~/components/ui/shadcn/badge";
import { Button } from "~/components/ui/shadcn/button";
import { Separator } from "~/components/ui/shadcn/separator";

export default function TermsPage() {
    return (
        <>
            <SEO
                description="Terms of Service for myrtle.moe - the legal agreement governing your use of our Arknights companion platform, including account registration, acceptable use, and user content policies."
                keywords={["terms of service", "legal", "user agreement", "acceptable use"]}
                path="/terms"
                title="Terms of Service"
            />
            <article className="mx-auto max-w-5xl">
                {/* Header */}
                <header className="mb-12 border-border border-b pb-8">
                    <div className="flex items-start justify-between gap-4">
                        <div>
                            <h1 className="mb-3 scroll-m-20 text-balance font-extrabold text-4xl tracking-tight md:text-5xl">Terms of Service</h1>
                            <p className="text-muted-foreground text-xl">Legal agreement governing your use of Myrtle</p>
                        </div>
                        <Badge className="mt-2" variant="outline">
                            Version 2.0
                        </Badge>
                    </div>
                    <div className="mt-6 flex items-center gap-4 text-muted-foreground text-sm">
                        <div className="flex items-center gap-2">
                            <Calendar className="h-4 w-4" />
                            <span>Effective: January 12, 2026</span>
                        </div>
                        <Separator className="h-4" orientation="vertical" />
                        <div className="flex items-center gap-2">
                            <FileText className="h-4 w-4" />
                            <span>Last Updated: January 12, 2026</span>
                        </div>
                    </div>
                </header>

                {/* Important Notice */}
                <Alert className="mb-10">
                    <Shield className="h-4 w-4" />
                    <AlertDescription className="text-base">By accessing or using Myrtle, you agree to be bound by these Terms of Service. Please read them carefully before continuing.</AlertDescription>
                </Alert>

                {/* Table of Contents */}
                <nav className="mb-12 rounded-lg border border-border bg-muted/50 p-6">
                    <h2 className="mb-4 font-semibold text-lg">Table of Contents</h2>
                    <ol className="ml-6 list-decimal space-y-2">
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#acceptance">
                                Acceptance of Terms
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#account">
                                Account Registration and Security
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#usage">
                                Acceptable Use Policy
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#content">
                                User Content and Submissions
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#intellectual">
                                Intellectual Property Rights
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#termination">
                                Termination and Suspension
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#liability">
                                Limitation of Liability
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#changes">
                                Changes to Terms
                            </a>
                        </li>
                        <li>
                            <a className="text-primary transition-colors hover:underline" href="#contact">
                                Contact Information
                            </a>
                        </li>
                    </ol>
                </nav>

                {/* Terms Content */}
                <div className="space-y-10">
                    {/* Section 1 */}
                    <section id="acceptance">
                        <div className="mb-4 flex items-center gap-3">
                            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                                <Scale className="h-5 w-5 text-primary" />
                            </div>
                            <h2 className="scroll-m-20 font-semibold text-3xl tracking-tight">1. Acceptance of Terms</h2>
                        </div>
                        <div className="ml-13 space-y-4">
                            <p className="leading-7">By accessing, browsing, or using the Myrtle platform ("Service"), you acknowledge that you have read, understood, and agree to be bound by these Terms of Service and all applicable laws and regulations.</p>
                            <p className="leading-7">If you do not agree with any part of these terms, you must not use our Service.</p>
                            <p className="leading-7">These terms constitute a legally binding agreement between you ("User," "you," or "your") and Myrtle ("we," "us," or "our"). Your continued use of the Service signifies your acceptance of these terms and any modifications thereto.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 2 */}
                    <section id="account">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">2. Account Registration and Security</h2>
                        <div className="space-y-4">
                            <h3 className="scroll-m-20 font-semibold text-xl tracking-tight">2.1 Account Creation via Yostar OAuth</h3>
                            <p className="leading-7">
                                To access account features like profile sync, tier lists, and leaderboards, you must authenticate using your Yostar account (the same account you use for Arknights). We use Yostar's email verification system—you will receive a verification code from Yostar to confirm your identity. We
                                never see or store your Yostar password.
                            </p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">2.2 Account Security</h3>
                            <p className="leading-7">You are responsible for:</p>
                            <ul className="my-4 ml-6 list-disc [&>li]:mt-2">
                                <li>Maintaining the security of your Yostar account and email</li>
                                <li>All activities that occur under your Myrtle account</li>
                                <li>Notifying us if you believe your account has been compromised</li>
                            </ul>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">2.3 Account Eligibility</h3>
                            <p className="leading-7">You must be at least 13 years old to use this Service. By creating an account, you represent that you meet this age requirement and have the legal capacity to enter into these Terms. You must also have a valid Yostar account with Arknights.</p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">2.4 Game Data Sync</h3>
                            <p className="leading-7">
                                When you sync your account, we fetch your Arknights game data (operator roster, stage progress, etc.) from Yostar's servers. This data is stored on our servers to power features like your profile page, account scoring, and leaderboards. You can re-sync at any time to update your data, or
                                delete your account to remove all stored data.
                            </p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 3 */}
                    <section id="usage">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">3. Acceptable Use Policy</h2>
                        <div className="space-y-4">
                            <p className="leading-7">You agree not to engage in any of the following prohibited activities:</p>
                            <ul className="my-4 ml-6 list-disc [&>li]:mt-2">
                                <li>Using the Service for any illegal or unauthorized purpose</li>
                                <li>Attempting to gain unauthorized access to any portion of the Service or related systems</li>
                                <li>Interfering with or disrupting the Service or servers connected to the Service</li>
                                <li>Uploading or transmitting viruses, malware, or any other malicious code</li>
                                <li>Scraping, crawling, or using automated systems to extract data without permission</li>
                                <li>Impersonating any person or entity, or falsely stating your affiliation with any person or entity</li>
                                <li>Harassing, threatening, or intimidating other users</li>
                                <li>Violating any applicable local, state, national, or international law</li>
                            </ul>
                            <p className="leading-7">We reserve the right to investigate and prosecute violations of any of the above to the fullest extent of the law.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 4 */}
                    <section id="content">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">4. User Content and Submissions</h2>
                        <div className="space-y-4">
                            <p className="leading-7">Our Service allows you to create and share content such as tier lists ("User Content"). By creating User Content, you grant us a worldwide, non-exclusive, royalty-free license to display and distribute such content in connection with the Service.</p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">4.1 Tier Lists</h3>
                            <p className="leading-7">You can create tier lists with full version control—every edit is logged and previous versions are preserved. You control the visibility (public, private, or shared with specific users) and can set permissions for who can edit your tier lists.</p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">4.2 Content Guidelines</h3>
                            <p className="leading-7">You represent and warrant that your User Content:</p>
                            <ul className="my-4 ml-6 list-disc [&>li]:mt-2">
                                <li>Is your original creation or you have the right to share it</li>
                                <li>Does not contain offensive, harmful, or inappropriate material</li>
                                <li>Does not impersonate other users or misrepresent your identity</li>
                            </ul>
                            <p className="leading-7">We reserve the right to remove any User Content that violates these Terms or is otherwise objectionable at our sole discretion.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 5 */}
                    <section id="intellectual">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">5. Intellectual Property Rights</h2>
                        <div className="space-y-4">
                            <h3 className="scroll-m-20 font-semibold text-xl tracking-tight">5.1 Myrtle Code and Content</h3>
                            <p className="leading-7">The Myrtle platform code is open source and available on GitHub. Original features, designs, and documentation created for Myrtle are provided under applicable open source licenses. See our GitHub repository for specific license terms.</p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">5.2 Arknights Assets and Content</h3>
                            <p className="leading-7">
                                Arknights and all related assets, characters, operator data, artwork, voice lines, and imagery are the property of Hypergryph Network Technology Co., Ltd. and Yostar Limited. This Service is a fan-made project and is not officially affiliated with, endorsed by, or sponsored by Hypergryph
                                or Yostar.
                            </p>
                            <p className="leading-7">We use game assets under fair use for the purpose of providing game companion tools to the community. All operator data, images, Spine animations, and other game content displayed on Myrtle are sourced from the official Arknights game files.</p>

                            <h3 className="mt-6 scroll-m-20 font-semibold text-xl tracking-tight">5.3 Third-Party Content</h3>
                            <p className="leading-7">Our asset processing uses FlatBuffers schemas from the OpenArknightsFBS project (MooncellWiki). DPS calculator implementations are based on community research and reference calculations.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 6 */}
                    <section id="termination">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">6. Termination and Suspension</h2>
                        <div className="space-y-4">
                            <p className="leading-7">We may terminate or suspend your account and access to the Service immediately, without prior notice or liability, for any reason, including but not limited to breach of these Terms.</p>
                            <p className="leading-7">Upon termination, your right to use the Service will immediately cease. If you wish to terminate your account, you may discontinue using the Service and contact us to request account deletion.</p>
                            <p className="leading-7">All provisions of these Terms which by their nature should survive termination shall survive, including ownership provisions, warranty disclaimers, and limitations of liability.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 7 */}
                    <section id="liability">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">7. Limitation of Liability</h2>
                        <div className="space-y-4">
                            <p className="leading-7">The Service is provided on an "AS IS" and "AS AVAILABLE" basis without warranties of any kind, either express or implied. We do not warrant that the Service will be uninterrupted, secure, or error-free.</p>
                            <p className="leading-7">To the maximum extent permitted by law, Myrtle shall not be liable for any indirect, incidental, special, consequential, or punitive damages, including but not limited to loss of profits, data, or other intangible losses, resulting from:</p>
                            <ul className="my-4 ml-6 list-disc [&>li]:mt-2">
                                <li>Your access to or use of (or inability to access or use) the Service</li>
                                <li>Any unauthorized access to or use of our servers and/or any personal information stored therein</li>
                                <li>Any interruption or cessation of transmission to or from the Service</li>
                                <li>Any bugs, viruses, or other harmful code that may be transmitted through the Service</li>
                            </ul>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 8 */}
                    <section id="changes">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">8. Changes to Terms</h2>
                        <div className="space-y-4">
                            <p className="leading-7">We reserve the right to modify or replace these Terms at any time at our sole discretion. If a revision is material, we will provide at least 30 days' notice prior to any new terms taking effect.</p>
                            <p className="leading-7">What constitutes a material change will be determined at our sole discretion. By continuing to access or use our Service after revisions become effective, you agree to be bound by the revised terms.</p>
                        </div>
                    </section>

                    <Separator />

                    {/* Section 9 */}
                    <section id="contact">
                        <h2 className="mb-4 scroll-m-20 font-semibold text-3xl tracking-tight">9. Contact Information</h2>
                        <div className="space-y-4">
                            <p className="leading-7">If you have any questions about these Terms, please contact us:</p>
                            <div className="rounded-lg border border-border bg-muted/50 p-6">
                                <p className="mb-2 font-medium">Discord:</p>
                                <Link className="mb-4 block text-primary transition-colors hover:underline" href="/discord" target="_blank">
                                    Join our Discord server
                                </Link>
                                <p className="mb-2 font-medium">Source Code:</p>
                                <Link className="text-primary transition-colors hover:underline" href="https://github.com/Eltik/myrtle.moe" target="_blank">
                                    github.com/Eltik/myrtle.moe
                                </Link>
                            </div>
                        </div>
                    </section>
                </div>

                {/* Footer Navigation */}
                <div className="mt-16 flex flex-col items-center justify-between gap-4 rounded-lg border border-border bg-muted/30 p-6 sm:flex-row">
                    <div>
                        <p className="font-medium text-sm">Related Documents</p>
                        <div className="mt-2 flex gap-4 text-sm">
                            <Link className="text-primary transition-colors hover:underline" href="/privacy">
                                Privacy Policy
                            </Link>
                        </div>
                    </div>
                    <Button asChild variant="outline">
                        <Link href="/">Return Home</Link>
                    </Button>
                </div>
            </article>
        </>
    );
}
