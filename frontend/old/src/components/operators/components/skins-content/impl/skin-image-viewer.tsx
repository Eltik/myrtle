import { Maximize2 } from "lucide-react";
import Image from "next/image";
import { getCDNURL } from "~/lib/cdn";
import type { UISkin } from "~/types/impl/frontend/impl/operators";

interface SkinImageViewerProps {
    operator: {
        name: string;
    };
    selectedSkinData: UISkin;
    imageSrc: string;
    isImageLoading: boolean;
    handleImageLoad: () => void;
    handleImageError: () => void;
    openFullscreen: () => void;
}

export function SkinImageViewer({ operator, selectedSkinData, imageSrc, isImageLoading, handleImageLoad, handleImageError, openFullscreen }: SkinImageViewerProps) {
    return (
        <div className="group relative h-[300px] min-h-[250px] w-full overflow-hidden rounded-lg border bg-black/10 backdrop-blur-sm sm:h-[350px] md:h-[400px] lg:h-[500px]">
            {isImageLoading && (
                <div className="absolute inset-0 flex items-center justify-center bg-card/20">
                    <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent sm:h-8 sm:w-8"></div>
                </div>
            )}
            <Image
                alt={`${operator.name} - ${selectedSkinData.name}`}
                className="transition-opacity duration-300"
                fill
                onError={handleImageError}
                onLoad={handleImageLoad}
                src={getCDNURL(imageSrc, true)}
                style={{
                    objectFit: "contain",
                    opacity: isImageLoading ? 0 : 1,
                }}
                unoptimized
            />

            {/* Fullscreen button with better touch target */}
            <button aria-label="View fullscreen" className="absolute top-2 right-2 translate-x-4 rounded-full bg-black/50 p-1.5 text-white opacity-0 transition-all duration-300 ease-out hover:bg-black/70 group-hover:translate-x-0 group-hover:opacity-100 sm:p-2" onClick={openFullscreen}>
                <Maximize2 className="" size={16} />
            </button>

            {/* Info overlay with improved text truncation and responsive sizes */}
            <div className="absolute right-0 bottom-0 left-0 bg-gradient-to-t from-black/80 via-black/50 to-transparent px-3 py-2 text-white shadow-lg backdrop-blur-sm transition-transform duration-300 ease-in-out group-hover:translate-y-full sm:px-4">
                <h3 className="line-clamp-1 font-bold text-base text-shadow-sm sm:text-lg md:text-xl">{selectedSkinData.name}</h3>
                <p className="line-clamp-2 text-xs opacity-95 drop-shadow-md sm:text-sm">{selectedSkinData.description}</p>
            </div>
        </div>
    );
}
