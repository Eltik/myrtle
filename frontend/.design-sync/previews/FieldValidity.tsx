import { Field, FieldLabel, FieldValidity, Form, Input } from "frontend";
import { CircleAlertIcon, InfoIcon } from "lucide-react";

export const InlineHint = () => (
    <Field className="w-full max-w-md" name="nickname">
        <FieldLabel>Public nickname</FieldLabel>
        <Input defaultValue="Kal'tsit" />
        <FieldValidity>
            {(state) =>
                state.validity.valid === false ? (
                    <p className="text-destructive-foreground text-xs">{state.error}</p>
                ) : (
                    <p className="inline-flex items-center gap-1.5 text-muted-foreground text-xs">
                        <InfoIcon className="size-3.5" />
                        3–24 characters. Visible on your profile and on lists you publish.
                    </p>
                )
            }
        </FieldValidity>
    </Field>
);

export const RejectedByServer = () => (
    <Form className="w-full max-w-md" errors={{ nickname: "That nickname is already taken by another Doctor." }}>
        <Field name="nickname">
            <FieldLabel>Public nickname</FieldLabel>
            <Input defaultValue="Amiya" />
            <FieldValidity>
                {(state) =>
                    state.validity.valid === false ? (
                        <p className="inline-flex items-center gap-1.5 rounded-lg border border-destructive/30 bg-destructive/8 px-3 py-2 font-sans text-destructive-foreground text-xs">
                            <CircleAlertIcon className="size-3.5 shrink-0" />
                            That nickname is already taken by another Doctor.
                        </p>
                    ) : (
                        <p className="text-muted-foreground text-xs">3–24 characters.</p>
                    )
                }
            </FieldValidity>
        </Field>
    </Form>
);

function StatusPill({ invalid }: { invalid: boolean }) {
    if (invalid) {
        return (
            <span className="inline-flex items-center gap-1.5 rounded-full border border-destructive/30 bg-destructive/8 px-2 py-0.5 text-destructive-foreground text-xs">
                <CircleAlertIcon className="size-3.5 shrink-0" />
                Needs attention
            </span>
        );
    }
    return <span className="inline-flex items-center gap-1.5 rounded-full bg-muted/30 px-2 py-0.5 text-muted-foreground text-xs">Ready to submit</span>;
}

export const StatusRow = () => (
    <Form
        className="w-full max-w-md"
        errors={{
            sanity: "Enter a value between 0 and 480.",
        }}
    >
        <Field name="server">
            <FieldLabel>Server</FieldLabel>
            <Input defaultValue="Global (EN)" />
            <FieldValidity>{(state) => <StatusPill invalid={state.validity.valid === false} />}</FieldValidity>
        </Field>

        <Field name="sanity">
            <FieldLabel>Sanity per day</FieldLabel>
            <Input defaultValue="900" />
            <FieldValidity>{(state) => <StatusPill invalid={state.validity.valid === false} />}</FieldValidity>
        </Field>
    </Form>
);
