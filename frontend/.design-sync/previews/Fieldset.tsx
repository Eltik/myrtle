import { Checkbox, CheckboxGroup, Field, FieldDescription, FieldItem, FieldLabel, Fieldset, FieldsetLegend, Input, Radio, RadioGroup } from "frontend";

export const AccountDetails = () => (
    <Fieldset>
        <FieldsetLegend>Account</FieldsetLegend>
        <Field>
            <FieldLabel>YoStar email</FieldLabel>
            <Input defaultValue="doctor@rhodes.island" type="email" />
        </Field>
        <Field>
            <FieldLabel>Doctor nickname</FieldLabel>
            <Input defaultValue="Kal'tsit" />
        </Field>
    </Fieldset>
);

export const VisibilityOptions = () => (
    <Fieldset className="max-w-md">
        <FieldsetLegend>Profile visibility</FieldsetLegend>
        <Field name="visibility">
            <RadioGroup defaultValue="public">
                <FieldItem className="items-start gap-2.5">
                    <Radio className="mt-0.5" value="public" />
                    <div className="flex flex-col gap-1">
                        <FieldLabel>Public</FieldLabel>
                        <FieldDescription>Anyone can see your roster, depot and enemy handbook.</FieldDescription>
                    </div>
                </FieldItem>
                <FieldItem className="items-start gap-2.5">
                    <Radio className="mt-0.5" value="unlisted" />
                    <div className="flex flex-col gap-1">
                        <FieldLabel>Unlisted</FieldLabel>
                        <FieldDescription>Reachable by direct link, but hidden from Doctor search.</FieldDescription>
                    </div>
                </FieldItem>
            </RadioGroup>
        </Field>
    </Fieldset>
);

export const Disabled = () => (
    <Fieldset className="max-w-md" disabled>
        <FieldsetLegend>Depot sync</FieldsetLegend>
        <Field name="sources">
            <CheckboxGroup defaultValue={["depot"]}>
                <FieldItem className="items-start gap-2.5">
                    <Checkbox className="mt-0.5" value="depot" />
                    <div className="flex flex-col gap-1">
                        <FieldLabel>Inventory and materials</FieldLabel>
                        <FieldDescription>Sign in with your YoStar email to enable syncing.</FieldDescription>
                    </div>
                </FieldItem>
                <FieldItem className="items-start gap-2.5">
                    <Checkbox className="mt-0.5" value="roster" />
                    <div className="flex flex-col gap-1">
                        <FieldLabel>Operator roster</FieldLabel>
                        <FieldDescription>Includes E2 levels, skill masteries and modules.</FieldDescription>
                    </div>
                </FieldItem>
            </CheckboxGroup>
        </Field>
    </Fieldset>
);
