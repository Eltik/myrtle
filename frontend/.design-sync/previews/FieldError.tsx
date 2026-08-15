import { Field, FieldDescription, FieldError, FieldLabel, Form, Input } from "frontend";

export const ServerRejected = () => (
    <Form className="w-full max-w-md" errors={{ nickname: "That nickname is already taken by another Doctor." }}>
        <Field name="nickname">
            <FieldLabel>Public nickname</FieldLabel>
            <Input defaultValue="Amiya" />
            <FieldError />
            <FieldDescription>Shown on your profile and on any tier list you publish.</FieldDescription>
        </Field>
    </Form>
);

export const AlwaysVisible = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>Tier list name</FieldLabel>
            <Input aria-invalid defaultValue="" placeholder="e.g. Endgame DPS rankings" />
            <FieldError match={true}>A name is required before you can publish.</FieldError>
        </Field>

        <Field>
            <FieldLabel>Sanity per day</FieldLabel>
            <Input aria-invalid defaultValue="900" />
            <FieldError match={true}>Enter a value between 0 and 480.</FieldError>
        </Field>
    </div>
);

export const MultipleFields = () => (
    <Form
        className="w-full max-w-md"
        errors={{
            email: "We could not find a YoStar account for this address.",
            code: "That one-time code has expired. Request a new one.",
        }}
    >
        <Field name="email">
            <FieldLabel>YoStar email</FieldLabel>
            <Input defaultValue="doctor@rhodes.island" type="email" />
            <FieldError />
        </Field>

        <Field name="code">
            <FieldLabel>One-time code</FieldLabel>
            <Input defaultValue="482913" />
            <FieldError />
        </Field>
    </Form>
);
