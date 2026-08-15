import { Button, Field, FieldDescription, FieldError, FieldLabel, Form, Input, Textarea } from "frontend";

export const LoginForm = () => (
    <Form className="max-w-sm">
        <Field>
            <FieldLabel>Email</FieldLabel>
            <Input placeholder="doctor@rhodes.island" type="email" />
        </Field>
        <Field>
            <FieldLabel>Server</FieldLabel>
            <Input defaultValue="Global (EN)" />
            <FieldDescription>Use your YoStar email to send an OTP code. No login information is stored on the server.</FieldDescription>
        </Field>
        <div className="flex items-center gap-2">
            <Button type="submit" variant="outline">
                Send Code
            </Button>
            <Button type="button" variant="ghost">
                Cancel
            </Button>
        </div>
    </Form>
);

export const TierListDetails = () => (
    <Form className="max-w-md">
        <Field>
            <FieldLabel>
                Name
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">21 / 60</span>
            </FieldLabel>
            <Input defaultValue="Endgame DPS rankings" />
            <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
        </Field>
        <Field>
            <FieldLabel>Description</FieldLabel>
            <Textarea defaultValue="Ranks every 6-star DPS operator at E2 L90, S3M3, no module." rows={2} />
        </Field>
        <div className="flex items-center gap-2">
            <Button type="submit">Create list</Button>
            <Button type="button" variant="outline">
                Cancel
            </Button>
        </div>
    </Form>
);

export const WithServerErrors = () => (
    <Form
        className="max-w-sm"
        errors={{
            email: "We could not find a YoStar account for this address.",
            code: "That one-time code has expired. Request a new one.",
        }}
    >
        <Field name="email">
            <FieldLabel>Email</FieldLabel>
            <Input defaultValue="doctor@rhodes.island" type="email" />
            <FieldError />
        </Field>
        <Field name="code">
            <FieldLabel>Code</FieldLabel>
            <Input defaultValue="482913" />
            <FieldError />
        </Field>
        <div className="flex items-center gap-2">
            <Button type="submit" variant="outline">
                Login
            </Button>
            <Button className="h-auto p-0 text-xs" type="button" variant="ghost">
                Resend code
            </Button>
        </div>
    </Form>
);
