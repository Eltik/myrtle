import Image from "next/image";
import type { UserCharacterModule } from "~/types/api/impl/user";

interface ModuleItemProps {
    module: UserCharacterModule;
    moduleLevel: number;
    isEquipped: boolean;
    size?: "small" | "large";
}

export function ModuleItem({ module, moduleLevel, isEquipped, size = "small" }: ModuleItemProps) {
    const isSmall = size === "small";

    return (
        <div className={`flex items-center gap-2 rounded-md px-2.5 py-1.5 ${isEquipped ? "border border-primary/30 bg-primary/5 shadow-[0_0_8px_rgba(var(--primary),0.15)]" : "bg-muted/30"}`}>
            <Image alt="Module" className={isSmall ? "h-6 w-6 shrink-0 object-contain" : "h-7 w-7 shrink-0 object-contain"} height={isSmall ? 24 : 28} src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`} unoptimized width={isSmall ? 24 : 28} />
            <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5">
                    <span className="truncate font-medium text-sm" title={module.uniEquipName}>
                        {module.uniEquipName}
                    </span>
                    {isEquipped && <span className="shrink-0 text-[10px] text-muted-foreground">(Equipped)</span>}
                </div>
            </div>
            <div className="flex shrink-0 items-center gap-1.5 text-muted-foreground text-xs">
                <span>{module.typeName1}</span>
                <span>Lv.{moduleLevel}</span>
            </div>
        </div>
    );
}
