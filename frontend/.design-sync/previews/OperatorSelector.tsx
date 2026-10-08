import { OperatorSelector } from "frontend";
import { type ReactNode, useEffect, useRef, useState } from "react";

// The plan dialog's single operator search. Its rows come ranked from
// `useOperatorOptions` (upcoming CN operators first, then rarity, then name),
// so the combobox's own filter is off. Each row is an avatar, the name and
// "6★ · Guard". Ids from `/api/operators/index`.

type Option = { id: string; name: string; displayName: string; appellation: string; rarity: number; profession: string; subProfessionId: string; tagList: string[]; nationId: string; isUpcoming: boolean };
const option = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, nationId: string, tagList: string[]): Option => ({ id, name, displayName: name, appellation: " ", rarity, profession, subProfessionId, tagList, nationId, isUpcoming: false });

const OPTIONS: Option[] = [
    option("char_2024_chyue", "Chongyue", 6, "WARRIOR", "fighter", "yan", ["Nuker"]),
    option("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "leithanien", ["DPS", "Debuff"]),
    option("char_4009_irene", "Irene", 6, "WARRIOR", "sword", "iberia", ["Nuker", "DPS", "Crowd-Control"]),
    option("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "kazimierz", ["DPS", "Nuker"]),
    option("char_249_mlyss", "Muelsyse", 6, "PIONEER", "tactician", "columbia", ["DP-Recovery", "Crowd-Control"]),
    option("char_202_demkni", "Saria", 6, "TANK", "guardian", "columbia", ["Defense", "Healing", "Support"]),
    option("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "egir", ["DPS", "Survival"]),
    option("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "lungmen", ["Fast-Redeploy", "DPS"]),
    option("char_293_thorns", "Thorns", 6, "WARRIOR", "lord", "iberia", ["DPS", "Defense"]),
    option("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "columbia", ["Healing", "Support"]),
    option("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "lungmen", ["DP-Recovery", "Crowd-Control"]),
];

const noop = () => undefined;

/** Presses the chevron trigger after Base UI has wired it, which opens the list, as a user's press does. */
function OpenOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const trigger = ref.current?.querySelector<HTMLElement>('[data-slot="combobox-trigger"]');
                if (!trigger) return;
                // Base UI opens on the press, not the click: send the whole sequence.
                for (const type of ["pointerdown", "mousedown", "pointerup", "mouseup", "click"]) {
                    const Ctor = type.startsWith("pointer") ? PointerEvent : MouseEvent;
                    trigger.dispatchEvent(new Ctor(type, { bubbles: true, cancelable: true, button: 0 }));
                }
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, []);
    return (
        <div className="max-w-md" ref={ref} style={{ height: 520 }}>
            {children}
        </div>
    );
}

function Picker({ initial, isLoading = false, options = OPTIONS }: { initial: string | null; isLoading?: boolean; options?: Option[] }) {
    const [selectedId, setSelectedId] = useState<string | null>(initial);
    const selected = options.find((o) => o.id === selectedId) ?? null;
    return <OperatorSelector options={options} selectedOption={selected} isLoading={isLoading} onSelect={setSelectedId} onSearchQueryChange={noop} />;
}

/** Nothing picked yet: the search placeholder. */
export const Empty = () => (
    <div className="max-w-md">
        <Picker initial={null} />
    </div>
);

/** Młynar picked: the input shows his name. */
export const Selected = () => (
    <div className="max-w-md">
        <Picker initial="char_4064_mlynar" />
    </div>
);

/** The index is still loading. */
export const Loading = () => (
    <div className="max-w-md">
        <Picker initial={null} isLoading options={[]} />
    </div>
);

// An open list stops at what the popup shows without scrolling: an avatar
// scrolled out of view never loads (`loading="lazy"`) and hangs the capture.
const VISIBLE = OPTIONS.slice(0, 5);

/** Open: avatar rows, rarity then name, Młynar ticked as the current pick. */
export const OpenList = () => (
    <OpenOnMount>
        <Picker initial="char_4064_mlynar" options={VISIBLE} />
    </OpenOnMount>
);
