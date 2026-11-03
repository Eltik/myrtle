import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { Badge } from "#/components/ui/badge";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import styles from "./LegalPageShell.module.css";
import type { messages } from "./LegalShell.messages";

/** `ambient` adds the hero's corner glow and faint grid behind the header. */
export function LegalContainer({ children, ambient }: { children: ReactNode; ambient?: boolean }) {
    return (
        <div className="relative overflow-x-clip">
            <div className={styles.pageAmbient} aria-hidden="true" />
            {ambient ? (
                <>
                    <div className={styles.heroAmb} aria-hidden="true" />
                    <div className={styles.heroGrid} aria-hidden="true" />
                </>
            ) : null}
            <article className="page-gutter relative z-1 py-14 [--page-max:880px] sm:py-16">{children}</article>
        </div>
    );
}

interface ILegalHeaderProps {
    title: ReactNode;
    subtitle: ReactNode;
    /** The document's version, already worded ("Version 2.0"), as an outline badge at the right. */
    version: ReactNode;
    /** The dated lines under the standfirst (effective, last updated), each an icon and its text. */
    children: ReactNode;
}

/** The one header both legal documents open with: title and standfirst left, the version badge right, the dates under a rule. */
export function LegalHeader({ title, subtitle, version, children }: ILegalHeaderProps) {
    return (
        <header className="mb-12 border-border border-b pb-8">
            {/* Below sm the version badge drops under the subtitle: beside it, it left the
                subtitle about 60% of a 375 px screen. */}
            <div className="flex flex-col items-start gap-4 sm:flex-row sm:justify-between">
                <div className="min-w-0">
                    {/* The site's page title size (ui/page-header): a 52 px title here read as a different site. */}
                    <h1 className="m-0 mb-2 text-balance font-bold font-sans text-[24px] text-foreground leading-[1.1] tracking-tight sm:text-[30px]">{title}</h1>
                    <p className="m-0 max-w-[56ch] font-sans text-[15px] text-muted-foreground leading-[1.55]">{subtitle}</p>
                </div>
                <div className="flex shrink-0 gap-2 sm:pt-2">
                    <Badge variant="outline" size="lg">
                        {version}
                    </Badge>
                </div>
            </div>
            <div className="mt-6 flex flex-wrap items-center gap-x-4 gap-y-2 font-sans text-[14px] text-muted-foreground">{children}</div>
        </header>
    );
}

/** One dated line in the header; a hairline separates it from the line before it from sm up. */
export function LegalHeaderLine({ icon, children, separated = false }: { icon: ReactNode; children: ReactNode; separated?: boolean }) {
    return (
        <>
            {separated ? <span className="hidden h-4 w-px bg-border sm:inline-block" aria-hidden="true" /> : null}
            <span className="inline-flex items-center gap-2">
                {icon}
                {children}
            </span>
        </>
    );
}

interface ISectionHeadingProps {
    title: ReactNode;
    subtitle?: ReactNode;
}

export function SectionHeading({ title, subtitle }: ISectionHeadingProps) {
    return (
        <div className="mb-4 min-w-0">
            <h2 className="m-0 font-(--font-heading) font-semibold text-[20px] text-foreground leading-[1.25] tracking-[-0.01em] sm:text-[22px]">{title}</h2>
            {subtitle ? <p className="m-0 mt-1 font-sans text-[14px] text-muted-foreground leading-normal">{subtitle}</p> : null}
        </div>
    );
}

export function RelatedLinksFooter({ children }: { children: ReactNode }) {
    const t: TypedT<typeof messages> = useT("legal");

    return (
        <div className="mt-12 flex flex-col gap-4 rounded-2xl border border-border bg-card px-5 py-4 sm:flex-row sm:items-center sm:justify-between sm:gap-6">
            <div>
                <p className="m-0 mb-1.5 font-medium font-sans text-[14px] text-foreground">{t("shell.relatedDocuments")}</p>
                <div className="inline-flex flex-wrap gap-x-4 gap-y-1.5">{children}</div>
            </div>
            <Button variant="outline" size="lg" render={<Link to="/" />}>
                {t("shell.returnHome")}
            </Button>
        </div>
    );
}

export function RelatedDocLink({ to, label }: { to: "/privacy" | "/terms"; label: string }) {
    return (
        <Link
            to={to}
            className="relative font-sans text-[14px] text-primary underline decoration-1 underline-offset-[3px] transition-[text-decoration-thickness] pointer-coarse:after:absolute pointer-coarse:after:inset-x-0 pointer-coarse:after:top-1/2 pointer-coarse:after:h-11 pointer-coarse:after:-translate-y-1/2 hover:decoration-2"
        >
            {label}
        </Link>
    );
}

export function LegalDivider() {
    return <Separator className="my-10" />;
}
