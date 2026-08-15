import { Checkbox, Field, FieldDescription, FieldLabel, Input, Switch } from "frontend";

export const WithCharacterCount = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>
                Name
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">18 / 60</span>
            </FieldLabel>
            <Input defaultValue="Chapter 8 farming" />
        </Field>

        <Field>
            <FieldLabel>
                Sanity budget
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">per day</span>
            </FieldLabel>
            <Input defaultValue="240" />
        </Field>
    </div>
);

export const OnControls = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>
                <Checkbox defaultChecked />
                Only show operators I own
            </FieldLabel>
            <FieldDescription>Filters the roster against your last profile sync.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>
                <Switch defaultChecked />
                Include CN-only operators
            </FieldLabel>
            <FieldDescription>Adds operators that have not reached the Global server yet.</FieldDescription>
        </Field>
    </div>
);

export const Disabled = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field disabled>
            <FieldLabel>
                One-time code
                <span className="ml-auto font-mono text-[10.5px] text-muted-foreground tabular-nums">6 digits</span>
            </FieldLabel>
            <Input defaultValue="482913" />
            <FieldDescription>Resend available in 43s.</FieldDescription>
        </Field>

        <Field disabled>
            <FieldLabel>
                <Checkbox />
                Remember this device
            </FieldLabel>
        </Field>
    </div>
);
