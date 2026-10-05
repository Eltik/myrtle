import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { PlusIcon } from "lucide-react";
import { useEffect, useId, useState } from "react";
import { Button } from "#/components/ui/button";
import { Dialog, DialogClose, DialogDescription, DialogFooter, DialogHeader, DialogPanel, DialogPopup, DialogTitle } from "#/components/ui/dialog";
import { useErrorMessage } from "#/components/ui/error-message";
import { Field, FieldLabel } from "#/components/ui/field";
import { Input } from "#/components/ui/input";
import { NumberField, NumberFieldDecrement, NumberFieldGroup, NumberFieldIncrement, NumberFieldInput } from "#/components/ui/number-field";
import { useAuth } from "#/hooks/use-auth";
import { createGridFn } from "#/lib/api/grids";
import type { TierEntityKind } from "#/lib/api/tier-entities";
import { authActions } from "#/lib/auth/store";
import { useGamedataServer, useT } from "#/lib/i18n";
import type { TypedT } from "#/lib/i18n/messages";
import { truncateCodePoints } from "#/lib/markdown/sanitize-input";
import { cn } from "#/lib/utils";
import { AllowedKindsChecklist } from "./AllowedKinds";
import type { messages } from "./NewGridButton.messages";
import { ABOUT_ME_SIZE, GRID_DEFAULT_SIZE, GRID_MAX_SIZE, GRID_MIN_SIZE, GRID_TITLE_MAX, type GridStarter } from "./shared";
import { starterGridInput, starterKinds } from "./state";

/** "New grid": the create dialog for a signed-in user, the sign-in dialog for anyone else. A created grid opens in the editor. */
export function NewGridButton({ className }: { className?: string }) {
    const t: TypedT<typeof messages> = useT("grids");
    const { user } = useAuth();
    const [open, setOpen] = useState(false);

    return (
        <>
            <Button type="button" className={className} onClick={() => (user ? setOpen(true) : authActions.openLoginDialog())}>
                <PlusIcon />
                {t("create.open")}
            </Button>
            {user && <CreateGridDialog open={open} onOpenChange={setOpen} />}
        </>
    );
}

const STARTERS: GridStarter[] = ["blank", "about-me"];

function CreateGridDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
    const t: TypedT<typeof messages> = useT("grids");
    const describeError = useErrorMessage();
    const queryClient = useQueryClient();
    const navigate = useNavigate();
    const server = useGamedataServer();
    const titleId = useId();
    const [title, setTitle] = useState("");
    const [rows, setRows] = useState(GRID_DEFAULT_SIZE);
    const [cols, setCols] = useState(GRID_DEFAULT_SIZE);
    const [starter, setStarter] = useState<GridStarter>("blank");
    const [kinds, setKinds] = useState<TierEntityKind[]>(() => starterKinds("blank"));
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!open) return;
        setTitle("");
        setRows(GRID_DEFAULT_SIZE);
        setCols(GRID_DEFAULT_SIZE);
        setStarter("blank");
        setKinds(starterKinds("blank"));
        setError(null);
    }, [open]);

    const create = useMutation({
        mutationFn: () => createGridFn({ data: { input: starterGridInput({ title, rows, cols, starter, kinds }), server } }),
        onSuccess: (grid) => {
            void queryClient.invalidateQueries({ queryKey: ["grids"] });
            onOpenChange(false);
            navigate({ to: "/grids/$slug/edit", params: { slug: grid.slug } });
        },
        onError: (err: unknown) => setError(describeError(err)),
    });

    const aboutMe = starter === "about-me";
    const trimmed = title.trim();
    const canSubmit = trimmed.length > 0 && kinds.length > 0 && !create.isPending;

    const pickStarter = (next: GridStarter) => {
        setStarter(next);
        // Each starter preselects its own types; the author can change them below.
        setKinds(starterKinds(next));
        // The About Me starter is its own 6 x 6; a title it suggests is only filled into an empty field.
        if (next === "about-me" && trimmed.length === 0) setTitle(t("create.aboutMeTitle"));
    };

    return (
        <Dialog open={open} onOpenChange={(next) => !create.isPending && onOpenChange(next)}>
            <DialogPopup className="sm:max-w-xl">
                <form
                    className="flex min-h-0 flex-col"
                    onSubmit={(e) => {
                        e.preventDefault();
                        if (canSubmit) create.mutate();
                    }}
                >
                    <DialogHeader>
                        <DialogTitle>{t("create.title")}</DialogTitle>
                        <DialogDescription>{t("create.description")}</DialogDescription>
                    </DialogHeader>

                    <DialogPanel className="flex flex-col gap-5">
                        <fieldset className="m-0 flex flex-col gap-2 border-0 p-0">
                            <legend className="mb-2 font-medium font-sans text-foreground text-sm">{t("create.starter")}</legend>
                            <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
                                {STARTERS.map((s) => (
                                    <label key={s} className={cn("flex cursor-pointer flex-col gap-0.5 rounded-lg border px-3 py-2.5 transition-colors has-focus-visible:ring-2 has-focus-visible:ring-ring", starter === s ? "border-primary bg-primary/8" : "border-border hover:bg-accent")}>
                                        <input type="radio" name="grid-starter" value={s} checked={starter === s} onChange={() => pickStarter(s)} className="sr-only" />
                                        <span className="font-medium font-sans text-foreground text-sm">{s === "blank" ? t("create.starter.blank") : t("create.starter.aboutMe")}</span>
                                        <span className="font-sans text-muted-foreground text-xs">{s === "blank" ? t("create.starter.blankHint") : t("create.starter.aboutMeHint", { size: ABOUT_ME_SIZE })}</span>
                                    </label>
                                ))}
                            </div>
                        </fieldset>

                        <Field>
                            <FieldLabel htmlFor={titleId}>
                                {t("create.name")}
                                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">
                                    {title.length} / {GRID_TITLE_MAX}
                                </span>
                            </FieldLabel>
                            <Input id={titleId} value={title} onChange={(e) => setTitle(truncateCodePoints((e.target as HTMLInputElement).value, GRID_TITLE_MAX))} placeholder={t("create.namePlaceholder")} autoFocus required />
                        </Field>

                        <div className="flex gap-4">
                            <CreateStepper label={t("create.rows")} value={aboutMe ? ABOUT_ME_SIZE : rows} disabled={aboutMe} onChange={setRows} />
                            <CreateStepper label={t("create.cols")} value={aboutMe ? ABOUT_ME_SIZE : cols} disabled={aboutMe} onChange={setCols} />
                        </div>

                        <AllowedKindsChecklist value={kinds} onChange={setKinds} />

                        {error && (
                            <div role="alert" className="rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                                {error}
                            </div>
                        )}
                    </DialogPanel>

                    <DialogFooter>
                        <DialogClose render={<Button type="button" variant="outline" disabled={create.isPending} />}>{t("create.cancel")}</DialogClose>
                        <Button type="submit" disabled={!canSubmit} loading={create.isPending}>
                            {t("create.submit")}
                        </Button>
                    </DialogFooter>
                </form>
            </DialogPopup>
        </Dialog>
    );
}

function CreateStepper({ label, value, disabled, onChange }: { label: string; value: number; disabled: boolean; onChange: (value: number) => void }) {
    return (
        <NumberField value={value} min={GRID_MIN_SIZE} max={GRID_MAX_SIZE} step={1} disabled={disabled} onValueChange={(v) => v !== null && onChange(v)} className="w-32 gap-1">
            <span className="font-medium font-sans text-foreground text-sm">{label}</span>
            <NumberFieldGroup>
                <NumberFieldDecrement />
                <NumberFieldInput aria-label={label} />
                <NumberFieldIncrement />
            </NumberFieldGroup>
        </NumberField>
    );
}
