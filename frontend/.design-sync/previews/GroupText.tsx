import { Button, Group, GroupText, Input } from "frontend";
import { Hash, Zap } from "lucide-react";

export const AsPrefix = () => (
    <Group>
        <GroupText>myrtle.moe/u/</GroupText>
        <Input className="w-56" defaultValue="dr-kaltsit" />
    </Group>
);

export const AsSuffix = () => (
    <Group>
        <Input className="w-32" defaultValue="240" />
        <GroupText>sanity / day</GroupText>
    </Group>
);

export const WithIcon = () => (
    <div className="flex flex-col gap-3">
        <Group>
            <GroupText>
                <Zap />
                Sanity
            </GroupText>
            <Input className="w-24" defaultValue="135" />
        </Group>
        <Group>
            <GroupText>
                <Hash />
                Doctor UID
            </GroupText>
            <Input className="w-40" defaultValue="10023481" />
        </Group>
    </div>
);

export const BetweenButtons = () => (
    <Group>
        <Button variant="outline">Prev</Button>
        <GroupText>Page 3 of 12</GroupText>
        <Button variant="outline">Next</Button>
    </Group>
);
