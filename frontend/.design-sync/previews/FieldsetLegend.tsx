import { Field, FieldDescription, FieldLabel, Fieldset, FieldsetLegend, Input, Switch } from "frontend";

export const WithDescription = () => (
    <Fieldset className="max-w-md">
        <div className="flex flex-col gap-1">
            <FieldsetLegend>Farming planner</FieldsetLegend>
            <p className="text-muted-foreground text-sm">Defaults the planner uses when estimating how long a material goal will take.</p>
        </div>
        <Field>
            <FieldLabel>Sanity per day</FieldLabel>
            <Input defaultValue="240" />
            <FieldDescription>Includes refreshes from Originite Prime.</FieldDescription>
        </Field>
        <Field>
            <FieldLabel>
                <Switch defaultChecked />
                Count event stage drops
            </FieldLabel>
        </Field>
    </Fieldset>
);

export const TwoGroups = () => (
    <div className="grid gap-6 sm:grid-cols-2">
        <Fieldset>
            <FieldsetLegend>Account</FieldsetLegend>
            <Field>
                <FieldLabel>YoStar email</FieldLabel>
                <Input defaultValue="doctor@rhodes.island" type="email" />
            </Field>
            <Field>
                <FieldLabel>Server</FieldLabel>
                <Input defaultValue="Global (EN)" />
            </Field>
        </Fieldset>

        <Fieldset>
            <FieldsetLegend>Display</FieldsetLegend>
            <Field>
                <FieldLabel>Doctor nickname</FieldLabel>
                <Input defaultValue="Kal'tsit" />
            </Field>
            <Field>
                <FieldLabel>Profile banner</FieldLabel>
                <Input defaultValue="Chapter 8 — Roaring Flare" />
            </Field>
        </Fieldset>
    </div>
);

export const Simple = () => (
    <Fieldset>
        <FieldsetLegend>Recruitment</FieldsetLegend>
        <Field>
            <FieldLabel>Recruitment time</FieldLabel>
            <Input defaultValue="09:00:00" />
            <FieldDescription>Longer timers unlock higher rarity pools.</FieldDescription>
        </Field>
    </Fieldset>
);
