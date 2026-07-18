import { useQuery } from "@tanstack/react-query";
import { createContext, type ReactNode, useCallback, useContext, useMemo } from "react";
import { chibiSkinKey } from "#/components/operators/detail/impl/skins";
import { useTheme } from "#/hooks/use-theme";
import { type IChibiCharacter, type IChibiSpineFiles, isCompleteSpineFiles, operatorChibisQueryOptions } from "#/lib/api/chibis";

export type ArtServer = "en" | "cn";

interface IDynamicArtContext {
    server: ArtServer;
    /**
     * Resolve the dynamic (`dyn_illust`) spine files for an operator's equipped
     * skin, or null when the "animate dynamic art" preference (localStorage, on
     * by default) is off, the catalog isn't loaded, or the operator/skin has no
     * complete dynamic set. `skinId` is the roster/skin id (e.g.
     * `char_1012_skadi2@boc#4` or null for the default/E2 art).
     */
    getDynamicFiles: (operatorCode: string | null | undefined, skinId: string | null | undefined) => IChibiSpineFiles | null;
}

const DynamicArtContext = createContext<IDynamicArtContext | null>(null);

/** Only en/cn have dynamic art unpacked; every other server maps to en (no matches). */
function toArtServer(server: string): ArtServer {
    return server === "cn" ? "cn" : "en";
}

export function DynamicArtProvider({ server, children }: { server: string; children: ReactNode }) {
    const { dynamicArtwork: enabled } = useTheme();
    const artServer = toArtServer(server);

    // Only fetch the (few-hundred-KB) catalog once the preference is on.
    const { data: catalog } = useQuery({ ...operatorChibisQueryOptions(artServer), enabled });

    const byOperator = useMemo(() => {
        const map = new Map<string, IChibiCharacter>();
        for (const char of catalog?.characters ?? []) map.set(char.operatorCode, char);
        return map;
    }, [catalog]);

    const getDynamicFiles = useCallback<IDynamicArtContext["getDynamicFiles"]>(
        (operatorCode, skinId) => {
            if (!enabled || !operatorCode) return null;
            const char = byOperator.get(operatorCode);
            if (!char) return null;

            // The equipped skin's key ("default" -> the E2 illustration, which
            // matches the E2 static art shown on these surfaces). Match strictly
            // so an outfit without dynamic art never borrows the default's.
            const key = chibiSkinKey(skinId ?? "").toLowerCase();
            const skin = char.skins.find((s) => s.name.toLowerCase() === key);
            const dyn = skin?.animationTypes.dynamic;
            return isCompleteSpineFiles(dyn) ? dyn : null;
        },
        [enabled, byOperator],
    );

    const value = useMemo<IDynamicArtContext>(() => ({ server: artServer, getDynamicFiles }), [artServer, getDynamicFiles]);

    return <DynamicArtContext.Provider value={value}>{children}</DynamicArtContext.Provider>;
}

/** Returns the dynamic-art context, or null when rendered outside the provider. */
export function useDynamicArt(): IDynamicArtContext | null {
    return useContext(DynamicArtContext);
}
