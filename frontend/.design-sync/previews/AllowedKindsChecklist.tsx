import { AllowedKindsChecklist } from "frontend";
import { useState } from "react";

// The grid's allowed-types checklist, as the create dialog and the editor's
// types dialog show it: one checkbox per tier-list entity kind, the last ticked
// kind never unticked. Stateful wrapper so the boxes respond the way they do in
// the product.

type Kind = "operator" | "class" | "subclass" | "enemy" | "event" | "faction" | "stronghold_bond" | "skin" | "module" | "skill" | "integrated_strategies" | "story_sprite" | "main_story";

const Live = ({ initial, placed, disabled }: { initial: Kind[]; placed?: Partial<Record<Kind, number>>; disabled?: boolean }) => {
    const [value, setValue] = useState<Kind[]>(initial);
    return (
        <div className="w-full max-w-xl p-4">
            <AllowedKindsChecklist value={value} onChange={setValue} placedByKind={placed} disabled={disabled} />
        </div>
    );
};

/** A fresh grid: operators only, the default a new grid starts with. */
export const OperatorsOnly = () => <Live initial={["operator"]} />;

/** An editor's grid with picks placed: the count beside each kind is how many cells hold one. */
export const WithPlacedPicks = () => <Live initial={["operator", "skin", "enemy", "event"]} placed={{ operator: 7, skin: 2, enemy: 3, event: 1 }} />;

/** Every kind ticked. */
export const AllKinds = () => <Live initial={["operator", "class", "subclass", "enemy", "event", "faction", "stronghold_bond", "skin", "module", "skill", "integrated_strategies", "story_sprite", "main_story"]} />;

/** Locked while a save is in flight. */
export const Disabled = () => <Live initial={["operator", "enemy"]} placed={{ operator: 4, enemy: 2 }} disabled />;
