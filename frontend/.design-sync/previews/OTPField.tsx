import { Field, FieldDescription, FieldLabel, OTPField, OTPFieldInput } from "frontend";

const SLOTS = ["s1", "s2", "s3", "s4", "s5", "s6"];

export const LoginCode = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" length={6} size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <FieldDescription>We sent a 6-digit code to doctor@rhodes.island.</FieldDescription>
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
        <FieldDescription>Codes expire five minutes after they are sent.</FieldDescription>
    </Field>
);

export const Sizes = () => (
    <div className="flex w-full max-w-sm flex-col gap-5">
        <Field>
            <FieldLabel>Default</FieldLabel>
            <OTPField aria-label="One-time password, default size" defaultValue="418032" length={6}>
                {SLOTS.map((slotKey, index) => (
                    <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
                ))}
            </OTPField>
        </Field>
        <Field>
            <FieldLabel>Large</FieldLabel>
            <OTPField aria-label="One-time password, large size" defaultValue="418032" length={6} size="lg">
                {SLOTS.map((slotKey, index) => (
                    <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
                ))}
            </OTPField>
        </Field>
    </div>
);

export const Disabled = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418032" disabled length={6} size="lg">
            {SLOTS.map((slotKey, index) => (
                <OTPFieldInput aria-label={`Character ${index + 1} of 6`} key={slotKey} />
            ))}
        </OTPField>
        <FieldDescription>Verifying with YoStar…</FieldDescription>
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
