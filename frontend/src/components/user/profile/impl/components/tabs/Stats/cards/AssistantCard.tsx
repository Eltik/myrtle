import { Sparkles } from "lucide-react";
import { useState } from "react";
import { operatorHero, skinTexture } from "#/components/operators/detail/impl/assets";
import { formatProfession, rarityToNumber } from "#/lib/utils";
import type { IOperatorListItem } from "#/types/operators";
import { DynamicArtOverlay } from "../../../../DynamicArtOverlay";
import { KICKER_TEXT, StatCard } from "../primitives";

interface IAssistantCardProps {
    secretary: string | null;
    secretarySkinId: string | null;
    server: string;
    operatorsStatic: IOperatorListItem[];
}

/**
 * Showcases the player's assistant/secretary operator as a wide banner. Shows
 * the static illustration by default; when the page-wide dynamic-art toggle is
 * on and the assistant has L2D art, {@link DynamicArtOverlay} animates it in
 * place (the static image fades out once the animation is rendering).
 */
export function AssistantCard({ secretary, secretarySkinId, server, operatorsStatic }: IAssistantCardProps) {
    const [errored, setErrored] = useState(false);
    const [dynActive, setDynActive] = useState(false);

    if (!secretary || errored) return null;

    const op = operatorsStatic.find((o) => o.id === secretary);
    const artServer = server === "cn" ? "cn" : undefined;
    const staticUrl = secretarySkinId ? skinTexture(secretary, secretarySkinId, artServer) : operatorHero(secretary, null, null, artServer);

    return (
        <StatCard className="sm:col-span-2" color="var(--primary)">
            <div className="relative h-56 w-full overflow-hidden sm:h-64">
                <img alt={op?.name ?? "Assistant"} src={staticUrl} onError={() => setErrored(true)} className={`absolute inset-0 h-full w-full object-cover object-top transition-opacity duration-500 ${dynActive ? "opacity-0" : "opacity-100"}`} decoding="async" loading="lazy" />
                <DynamicArtOverlay operatorCode={secretary} skinId={secretarySkinId} fit={{ mode: "cover", align: "top" }} onActiveChange={setDynActive} />
                {/* Fade to the card background so the label stays readable. */}
                <div className="pointer-events-none absolute inset-0 bg-linear-to-t from-card via-card/40 to-transparent" />
                <div className="absolute inset-x-0 bottom-0 flex items-end justify-between gap-3 p-4 sm:p-5">
                    <div className="min-w-0">
                        <span className={KICKER_TEXT}>
                            <Sparkles className="mr-1 inline size-3 align-[-2px]" />
                            Assistant
                        </span>
                        <h3 className="truncate font-bold text-2xl text-foreground tracking-tight">{op?.name ?? secretary}</h3>
                        {op && (
                            <span className="mt-0.5 inline-flex items-center gap-1.5 text-muted-foreground text-xs">
                                {"★".repeat(rarityToNumber(op.rarity))} · {formatProfession(op.profession)}
                            </span>
                        )}
                    </div>
                </div>
            </div>
        </StatCard>
    );
}
