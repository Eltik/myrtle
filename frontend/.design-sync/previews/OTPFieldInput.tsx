import { Field, FieldDescription, FieldLabel, OTPField, OTPFieldInput } from "frontend";

const SLOTS = ["s1", "s2", "s3", "s4", "s5", "s6"];

export const Empty = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" length={6} size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <FieldDescription>Each slot is its own input; typing advances to the next.</FieldDescription>
    </Field>
);

export const Filled = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418032" length={6} size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <FieldDescription>Paste the whole code into any slot and it spreads across the field.</FieldDescription>
    </Field>
);

export const Masked = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418032" length={6} mask size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <FieldDescription>`mask` on the root hides the entered characters.</FieldDescription>
    </Field>
);

export const Invalid = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418031" length={6} size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-invalid aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <p className="text-destructive-foreground text-xs">That code didn't match. Request a new one and try again.</p>
    </Field>
);
