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
     * `char_1012_skadi2@boc#4`, `char_1012_skadi2#1`, or null for the default art).
     * `elite` is the owner's promotion of that operator; the default dynamic set is the
     * E2 illustration, so it is withheld when the static art shown is the pre-E2 one
     * (see `defaultArtIsE2`). Leave it undefined on surfaces that show every skin at E2.
     */
    getDynamicFiles: (operatorCode: string | null | undefined, skinId: string | null | undefined, elite?: number | null) => IChibiSpineFiles | null;
}

const DynamicArtContext = createContext<IDynamicArtContext | null>(null);

/** Only en/cn have dynamic art unpacked; every other server maps to en (no matches). */
function toArtServer(server: string): ArtServer {
    return server === "cn" ? "cn" : "en";
}

/**
 * Whether the DEFAULT (non-outfit) art an owner sees is the E2 illustration, which is the
 * only default art with a dynamic set. Mirrors the roster's static-art rule
 * (`ownedHeroURL`): a `#n` template suffix names the phase art outright (`#1` is the E0/E1
 * art, `#2` the E2 one, whatever the promotion), otherwise the promotion decides. An
 * unknown promotion keeps the old answer, E2, for surfaces that show the catalog rather
 * than a roster.
 */
export function defaultArtIsE2(skinId: string | null | undefined, elite: number | null | undefined): boolean {
    if (skinId?.endsWith("_e2")) return true;
    const tmpl = skinId?.match(/#(\d+)$/);
    if (tmpl) return Number(tmpl[1]) >= 2;
    return elite == null || elite >= 2;
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
        (operatorCode, skinId, elite) => {
            if (!enabled || !operatorCode) return null;
            const char = byOperator.get(operatorCode);
            if (!char) return null;

            // The equipped skin's key ("default" -> the E2 illustration). Match strictly
            // so an outfit without dynamic art never borrows the default's, and withhold
            // the default set when the static art under it is the pre-E2 one: an owner
            // who has not promoted the operator sees the E1 art, and the E2 animation
            // played over it (the /user/{id} bug, 2026-09-10).
            const key = chibiSkinKey(skinId ?? "").toLowerCase();
            if (key === "default" && !defaultArtIsE2(skinId, elite)) return null;
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
