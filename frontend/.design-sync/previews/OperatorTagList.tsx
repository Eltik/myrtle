import { OperatorTagList } from "frontend";

// The recruitment tags one operator carries, as outline badges: position,
// class, rarity qualification, then the game's affix tags. It fills the hover
// card on a recruitment result and the phone-only expansion under a detailed
// row. An operator with no tags gets an italic "No tags" line.

/** A Top Operator: position, class, qualification, affixes. */
export const SixStar = () => <OperatorTagList tags={["Ranged", "Sniper", "Top Operator", "DPS", "Crowd-Control"]} />;

/** A 4★ with the usual three to four tags. */
export const FourStar = () => <OperatorTagList tags={["Ranged", "Supporter", "Slow", "Healing"]} />;

/** No tags. */
export const Empty = () => <OperatorTagList tags={[]} />;

/** As it sits in the hover card: "Tags" eyebrow over the list. */
export const InHoverCard = () => (
    <div className="flex w-max max-w-72 flex-col gap-2 rounded-lg border border-border bg-popover p-3 shadow-lg">
        <div className="font-medium text-[12px] text-muted-foreground">Tags</div>
        <OperatorTagList tags={["Ranged", "Caster", "Senior Operator", "DPS", "Healing", "Slow"]} />
    </div>
);
