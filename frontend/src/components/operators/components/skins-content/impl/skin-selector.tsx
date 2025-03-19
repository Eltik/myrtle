import Image from "next/image";
import { ScrollArea, ScrollBar } from "~/components/ui/scroll-area";
import type { UISkin } from "~/types/impl/frontend/impl/operators";

interface SkinSelectorProps {
    skins: UISkin[];
    selectedSkin: string;
    setSelectedSkin: (id: string) => void;
    setIsImageLoading: (loading: boolean) => void;
    handleThumbnailError: (skin: UISkin, e: React.SyntheticEvent<HTMLImageElement>) => void;
}

export function SkinSelector({ skins, selectedSkin, setSelectedSkin, setIsImageLoading, handleThumbnailError }: SkinSelectorProps) {
    return (
        <div className="mt-4 w-full overflow-hidden sm:mt-5 md:mt-6">
            <h3 className="mb-2 text-base font-semibold sm:mb-3 sm:text-lg md:mb-4">Available Skins</h3>
            <div className="relative w-full">
                <ScrollArea className="w-full">
                    <div className="flex space-x-2 pb-3 sm:space-x-3 md:space-x-4 md:pb-4" style={{ width: "max-content" }}>
                        {skins.map((skin) => (
                            <div
                                key={skin.id}
                                className={`xs:h-24 xs:w-24 relative h-20 w-20 flex-shrink-0 cursor-pointer overflow-hidden rounded-lg border-2 transition-all hover:scale-105 sm:h-28 sm:w-28 md:h-32 md:w-32 ${selectedSkin === skin.id ? "border-primary shadow-lg" : "border-transparent hover:border-primary/50"}`}
                                onClick={() => {
                                    setIsImageLoading(true);
                                    setSelectedSkin(skin.id);
                                }}
                            >
                                <Image src={skin.image} alt={skin.name} fill style={{ objectFit: "cover" }} className="transition-opacity hover:opacity-90" onError={(e) => handleThumbnailError(skin, e)} unoptimized />
                                <div className="text-2xs absolute bottom-0 left-0 right-0 line-clamp-1 bg-black/50 p-0.5 text-white sm:p-1 sm:text-xs">{skin.name}</div>
                                {!skin.available && !skin.isDefault && (
                                    <div className="absolute right-1 top-1 flex h-3 w-3 items-center justify-center rounded-full bg-yellow-500 sm:h-4 sm:w-4">
                                        <span className="text-2xs sm:text-xs">!</span>
                                    </div>
                                )}
                            </div>
                        ))}
                    </div>
                    <ScrollBar orientation="horizontal" />
                </ScrollArea>
            </div>
        </div>
    );
}
