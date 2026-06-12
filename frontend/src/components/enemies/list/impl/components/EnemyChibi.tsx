import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { DynamicChibiViewer } from "#/components/operators/detail/impl/components/chibi/ChibiViewer.lazy";
import { Skeleton } from "#/components/ui/skeleton";
import { enemyChibisQueryOptions } from "#/lib/api/chibis";

interface IEnemyChibiTabProps {
    enemyId: string;
}

export function EnemyChibiTab({ enemyId }: IEnemyChibiTabProps) {
    const { data, isLoading, isError } = useQuery(enemyChibisQueryOptions());

    const character = useMemo(() => data?.characters.find((c) => c.operatorCode === enemyId) ?? null, [data, enemyId]);
    const skin = character?.skins.find((s) => s.name === "default") ?? character?.skins[0] ?? null;

    if (isLoading) {
        return (
            <div className="flex flex-col gap-3 p-4 sm:p-5">
                <Skeleton className="h-45 w-full rounded-md" />
            </div>
        );
    }

    if (isError || !character || !skin) {
        return (
            <div className="p-10 text-center">
                <p className="m-0 font-sans text-[13px] text-muted-foreground leading-normal">{isError ? "Failed to load chibi data." : "No chibi available for this enemy."}</p>
            </div>
        );
    }

    return (
        <div className="flex flex-col gap-3 p-4 sm:p-5">
            <DynamicChibiViewer chibi={character} skin={skin} />
        </div>
    );
}
