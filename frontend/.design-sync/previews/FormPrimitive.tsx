import { Button, Field, FieldDescription, FieldError, FieldLabel, FormPrimitive, Input } from "frontend";

export const UnstyledRoot = () => (
    <FormPrimitive className="flex w-full max-w-sm flex-col gap-4">
        <Field>
            <FieldLabel>Email</FieldLabel>
            <Input placeholder="doctor@rhodes.island" type="email" />
            <FieldDescription>We send a one-time code to this address. No login information is stored on the server.</FieldDescription>
        </Field>
        <Field>
            <FieldLabel>Server</FieldLabel>
            <Input defaultValue="Global (EN)" />
        </Field>
        <Button className="self-start" type="submit" variant="outline">
            Send Code
        </Button>
    </FormPrimitive>
);

export const WithServerErrors = () => (
    <FormPrimitive className="flex w-full max-w-sm flex-col gap-4" errors={{ nickname: "That nickname is already taken by another Doctor." }}>
        <Field name="nickname">
            <FieldLabel>Public nickname</FieldLabel>
            <Input defaultValue="Amiya" />
            <FieldError />
        </Field>
        <Field name="bio">
            <FieldLabel>Tagline</FieldLabel>
            <Input defaultValue="Chapter 8 — Roaring Flare" />
            <FieldDescription>Shown under your nickname on the public profile.</FieldDescription>
        </Field>
        <Button className="self-start" type="submit">
            Save profile
        </Button>
    </FormPrimitive>
);

export const GridLayout = () => (
    <FormPrimitive className="grid w-full max-w-md gap-4 sm:grid-cols-2">
        <Field>
            <FieldLabel>Sanity per day</FieldLabel>
            <Input defaultValue="240" />
        </Field>
        <Field>
            <FieldLabel>Target E2 level</FieldLabel>
            <Input defaultValue="90" />
        </Field>
        <Field>
            <FieldLabel>Skill mastery</FieldLabel>
            <Input defaultValue="S3M3" />
        </Field>
        <Field>
            <FieldLabel>Module stage</FieldLabel>
            <Input defaultValue="Stage 3" />
        </Field>
    </FormPrimitive>
);
