import { Link } from "@tanstack/react-router";
import { CheckIcon, ExternalLinkIcon, PlusIcon, RotateCcwIcon } from "lucide-react";
import { useId } from "react";
import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "#/components/ui/breadcrumb";
import { Button } from "#/components/ui/button";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { Kicker } from "#/components/ui/kicker";
import { MarkdownEditor } from "#/components/ui/markdown-editor";
import { useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { LIST_DESCRIPTION_MAX as DESC_MAX, LIST_NAME_MAX as TITLE_MAX } from "../shared";
import type { messages } from "./EditHero.messages";
import type { IPendingChange } from "./state";

interface IEditHeroProps {
    slug: string;
    title: string;
    description: string;
    onTitleChange: (next: string) => void;
    onDescriptionChange: (next: string) => void;
    onSave: () => void;
    onReset: () => void;
    onAddTier: () => void;
    pendingChanges: IPendingChange[];
    saving: boolean;
    saveError: string | null;
    saveProgress: { step: number; total: number; label: string } | null;
}

export function EditHero({ slug, title, description, onTitleChange, onDescriptionChange, onSave, onReset, onAddTier, pendingChanges, saving, saveError, saveProgress }: IEditHeroProps) {
    const t: TypedT<typeof messages> = useT("tierLists");
    const titleId = useId();
    const descId = useId();
    const isDirty = pendingChanges.length > 0;
    const trimmedTitle = title.trim();
    const titleValid = trimmedTitle.length > 0;

    return (
        <header className="border-border/60 border-b bg-linear-to-b from-card/40 to-transparent">
            <div className="mx-auto w-[min(1280px,calc(100%-1.5rem))] pt-5 pb-6 sm:w-[min(1280px,calc(100%-2rem))] sm:pt-8 sm:pb-7">
                <Breadcrumb className="mb-3">
                    <BreadcrumbList className="text-xs">
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/tier-lists/my" search={{ sort: "recent", type: "all", view: "grid", q: "" }} />}>{t("edit.hero.breadcrumbMine")}</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbLink render={<Link to="/tier-lists/$id" params={{ id: slug }} />}>{title || t("edit.hero.breadcrumbUntitled")}</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator />
                        <BreadcrumbItem>
                            <BreadcrumbPage className="font-medium">{t("edit.hero.breadcrumbEdit")}</BreadcrumbPage>
                        </BreadcrumbItem>
                    </BreadcrumbList>
                </Breadcrumb>

                <div className="flex flex-col gap-6 lg:flex-row lg:items-start lg:justify-between">
                    <div className="min-w-0 flex-1">
                        <Kicker>{t("edit.hero.kicker")}</Kicker>

                        <div className="mt-1 flex flex-col gap-3">
                            <Field>
                                <FieldLabel htmlFor={titleId} className="sr-only">
                                    {t("edit.hero.titleLabel")}
                                </FieldLabel>
                                <Input
                                    id={titleId}
                                    value={title}
                                    onChange={(e) => onTitleChange((e.target as HTMLInputElement).value.slice(0, TITLE_MAX))}
                                    placeholder={t("edit.hero.titlePlaceholder")}
                                    size="lg"
                                    aria-invalid={!titleValid || undefined}
                                    className="font-bold text-2xl sm:text-3xl [&_input]:font-sans [&_input]:tracking-tight"
                                />
                            </Field>

                            <Field>
                                <FieldLabel htmlFor={descId} className="sr-only">
                                    {t("edit.hero.descriptionLabel")}
                                </FieldLabel>
                                <MarkdownEditor id={descId} value={description} onChange={onDescriptionChange} placeholder={t("edit.hero.descriptionPlaceholder")} rows={4} maxLength={DESC_MAX} showHint={false} />
                            </Field>

                            <div className="flex flex-wrap items-center gap-x-3 gap-y-1 font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">
                                <span className="tabular-nums">
                                    <span className="text-foreground">{title.length}</span> {t("edit.hero.titleCounter", { max: TITLE_MAX })}
                                </span>
                                <span aria-hidden="true">·</span>
                                <span className="tabular-nums">
                                    <span className="text-foreground">{description.length}</span> {t("edit.hero.descriptionCounter", { max: DESC_MAX })}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div className="flex w-full shrink-0 flex-col gap-3 lg:w-72">
                        <div className="rounded-xl border border-border bg-card p-3 shadow-[0_1px_2px_oklch(0_0_0/0.04)]">
                            <div className="flex items-center justify-between gap-2">
                                <span className="font-bold font-mono text-[10.5px] text-muted-foreground uppercase tracking-[0.14em]">{isDirty ? t("edit.hero.unsaved") : t("edit.hero.allSaved")}</span>
                                {!isDirty && <CheckIcon className="h-3.5 w-3.5 text-primary" aria-hidden="true" />}
                            </div>
                            {isDirty ? (
                                <ul className="mt-2 max-h-28 list-none overflow-y-auto p-0 font-sans text-[12px] text-foreground">
                                    {pendingChanges.slice(0, 6).map((c) => (
                                        <li key={`${c.kind}::${c.label}`} className="flex items-center gap-1.5 py-0.5">
                                            <span className="h-1.5 w-1.5 rounded-full bg-primary" aria-hidden="true" />
                                            <span className="truncate">{c.label}</span>
                                        </li>
                                    ))}
                                    {pendingChanges.length > 6 && <li className="mt-1 font-mono text-[10.5px] text-muted-foreground">{t("edit.hero.moreChanges", { count: pendingChanges.length - 6 })}</li>}
                                </ul>
                            ) : (
                                <p className="mt-1 font-sans text-[12px] text-muted-foreground">{t("edit.hero.upToDate")}</p>
                            )}

                            {saveProgress && (
                                <div className="mt-2 rounded-md border border-border bg-muted/50 px-2 py-1.5">
                                    <div className="flex items-center justify-between font-mono text-[10px] text-muted-foreground tabular-nums">
                                        <span>{saveProgress.label}</span>
                                        <span>
                                            {saveProgress.step}/{saveProgress.total}
                                        </span>
                                    </div>
                                    <div className="mt-1 h-1 overflow-hidden rounded-full bg-border">
                                        <div className="h-full bg-primary transition-[width] duration-200" style={{ width: `${Math.round((saveProgress.step / Math.max(1, saveProgress.total)) * 100)}%` }} aria-hidden="true" />
                                    </div>
                                </div>
                            )}

                            {saveError && (
                                <div role="alert" className="mt-2 rounded-md border border-destructive/30 bg-destructive/8 px-2 py-1.5 font-sans text-[11.5px] text-destructive-foreground">
                                    {saveError}
                                </div>
                            )}
                        </div>

                        <div className="flex flex-wrap items-center gap-1.5">
                            <Button type="button" onClick={onSave} disabled={!isDirty || saving || !titleValid} loading={saving} className="flex-1">
                                <CheckIcon />
                                {t("edit.hero.save")}
                            </Button>
                            <Button type="button" variant="outline" onClick={onReset} disabled={!isDirty || saving} size="icon" aria-label={t("edit.hero.discard")}>
                                <RotateCcwIcon />
                            </Button>
                        </div>

                        <div className="flex flex-wrap items-center gap-1.5">
                            <Button type="button" variant="outline" onClick={onAddTier} className="flex-1">
                                <PlusIcon />
                                {t("edit.hero.addTier")}
                            </Button>
                            <Button type="button" variant="ghost" render={<Link to="/tier-lists/$id" params={{ id: slug }} target="_blank" rel="noreferrer" />} size="icon" aria-label={t("edit.hero.openPublic")}>
                                <ExternalLinkIcon />
                            </Button>
                        </div>
                    </div>
                </div>
            </div>
        </header>
    );
}
