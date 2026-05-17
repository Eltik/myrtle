import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import styles from "./LegalPageShell.module.css";

interface ILegalContainerProps {
    children: ReactNode;
    ambient?: boolean;
}

export function LegalContainer({ children, ambient }: ILegalContainerProps) {
    return (
        <main className="relative overflow-x-clip">
            <div className={styles.pageAmbient} aria-hidden="true" />
            {ambient ? (
                <>
                    <div className={styles.heroAmb} aria-hidden="true" />
                    <div className={styles.heroGrid} aria-hidden="true" />
                </>
            ) : null}
            <article className="relative z-1 mx-auto w-[min(880px,calc(100%-2rem))] py-14 sm:py-16">{children}</article>
        </main>
    );
}

interface ISectionHeadingProps {
    icon: ReactNode;
    title: ReactNode;
    subtitle?: ReactNode;
}

export function SectionHeading({ icon, title, subtitle }: ISectionHeadingProps) {
    return (
        <div className="mb-4 flex items-center gap-3">
            <div className="inline-flex size-10 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary">{icon}</div>
            <div>
                <h2 className="m-0 font-(--font-heading) text-[26px] text-foreground leading-[1.2] tracking-[-0.02em] sm:text-[30px]">{title}</h2>
                {subtitle ? <p className="m-0 mt-1 font-sans text-[14px] text-muted-foreground leading-snug">{subtitle}</p> : null}
            </div>
        </div>
    );
}

export function RelatedLinksFooter({ children }: { children: ReactNode }) {
    return (
        <div className="mt-12 flex flex-col gap-4 rounded-xl border border-border bg-[color-mix(in_srgb,var(--muted)_30%,transparent)] px-5 py-4 sm:flex-row sm:items-center sm:justify-between sm:gap-6">
            <div>
                <p className="m-0 mb-1.5 font-medium font-sans text-[14px] text-foreground">Related Documents</p>
                <div className="inline-flex flex-wrap gap-x-4 gap-y-1.5">{children}</div>
            </div>
            <Button variant="outline" size="lg" render={<Link to="/" />}>
                Return Home
            </Button>
        </div>
    );
}

export function RelatedDocLink({ to, label }: { to: "/privacy" | "/terms"; label: string }) {
    return (
        <Link to={to} className="font-sans text-[14px] text-primary no-underline transition-colors hover:underline hover:underline-offset-[3px]">
            {label}
        </Link>
    );
}

export function LegalDivider() {
    return <Separator className="my-10" />;
}
