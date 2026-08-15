import { Field, FieldControl, FieldDescription, FieldLabel, Input, Textarea } from "frontend";

export const StyledInput = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>Tier list name</FieldLabel>
            <FieldControl defaultValue="Endgame DPS rankings" render={<Input />} />
            <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>Doctor nickname</FieldLabel>
            <FieldControl placeholder="e.g. Kal'tsit" render={<Input />} />
            <FieldDescription>Leave blank to keep the nickname from your last profile sync.</FieldDescription>
        </Field>
    </div>
);

export const AsTextarea = () => (
    <Field className="w-full max-w-md">
        <FieldLabel>Version notes</FieldLabel>
        <FieldControl defaultValue="Moved Mlynar to S after the module release; dropped Ch'en to A." render={<Textarea rows={3} />} />
        <FieldDescription>Shown in the changelog next to this version of the list.</FieldDescription>
    </Field>
);

export const Disabled = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field disabled>
            <FieldLabel>YoStar email</FieldLabel>
            <FieldControl defaultValue="doctor@rhodes.island" render={<Input />} type="email" />
            <FieldDescription>Locked while the one-time code is pending.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>Sanity per day</FieldLabel>
            <FieldControl defaultValue="240" render={<Input />} />
            <FieldDescription>Used by the planner to estimate farming time.</FieldDescription>
        </Field>
    </div>
);
