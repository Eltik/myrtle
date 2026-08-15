import { Field, FieldDescription, FieldLabel, OTPField, OTPFieldInput, OTPFieldSeparator } from "frontend";

export const Grouped = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418032" length={6} size="lg">
            <OTPFieldInput aria-label="Character 1 of 6" />
            <OTPFieldInput aria-label="Character 2 of 6" />
            <OTPFieldInput aria-label="Character 3 of 6" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 4 of 6" />
            <OTPFieldInput aria-label="Character 5 of 6" />
            <OTPFieldInput aria-label="Character 6 of 6" />
        </OTPField>
        <FieldDescription>Splitting 6 digits into two blocks of 3 makes them easier to read back.</FieldDescription>
    </Field>
);

export const Empty = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" length={6} size="lg">
            <OTPFieldInput aria-label="Character 1 of 6" />
            <OTPFieldInput aria-label="Character 2 of 6" />
            <OTPFieldInput aria-label="Character 3 of 6" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 4 of 6" />
            <OTPFieldInput aria-label="Character 5 of 6" />
            <OTPFieldInput aria-label="Character 6 of 6" />
        </OTPField>
        <FieldDescription>We sent a 6-digit code to doctor@rhodes.island.</FieldDescription>
    </Field>
);

export const DefaultSize = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Code</FieldLabel>
        <OTPField aria-label="One-time password" defaultValue="418032" length={6}>
            <OTPFieldInput aria-label="Character 1 of 6" />
            <OTPFieldInput aria-label="Character 2 of 6" />
            <OTPFieldInput aria-label="Character 3 of 6" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 4 of 6" />
            <OTPFieldInput aria-label="Character 5 of 6" />
            <OTPFieldInput aria-label="Character 6 of 6" />
        </OTPField>
        <FieldDescription>The separator scales with the field size.</FieldDescription>
    </Field>
);

export const TwoBlocks = () => (
    <Field className="w-full max-w-sm">
        <FieldLabel>Support code</FieldLabel>
        <OTPField aria-label="Support code" defaultValue="41803274" length={8}>
            <OTPFieldInput aria-label="Character 1 of 8" />
            <OTPFieldInput aria-label="Character 2 of 8" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 3 of 8" />
            <OTPFieldInput aria-label="Character 4 of 8" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 5 of 8" />
            <OTPFieldInput aria-label="Character 6 of 8" />
            <OTPFieldSeparator />
            <OTPFieldInput aria-label="Character 7 of 8" />
            <OTPFieldInput aria-label="Character 8 of 8" />
        </OTPField>
        <FieldDescription>Longer codes group in pairs.</FieldDescription>
    </Field>
);
