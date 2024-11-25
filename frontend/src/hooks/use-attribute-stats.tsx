import { useState, useCallback } from "react";
import { getAttributeStats } from "~/helper";
import type { CharacterData } from "~/types/impl/api";
import type { ModuleData } from "~/types/impl/api/static/modules";
import type { Operator } from "~/types/impl/api/static/operator";

export function useAttributeStats(data: CharacterData) {
    const [attributeStats, setAttributeStats] = useState<Operator["phases"][number]["attributesKeyFrames"][number]["data"] | null>(null);

    const fetchAttributeStats = useCallback(async () => {
        if (data.currentEquip) {
            const moduleData = (await (
                await fetch("/api/static", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        type: "modules",
                        id: data.currentEquip,
                        method: "details",
                    }),
                })
            ).json()) as { details: ModuleData };

            const battleEquip = {
                [data.currentEquip]: moduleData.details,
            };

            const stats = getAttributeStats(data, data.level, battleEquip);
            setAttributeStats(stats);
        } else {
            const stats = getAttributeStats(data, data.level);
            setAttributeStats(stats);
        }
    }, [data]);

    return { attributeStats, fetchAttributeStats };
}
