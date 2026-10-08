import { OperatorMultiCombobox } from "frontend";
import { type ReactNode, useEffect, useRef, useState } from "react";

// OperatorMultiCombobox picks several operators for the user search's "Owns
// operators" filter: chips inside a search-addon input (up to 20), the same
// avatar list popup as the single picker. The value is the ids. The fixture
// is 14 real `/api/operators/index` rows, rarity first as the toolbar sorts.

const OPERATORS = [
    {"id": "char_003_kalts", "name": "Kal'tsit", "appellation": " ", "rarity": 6, "profession": "MEDIC", "subProfessionId": "physician", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Medic", "subProfessionName": "Medic"},
    {"id": "char_1012_skadi2", "name": "Skadi the Corrupting Heart", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "bard", "position": "RANGED", "nationId": "egir", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Bard"},
    {"id": "char_4064_mlynar", "name": "Młynar", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "librator", "position": "MELEE", "nationId": "kazimierz", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Liberator"},
    {"id": "char_293_thorns", "name": "Thorns", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "lord", "position": "MELEE", "nationId": "iberia", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Lord"},
    {"id": "char_2024_chyue", "name": "Chongyue", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "fighter", "position": "MELEE", "nationId": "yan", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Fighter"},
    {"id": "char_1035_wisdel", "name": "Wiš'adel", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "bombarder", "position": "RANGED", "nationId": "", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Flinger"},
    {"id": "char_4087_ines", "name": "Ines", "appellation": " ", "rarity": 6, "profession": "PIONEER", "subProfessionId": "agent", "position": "MELEE", "nationId": "", "isNotObtainable": false, "professionName": "Vanguard", "subProfessionName": "Agent"},
    {"id": "char_291_aglina", "name": "Angelina", "appellation": " ", "rarity": 6, "profession": "SUPPORT", "subProfessionId": "slower", "position": "RANGED", "nationId": "siracusa", "isNotObtainable": false, "professionName": "Supporter", "subProfessionName": "Decel Binder"},
    {"id": "char_017_huang", "name": "Blaze", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "centurion", "position": "MELEE", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Centurion"},
    {"id": "char_172_svrash", "name": "SilverAsh", "appellation": " ", "rarity": 6, "profession": "WARRIOR", "subProfessionId": "lord", "position": "MELEE", "nationId": "kjerag", "isNotObtainable": false, "professionName": "Guard", "subProfessionName": "Lord"},
    {"id": "char_103_angel", "name": "Exusiai", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "fastshot", "position": "RANGED", "nationId": "lungmen", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Marksman"},
    {"id": "char_113_cqbw", "name": "W", "appellation": " ", "rarity": 6, "profession": "SNIPER", "subProfessionId": "aoesniper", "position": "RANGED", "nationId": "", "isNotObtainable": false, "professionName": "Sniper", "subProfessionName": "Artilleryman"},
    {"id": "char_002_amiya", "name": "Amiya", "appellation": " ", "rarity": 5, "profession": "CASTER", "subProfessionId": "corecaster", "position": "RANGED", "nationId": "rhodes", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Core Caster"},
    {"id": "char_213_mostma", "name": "Mostima", "appellation": " ", "rarity": 6, "profession": "CASTER", "subProfessionId": "splashcaster", "position": "RANGED", "nationId": "laterano", "isNotObtainable": false, "professionName": "Caster", "subProfessionName": "Splash Caster"},
];

/** Clicks the first input after first paint (Base UI wires it late), then drops the focus ring. */
function OpenOnMount({ children }: { children: ReactNode }) {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let f2 = 0;
        const f1 = requestAnimationFrame(() => {
            f2 = requestAnimationFrame(() => {
                const input = ref.current?.querySelector<HTMLInputElement>("input");
                input?.click();
                input?.dispatchEvent(new KeyboardEvent("keydown", { key: "ArrowDown", bubbles: true }));
            });
        });
        return () => {
            cancelAnimationFrame(f1);
            cancelAnimationFrame(f2);
        };
    }, []);
    return (
        <div ref={ref} style={{ minHeight: 560, width: 380 }} className="relative">
            {children}
        </div>
    );
}

function Picker({ initial, operators = OPERATORS, loading = false }: { initial: string[]; operators?: typeof OPERATORS; loading?: boolean }) {
    const [value, setValue] = useState(initial);
    return <OperatorMultiCombobox operators={(loading ? undefined : operators) as never} value={value} onChange={setValue} id="owns-has" label="Owns operators" placeholder="Add an operator…" className="w-full" />;
}

/** Nothing picked: the placeholder. */
export const Empty = () => (
    <div style={{ width: 380 }}>
        <Picker initial={[]} />
    </div>
);

/** Three operators as chips. */
export const ThreeChosen = () => (
    <div style={{ width: 380 }}>
        <Picker initial={["char_003_kalts", "char_1012_skadi2", "char_4064_mlynar"]} />
    </div>
);

/** The index still loading. */
export const Loading = () => (
    <div style={{ width: 380 }}>
        <Picker initial={[]} loading />
    </div>
);

/** Open with two chosen (six rows, every lazy avatar in view for the capture): the list beneath, chosen rows marked. */
export const Open = () => (
    <OpenOnMount>
        <Picker initial={["char_293_thorns", "char_2024_chyue"]} operators={OPERATORS.slice(0, 6)} />
    </OpenOnMount>
);
