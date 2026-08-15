import { Field, FieldDescription, FieldError, FieldLabel, Form, Input, Textarea } from "frontend";

export const TierListDetails = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>
                Name
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">21 / 60</span>
            </FieldLabel>
            <Input defaultValue="Endgame DPS rankings" placeholder="e.g. Endgame DPS rankings" />
            <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>
                Description
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">62 / 240</span>
            </FieldLabel>
            <Textarea defaultValue="Ranks every 6-star DPS operator at E2 L90, S3M3, no module." rows={3} />
            <FieldDescription>Optional. A sentence or two helps readers know what to expect.</FieldDescription>
        </Field>
    </div>
);

export const Disabled = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field disabled>
            <FieldLabel>YoStar email</FieldLabel>
            <Input defaultValue="doctor@rhodes.island" type="email" />
            <FieldDescription>Locked while the one-time code is pending. Use "Change email" to edit it.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>Server</FieldLabel>
            <Input defaultValue="Global (EN)" />
            <FieldDescription>Determines which release schedule the site follows.</FieldDescription>
        </Field>
    </div>
);

export const Invalid = () => (
    <Form className="w-full max-w-md" errors={{ nickname: "That nickname is already taken by another Doctor." }}>
        <Field name="nickname">
            <FieldLabel>Public nickname</FieldLabel>
            <Input defaultValue="Amiya" />
            <FieldError />
            <FieldDescription>Shown on your profile and on any tier list you publish.</FieldDescription>
        </Field>
    </Form>
);
