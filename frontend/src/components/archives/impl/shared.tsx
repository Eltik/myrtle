import { Link } from "@tanstack/react-router";
import { EyeOffIcon } from "lucide-react";
import type * as React from "react";
import { useEffect, useId, useState } from "react";
import { campLogo } from "#/components/operators/detail/impl/assets";
import { OperatorAvatar } from "#/components/ui/operator-avatar";
import { type TypedRichT, useRichT, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { cn, RARITY_HEX } from "#/lib/utils";
import type { ArchivePerson } from "#/types/generated/ArchivePerson";
import type { ExamAxis } from "#/types/generated/ExamAxis";
import type { ExamStep } from "#/types/generated/ExamStep";
import type { InfectionState } from "#/types/generated/InfectionState";
import type { messages } from "../Archives.messages";

export type ArchivesT = TypedT<typeof messages>;

export function useArchivesT(): ArchivesT {
    return useT("archives");
}

export function useArchivesRichT(): TypedRichT<typeof messages> {
    return useRichT("archives");
}

export const INFECTION_KEY: Record<InfectionState, "infection.infected" | "infection.clear" | "infection.unclear" | "infection.unstated"> = {
    infected: "infection.infected",
    clear: "infection.clear",
    unclear: "infection.unclear",
    unstated: "infection.unstated",
};

export const AXIS_KEY: Record<ExamAxis, "exam.strength" | "exam.mobility" | "exam.resilience" | "exam.tactics" | "exam.combat" | "exam.arts"> = {
    strength: "exam.strength",
    mobility: "exam.mobility",
    resilience: "exam.resilience",
    tactics: "exam.tactics",
    combat: "exam.combat",
    arts: "exam.arts",
};

/** The site's words for the exam steps, used only when a server's files print no word for a step. */
export const STEP_KEY: Record<ExamStep, "exam.step.flawed" | "exam.step.normal" | "exam.step.standard" | "exam.step.excellent" | "exam.step.outstanding"> = {
    flawed: "exam.step.flawed",
    normal: "exam.step.normal",
    standard: "exam.step.standard",
    excellent: "exam.step.excellent",
    outstanding: "exam.step.outstanding",
};

/** Infected red and not-infected teal, shared by every infection chart. */
export const INFECTED_COLOR = "var(--destructive)";
export const CLEAR_COLOR = "var(--success)";

export function rarityColor(rarity: number | null | undefined): string {
    return rarity ? (RARITY_HEX[rarity] ?? "var(--muted-foreground)") : "var(--muted-foreground)";
}

/** A person as a small chip: avatar, name, a rarity edge. Clickable when `onOpen` is given. */
export function PersonChip({ person, onOpen, server, className }: { person: ArchivePerson | undefined; onOpen?: (id: string) => void; server?: string; className?: string }): React.ReactElement | null {
    if (!person) return null;
    const body = (
        <>
            <span className="relative size-5 shrink-0 overflow-hidden rounded-sm bg-muted font-semibold text-[10px] text-muted-foreground leading-5">
                <OperatorAvatar charId={person.npc ? null : person.id} name={person.name} server={server} />
            </span>
            <span className="truncate" title={person.name}>
                {person.name}
            </span>
        </>
    );
    const classes = cn("inline-flex max-w-full items-center gap-1.5 rounded-md border border-border border-l-[3px] bg-card py-0.5 pr-2 pl-1 text-[12.5px] text-foreground", className);
    const style = { borderLeftColor: rarityColor(person.rarity) };
    if (onOpen && !person.npc) {
        return (
            <button type="button" onClick={() => onOpen(person.id)} className={cn(classes, "pointer-coarse:min-h-11 cursor-pointer transition-colors hover:bg-accent")} style={style}>
                {body}
            </button>
        );
    }
    return (
        <span className={classes} style={style}>
            {body}
        </span>
    );
}

/**
 * A faction's camp logo in the current text colour, dropped when the game
 * ships none for it. The game's camp logos are one flat white on alpha
 * (logo_rhodes, logo_kazimierz, logo_lungmen all read 255,255,255 on every
 * opaque pixel), so the art is a mask over `currentColor` and follows the
 * theme the way text does; an <img> would be white on the light theme.
 */
export function FactionLogo({ id, className }: { id: string; className?: string }): React.ReactElement | null {
    const src = campLogo(id);
    const [failed, setFailed] = useState<string | null>(null);
    // A CSS mask reports no load error, so an Image probes the file first.
    useEffect(() => {
        let alive = true;
        const probe = new Image();
        probe.onerror = () => {
            if (alive) setFailed(src);
        };
        probe.src = src;
        return () => {
            alive = false;
        };
    }, [src]);
    if (failed === src) return null;
    const mask = `url("${src}") center / contain no-repeat`;
    return <span aria-hidden className={cn("inline-block shrink-0 bg-current", className)} style={{ mask, WebkitMask: mask }} />;
}

/** {@link FactionLogo} inside an SVG: the logo masks a rect filled with `currentColor`. */
export function FactionLogoSvg({ id, x, y, size, opacity, className }: { id: string; x: number; y: number; size: number; opacity?: number; className?: string }): React.ReactElement {
    const maskId = useId();
    return (
        <g opacity={opacity} className={className} aria-hidden>
            <mask id={maskId} maskUnits="userSpaceOnUse" x={x} y={y} width={size} height={size}>
                <image href={campLogo(id)} x={x} y={y} width={size} height={size} />
            </mask>
            <rect x={x} y={y} width={size} height={size} fill="currentColor" mask={`url(#${maskId})`} />
        </g>
    );
}

/** The line shown in place of a spoiler when the spoilers setting is off. */
export function SpoilerNote({ className }: { className?: string }): React.ReactElement {
    const rt = useArchivesRichT();
    const t = useArchivesT();
    return (
        <p className={cn("m-0 flex items-start gap-2 font-sans text-[13px] text-muted-foreground leading-normal", className)}>
            <EyeOffIcon aria-hidden className="mt-0.5 size-3.5 shrink-0" />
            <span>
                {rt("spoilers.off", {
                    setting: (
                        <Link to="/settings" className="font-medium text-foreground underline underline-offset-2">
                            {t("spoilers.setting")}
                        </Link>
                    ),
                })}
            </span>
        </p>
    );
}

/** Dark card with a primary kicker title, the panel every Archives card sits in. */
export function Panel({ title, children, className, footer }: { title?: React.ReactNode; children: React.ReactNode; className?: string; footer?: React.ReactNode }): React.ReactElement {
    return (
        <section className={cn("flex min-w-0 flex-col gap-3 rounded-[14px] border border-border bg-card p-4 sm:p-5", className)}>
            {title ? <h3 className="m-0 font-bold font-sans text-[0.69rem] text-primary uppercase tracking-[0.22em]">{title}</h3> : null}
            {children}
            {footer ? <div className="font-sans text-[12px] text-muted-foreground leading-normal">{footer}</div> : null}
        </section>
    );
}

export function peopleById(people: readonly ArchivePerson[] | undefined): Map<string, ArchivePerson> {
    return new Map((people ?? []).map((p) => [p.id, p]));
}

/** A tab's loading or error line, in place of its body. */
export function TabMessage({ children }: { children: React.ReactNode }): React.ReactElement {
    return <p className="text-muted-foreground text-sm">{children}</p>;
}

/** The headline figures under a tab's lede: four cells, two per row on a phone. */
export function StatStrip({ children }: { children: React.ReactNode }): React.ReactElement {
    return <div className="grid grid-cols-4 overflow-hidden rounded-[14px] border border-border bg-card max-[760px]:grid-cols-2">{children}</div>;
}

/** One {@link StatStrip} cell. `grow` sets the figure larger from `sm` up. */
export function BigStat({ label, value, grow }: { label: string; value: string; grow?: boolean }): React.ReactElement {
    return (
        <div className="flex flex-col gap-1.5 border-border not-last:border-r px-4 py-3.5 max-[760px]:nth-[2n]:border-r-0">
            <span className={cn("font-bold text-[28px] text-foreground tabular-nums leading-none tracking-tight", grow && "sm:text-[34px]")}>{value}</span>
            <span className="font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">{label}</span>
        </div>
    );
}

/** A tab section's heading row: its index, its title, a line on what it shows. */
export function SectionHead({ index, title, blurb }: { index: string; title: string; blurb: string }): React.ReactElement {
    return (
        <header className="flex flex-wrap items-baseline gap-x-3 gap-y-1 border-border border-b pb-2">
            <span className="font-bold font-mono text-primary text-sm">{index}</span>
            <h2 className="m-0 font-semibold text-[20px] text-foreground tracking-tight">{title}</h2>
            <p className="m-0 text-[13px] text-muted-foreground">{blurb}</p>
        </header>
    );
}

/** Pointer and focus both set the hover, so a keyboard reader lights the same things a mouse does. */
export function hoverProps<T>(onHover: (value: T | null) => void, value: T) {
    return {
        onPointerEnter: () => onHover(value),
        onPointerLeave: () => onHover(null),
        onFocus: () => onHover(value),
        onBlur: () => onHover(null),
    };
}
