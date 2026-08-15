import { Checkbox, CheckboxGroup, Field, FieldDescription, FieldItem, FieldLabel, Radio, RadioGroup } from "frontend";

export const CheckboxRows = () => (
    <Field className="w-full max-w-md gap-3" name="notifications">
        <FieldLabel className="font-semibold">Notify me about</FieldLabel>
        <CheckboxGroup defaultValue={["banner", "maintenance"]}>
            <FieldItem className="items-start gap-2.5">
                <Checkbox className="mt-0.5" value="banner" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>New headhunting banner</FieldLabel>
                    <FieldDescription>Fires when a CN or Global banner gets a confirmed start date.</FieldDescription>
                </div>
            </FieldItem>
            <FieldItem className="items-start gap-2.5">
                <Checkbox className="mt-0.5" value="maintenance" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>Server maintenance</FieldLabel>
                    <FieldDescription>Sent a few hours before the EN servers go down.</FieldDescription>
                </div>
            </FieldItem>
            <FieldItem className="items-start gap-2.5">
                <Checkbox className="mt-0.5" value="birthdays" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>Operator birthdays</FieldLabel>
                    <FieldDescription>A daily digest of who is celebrating on Rhodes Island.</FieldDescription>
                </div>
            </FieldItem>
        </CheckboxGroup>
    </Field>
);

export const RadioRows = () => (
    <Field className="w-full max-w-md gap-3" name="visibility">
        <FieldLabel className="font-semibold">Profile visibility</FieldLabel>
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
            <FieldItem className="items-start gap-2.5">
                <Radio className="mt-0.5" value="private" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>Private</FieldLabel>
                    <FieldDescription>Only your nickname and Doctor level are visible.</FieldDescription>
                </div>
            </FieldItem>
        </RadioGroup>
    </Field>
);

export const DisabledItem = () => (
    <Field className="w-full max-w-md gap-3" name="sources">
        <FieldLabel className="font-semibold">Include drop sources</FieldLabel>
        <CheckboxGroup defaultValue={["main"]}>
            <FieldItem className="items-start gap-2.5">
                <Checkbox className="mt-0.5" value="main" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>Main theme stages</FieldLabel>
                    <FieldDescription>Chapters 0 through 14, including Adverse Environment.</FieldDescription>
                </div>
            </FieldItem>
            <FieldItem className="items-start gap-2.5" disabled>
                <Checkbox className="mt-0.5" value="rerun" />
                <div className="flex flex-col gap-1">
                    <FieldLabel>Event reruns</FieldLabel>
                    <FieldDescription>No rerun is live on the EN server right now.</FieldDescription>
                </div>
            </FieldItem>
        </CheckboxGroup>
    </Field>
);
