import { Link } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { Button } from "#/components/ui/button";
import { Separator } from "#/components/ui/separator";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import styles from "./LegalPageShell.module.css";
import type { messages } from "./LegalShell.messages";

interface ILegalContainerProps {
    children: ReactNode;
    ambient?: boolean;
}

export function LegalContainer({ children, ambient }: ILegalContainerProps) {
    return (
        <div className="relative overflow-x-clip">
            {ambient ? <div className={styles.heroGrid} aria-hidden="true" /> : null}
            <article className="page-gutter relative z-1 py-14 [--page-max:880px] sm:py-16">{children}</article>
        </div>
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
            <div className="min-w-0">
                <h2 className="m-0 font-(--font-heading) text-[26px] text-foreground leading-[1.2] tracking-[-0.02em] sm:text-[30px]">{title}</h2>
                {subtitle ? <p className="m-0 mt-1 font-sans text-[14px] text-muted-foreground leading-normal">{subtitle}</p> : null}
            </div>
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
