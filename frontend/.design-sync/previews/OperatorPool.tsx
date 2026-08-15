import { DragControllerProvider, OperatorPool } from "frontend";
import { type ReactNode, useEffect, useRef } from "react";

/** The fields of `IOperatorIndexEntry` the pool actually reads. Ids verified against /api/operators/index. */
interface IPoolEntry {
    id: string;
    name: string;
    appellation: string;
    rarity: number;
    profession: string;
    subProfessionId: string;
    position: string;
    nationId: string;
    isNotObtainable: boolean;
}

const op = (id: string, name: string, rarity: number, profession: string, subProfessionId: string, position: string, nationId: string): IPoolEntry => ({
    id,
    name,
    appellation: "",
    rarity,
    profession,
    subProfessionId,
    position,
    nationId,
    isNotObtainable: false,
});

const ROSTER: IPoolEntry[] = [
    op("char_1035_wisdel", "Wiš'adel", 6, "SNIPER", "bombarder", "RANGED", ""),
    op("char_4064_mlynar", "Młynar", 6, "WARRIOR", "librator", "MELEE", "kazimierz"),
    op("char_1028_texas2", "Texas the Omertosa", 6, "SPECIAL", "executor", "MELEE", "lungmen"),
    op("char_4087_ines", "Ines", 6, "PIONEER", "agent", "MELEE", ""),
    op("char_350_surtr", "Surtr", 6, "WARRIOR", "artsfghter", "MELEE", "rhodes"),
    op("char_4116_blkkgt", "Degenbrecher", 6, "WARRIOR", "sword", "MELEE", "kjerag"),
    op("char_2012_typhon", "Typhon", 6, "SNIPER", "siegesniper", "RANGED", "sami"),
    op("char_377_gdglow", "Goldenglow", 6, "CASTER", "funnel", "RANGED", "victoria"),
    op("char_103_angel", "Exusiai", 6, "SNIPER", "fastshot", "RANGED", "lungmen"),
    op("char_180_amgoat", "Eyjafjalla", 6, "CASTER", "corecaster", "RANGED", "leithanien"),
    op("char_293_thorns", "Thorns", 6, "WARRIOR", "lord", "MELEE", "iberia"),
    op("char_263_skadi", "Skadi", 6, "WARRIOR", "fearless", "MELEE", "egir"),
    op("char_017_huang", "Blaze", 6, "WARRIOR", "centurion", "MELEE", "rhodes"),
    op("char_4133_logos", "Logos", 6, "CASTER", "corecaster", "RANGED", "rhodes"),
    op("char_2023_ling", "Ling", 6, "SUPPORT", "summoner", "RANGED", "yan"),
    op("char_311_mudrok", "Mudrock", 6, "TANK", "unyield", "MELEE", "rhodes"),
    op("char_4039_horn", "Horn", 6, "TANK", "fortress", "MELEE", "victoria"),
    op("char_358_lisa", "Suzuran", 6, "SUPPORT", "slower", "RANGED", "siracusa"),
    op("char_003_kalts", "Kal'tsit", 6, "MEDIC", "physician", "RANGED", "rhodes"),
    op("char_179_cgbird", "Nightingale", 6, "MEDIC", "ringhealer", "RANGED", ""),
    op("char_202_demkni", "Saria", 6, "TANK", "guardian", "MELEE", "columbia"),
    op("char_222_bpipe", "Bagpipe", 6, "PIONEER", "charger", "MELEE", "victoria"),
    op("char_102_texas", "Texas", 5, "PIONEER", "pioneer", "MELEE", "lungmen"),
    op("char_128_plosis", "Ptilopsis", 5, "MEDIC", "ringhealer", "RANGED", "columbia"),
    op("char_140_whitew", "Lappland", 5, "WARRIOR", "lord", "MELEE", "siracusa"),
    op("char_143_ghost", "Specter", 5, "WARRIOR", "centurion", "MELEE", "egir"),
    op("char_199_yak", "Matterhorn", 4, "TANK", "protector", "MELEE", "kjerag"),
    op("char_151_myrtle", "Myrtle", 4, "PIONEER", "bearer", "MELEE", "rhodes"),
];

const operatorById = Object.fromEntries(ROSTER.map((entry) => [entry.id, { ...entry, appellation: null, subOrder: 0, description: null, updatedAt: "2024-05-12T09:20:00.000Z" }]));

const PLACED = new Set(["char_1035_wisdel", "char_4064_mlynar", "char_1028_texas2", "char_4087_ines", "char_350_surtr", "char_4116_blkkgt", "char_2012_typhon", "char_377_gdglow", "char_103_angel"]);

const noop = () => {};

/** The editor's sidebar column: the pool only renders inside the drag controller's context. */
const Sidebar = ({ children }: { children: ReactNode }) => (
    <DragControllerProvider operatorById={operatorById} onPlace={noop} onUnplace={noop}>
        <div className="w-80">{children}</div>
    </DragControllerProvider>
);

/** Opens an overlay whose trigger is internal — deferred two frames so Base UI has wired the button. */
const ClickOnMount = ({ label, children }: { label: string; children: ReactNode }) => {
    const ref = useRef<HTMLDivElement>(null);
    useEffect(() => {
        let inner = 0;
        const outer = requestAnimationFrame(() => {
            inner = requestAnimationFrame(() => {
                const buttons = Array.from(ref.current?.querySelectorAll("button") ?? []);
                buttons.find((b) => (b.textContent ?? "").includes(label))?.click();
            });
        });
        return () => {
            cancelAnimationFrame(outer);
            cancelAnimationFrame(inner);
        };
    }, [label]);
    return (
        <div ref={ref} className="relative min-h-[520px]">
            {children}
        </div>
    );
};

export const Default = () => (
    <Sidebar>
        <OperatorPool operators={ROSTER} placedIds={new Set()} onUnplace={noop} onPickerActivate={noop} rootClassName="h-96" />
    </Sidebar>
);

export const WithPlacements = () => (
    <Sidebar>
        <OperatorPool operators={ROSTER} placedIds={PLACED} onUnplace={noop} onPickerActivate={noop} rootClassName="h-96" />
    </Sidebar>
);

export const NoMatches = () => (
    <Sidebar>
        <OperatorPool operators={ROSTER} placedIds={new Set(ROSTER.map((entry) => entry.id))} onUnplace={noop} onPickerActivate={noop} rootClassName="h-96" />
    </Sidebar>
);

export const ExpandedPicker = () => (
    <ClickOnMount label="Expand">
        <DragControllerProvider operatorById={operatorById} onPlace={noop} onUnplace={noop}>
            <div className="w-80">
                <OperatorPool operators={ROSTER} placedIds={PLACED} onUnplace={noop} onPickerActivate={noop} rootClassName="h-40" />
            </div>
        </DragControllerProvider>
    </ClickOnMount>
);
