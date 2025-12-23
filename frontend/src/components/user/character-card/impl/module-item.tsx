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
    const iconSize = isSmall ? 28 : 40;
    const containerClass = isSmall ? "flex items-center gap-2 rounded-md border p-2" : "flex items-center gap-3 rounded-lg border p-3";
    const imageClass = isSmall ? "h-7 w-7 shrink-0 object-contain" : "h-10 w-10 object-contain";

    return (
        <div className={`${containerClass} ${isEquipped ? "border-neutral-400 bg-neutral-100 dark:bg-neutral-800/30" : ""}`}>
            <Image alt="Module" className={imageClass} height={iconSize} src={module.image ? `/api/cdn${module.image}` : `/api/cdn/upk/spritepack/ui_equip_big_img_hub_0/${module.uniEquipIcon}.png`} unoptimized width={iconSize} />
            <div className={isSmall ? "min-w-0 flex-1" : "flex-1"}>
                {!isSmall && (
                    <div className="text-muted-foreground text-xs">
                        {module.typeName1} {module.typeName2 ? `(${module.typeName2})` : ""}
                    </div>
                )}
                <div className="truncate font-medium text-sm">
                    {module.uniEquipName}
                    {!isSmall && isEquipped && <span className="ml-2 text-neutral-500 text-xs">(Equipped)</span>}
                </div>
                <div className="text-muted-foreground text-xs">{isSmall ? `${module.typeName1} Lv.${moduleLevel}` : `Level ${moduleLevel}`}</div>
            </div>
        </div>
    );
}
