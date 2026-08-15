import { Field, FieldDescription, FieldLabel, Input, Switch, Textarea } from "frontend";

export const UnderInput = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>Name</FieldLabel>
            <Input defaultValue="Endgame DPS rankings" />
            <FieldDescription>Shown on browse cards and on the public detail page.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>Description</FieldLabel>
            <Textarea defaultValue="Ranks every 6-star DPS operator at E2 L90, S3M3, no module." rows={2} />
            <FieldDescription>Optional. A sentence or two helps readers know what to expect.</FieldDescription>
        </Field>
    </div>
);

export const OnToggleRow = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>
                <Switch defaultChecked />
                Sync depot automatically
            </FieldLabel>
            <FieldDescription>Pulls your inventory from YoStar once every 12 hours. Manual resync is always available.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>
                <Switch />
                Show CN release dates
            </FieldLabel>
            <FieldDescription>Adds the CN debut alongside each Global release date.</FieldDescription>
        </Field>
    </div>
);

export const AsHelperText = () => (
    <div className="flex w-full max-w-md flex-col gap-5">
        <Field>
            <FieldLabel>YoStar email</FieldLabel>
            <Input placeholder="doctor@rhodes.island" type="email" />
            <FieldDescription>We send a one-time code to this address. No login information is stored on the server.</FieldDescription>
        </Field>

        <Field>
            <FieldLabel>Sanity per day</FieldLabel>
            <Input defaultValue="240" />
            <FieldDescription>Used by the planner to estimate how many days a farming goal will take.</FieldDescription>
        </Field>
    </div>
);
