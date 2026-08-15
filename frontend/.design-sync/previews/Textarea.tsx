import { Field, FieldDescription, FieldLabel, Textarea } from "frontend";

/** Ported from the "Create tier list" dialog: label, live counter, hint, textarea. */
export const DescriptionField = () => (
    <Field className="w-full max-w-md">
        <FieldLabel>
            Description
            <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">64 / 280</span>
        </FieldLabel>
        <Textarea defaultValue="Ranks every 6★ Guard by how much raw DPS they add to a Chapter 8 clear." rows={3} />
        <FieldDescription>Optional. A sentence or two helps readers know what to expect.</FieldDescription>
    </Field>
);

/** Empty with a placeholder — what a doctor sees before typing. */
export const Placeholder = () => (
    <Field className="w-full max-w-md">
        <FieldLabel>Notes on this operator</FieldLabel>
        <Textarea placeholder="What's this list about? Who is it for?" rows={3} />
    </Field>
);

/** The three sizes the control ships. */
export const Sizes = () => (
    <div className="flex w-full max-w-md flex-col gap-4">
        <Textarea defaultValue="Small — 2 rows of leading, used inside compact planner panels." size="sm" />
        <Textarea defaultValue="Default — the size used by dialogs and the tier-list editor." />
        <Textarea defaultValue="Large — the changelog composer, where entries run long." size="lg" />
    </div>
);

/** Invalid and disabled, as the tier-list editor renders them. */
export const States = () => (
    <div className="flex w-full max-w-md flex-col gap-4">
        <Field>
            <FieldLabel>Description</FieldLabel>
            <Textarea aria-invalid defaultValue="Ranks every 6★ Guard by how much raw DPS they add to a Chapter 8 clear. Then it keeps going well past the 280 character limit…" rows={3} />
            <FieldDescription className="text-destructive">Description must be 280 characters or fewer.</FieldDescription>
        </Field>
        <Field>
            <FieldLabel>Moderator note</FieldLabel>
            <Textarea defaultValue="Read-only — you need the tier-list:moderate grant to edit this." disabled rows={2} />
        </Field>
    </div>
);
