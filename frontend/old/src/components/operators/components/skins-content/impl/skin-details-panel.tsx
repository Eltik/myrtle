import type { FormattedChibis } from "~/types/impl/frontend/impl/chibis";
import type { UISkin } from "~/types/impl/frontend/impl/operators";
import { ChibiViewer } from "./chibi-viewer";

interface SkinDetailsPanelProps {
    selectedSkinData: UISkin;
    chibi: FormattedChibis | null;
}

export function SkinDetailsPanel({ selectedSkinData, chibi }: SkinDetailsPanelProps) {
    return (
        <div className="flex w-full flex-col space-y-3 overflow-hidden rounded-lg border bg-card/50 p-3 backdrop-blur-sm sm:space-y-4 sm:p-4">
            <h3 className="truncate border-b pb-1 font-semibold text-base sm:pb-2 sm:text-lg md:text-xl">{selectedSkinData.name}</h3>

            <div className="flex flex-col space-y-1.5 overflow-hidden text-xs sm:space-y-2 sm:text-sm">
                <div className="grid grid-cols-[70px_minmax(0,1fr)] gap-1 sm:grid-cols-[80px_minmax(0,1fr)] sm:gap-2">
                    <span className="text-muted-foreground">Obtain:</span>
                    <span className="line-clamp-2 break-words">{selectedSkinData.obtainMethod}</span>
                </div>

                <div className="grid grid-cols-[70px_minmax(0,1fr)] gap-1 sm:grid-cols-[80px_minmax(0,1fr)] sm:gap-2">
                    <span className="text-muted-foreground">Released:</span>
                    <span className="break-words">{selectedSkinData.releaseDate}</span>
                </div>

                <div className="grid grid-cols-[70px_minmax(0,1fr)] gap-1 sm:grid-cols-[80px_minmax(0,1fr)] sm:gap-2">
                    <span className="text-muted-foreground">Artist:</span>
                    <span className="line-clamp-2 break-words">{selectedSkinData.artists.join(", ")}</span>
                </div>

                <div className="grid grid-cols-[70px_minmax(0,1fr)] gap-1 sm:grid-cols-[80px_minmax(0,1fr)] sm:gap-2">
                    <span className="text-muted-foreground">Voice:</span>
                    <span className="break-words">{selectedSkinData.voiceLines ? "New voice lines" : "Default"}</span>
                </div>

                <div className="grid grid-cols-[70px_minmax(0,1fr)] gap-1 sm:grid-cols-[80px_minmax(0,1fr)] sm:gap-2">
                    <span className="text-muted-foreground">Animations:</span>
                    <span className="break-words">{selectedSkinData.animations ? "Custom" : "Standard"}</span>
                </div>
            </div>

            {/* Chibi viewer with better size constraints */}
            {chibi && (
                <div className="mt-auto w-full overflow-hidden pt-2 sm:pt-3 md:pt-4">
                    <ChibiViewer chibi={chibi} skinId={selectedSkinData.id} />
                </div>
            )}

            {!selectedSkinData.isDefault && !selectedSkinData.available && <div className="rounded-md border border-yellow-500/50 bg-yellow-500/10 p-1.5 text-xs sm:p-2 sm:text-sm">This skin is currently unavailable.</div>}
        </div>
    );
}
