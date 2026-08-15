import { SubProfessionIcon } from "frontend";

const GUARD_ARCHETYPES = [
    { id: "librator", label: "Liberator Guard" },
    { id: "lord", label: "Lord Guard" },
    { id: "fearless", label: "Dreadnought Guard" },
    { id: "centurion", label: "Centurion Guard" },
    { id: "sword", label: "Swordmaster Guard" },
    { id: "musha", label: "Soloblade Guard" },
    { id: "reaper", label: "Reaper Guard" },
    { id: "instructor", label: "Instructor Guard" },
];

export const GuardArchetypes = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Guard archetypes</div>
        <div className="grid grid-cols-4 gap-x-6 gap-y-4">
            {GUARD_ARCHETYPES.map((a) => (
                <div key={a.id} className="flex flex-col items-center gap-1.5">
                    <SubProfessionIcon subProfession={a.id} size={26} />
                    <span className="text-[11px] text-muted-foreground leading-none">{a.label.replace(" Guard", "")}</span>
                </div>
            ))}
        </div>
    </div>
);

export const ArchetypeOptions = () => (
    <div className="w-72 overflow-hidden rounded-lg border border-border bg-popover p-1 shadow-md">
        <div className="px-2 py-1.5 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">Sniper</div>
        {[
            { id: "fastshot", label: "Marksman Sniper" },
            { id: "bombarder", label: "Flinger Sniper" },
            { id: "aoesniper", label: "Artilleryman Sniper" },
            { id: "siegesniper", label: "Besieger Sniper" },
        ].map((a, i) => (
            <div key={a.id} className={i === 1 ? "flex items-center gap-2 rounded-md bg-accent px-2 py-1.5 text-foreground text-sm" : "flex items-center gap-2 rounded-md px-2 py-1.5 text-foreground text-sm"}>
                <SubProfessionIcon subProfession={a.id} size={18} />
                <span>{a.label}</span>
            </div>
        ))}
    </div>
);

export const Sizes = () => (
    <div className="flex w-fit items-end gap-6 rounded-xl border border-border bg-card p-4">
        {[14, 18, 26, 36].map((size) => (
            <div key={size} className="flex flex-col items-center gap-2">
                <SubProfessionIcon subProfession="tactician" size={size} />
                <span className="font-mono text-[10px] text-muted-foreground leading-none">{size}px</span>
            </div>
        ))}
    </div>
);

export const AcrossClasses = () => (
    <div className="w-fit rounded-xl border border-border bg-card p-4">
        <div className="mb-3 font-medium font-mono text-[10px] text-muted-foreground uppercase tracking-[0.12em]">One archetype per class</div>
        <div className="flex flex-col gap-2.5">
            {[
                { id: "tactician", label: "Tactician Vanguard", op: "Muelsyse" },
                { id: "protector", label: "Protector Defender", op: "Hoshiguma" },
                { id: "corecaster", label: "Core Caster", op: "Eyjafjalla" },
                { id: "ringhealer", label: "Multi-target Medic", op: "Ptilopsis" },
                { id: "geek", label: "Geek Specialist", op: "Aak" },
            ].map((a) => (
                <div key={a.id} className="flex items-center gap-2.5">
                    <SubProfessionIcon subProfession={a.id} size={20} />
                    <span className="w-44 text-foreground text-sm">{a.label}</span>
                    <span className="font-mono text-[11px] text-muted-foreground">{a.op}</span>
                </div>
            ))}
        </div>
    </div>
);
