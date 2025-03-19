import Image from "next/image";
import { Badge } from "~/components/ui/badge";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "~/components/ui/tooltip";
import type { Item } from "~/types/impl/api/static/material";
import type { MaterialCost } from "~/types/impl/frontend/impl/operators";
import { fetchMaterial } from "./helper";

interface MaterialItemProps {
    material: MaterialCost;
    materials: Item[];
}

export const MaterialItem = ({ material, materials }: MaterialItemProps) => {
    const materialData = fetchMaterial(material.material.itemId, materials);

    return (
        <TooltipProvider key={material.material.itemId}>
            <Tooltip>
                <TooltipTrigger asChild>
                    <div className="flex flex-col items-center">
                        <div className="relative">
                            {materialData ? (
                                <Image src={`https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/item/${materialData.iconId}.png`} alt={materialData.name ?? material.material.name} width={50} height={50} className="rounded-md" />
                            ) : (
                                <div className="flex h-[50px] w-[50px] items-center justify-center rounded-md bg-gray-200">
                                    <span className="text-xs">Loading...</span>
                                </div>
                            )}
                            <Badge className="absolute -bottom-2 -right-2 flex h-5 min-w-5 items-center justify-center rounded-full p-0 px-1 text-xs">{material.quantity}</Badge>
                        </div>
                        <span className="mt-2 line-clamp-1 text-xs">{materialData?.name ?? material.material.name}</span>
                    </div>
                </TooltipTrigger>
                <TooltipContent>
                    <p>
                        {materialData?.name ?? material.material.name} x{material.quantity}
                    </p>
                </TooltipContent>
            </Tooltip>
        </TooltipProvider>
    );
};
