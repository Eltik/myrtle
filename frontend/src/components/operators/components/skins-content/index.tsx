import Image from "next/image";
import { useEffect, useState, useMemo } from "react";
import { ScrollArea, ScrollBar } from "~/components/ui/scroll-area";
import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Skin } from "~/types/impl/api/static/skins";
import { Dialog, DialogContent } from "~/components/ui/dialog";
import { Maximize2 } from "lucide-react";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import { ChibiViewer } from "./impl/chibi-viewer";

interface UISkin {
    id: string;
    name: string;
    description: string;
    image: string;
    fallbackImage?: string;
    obtainMethod: string;
    releaseDate: string;
    artists: string[];
    voiceLines: boolean;
    animations: boolean;
    available: boolean;
    isDefault: boolean;
}

function SkinsContent({ operator }: { operator: Operator }) {
    // State for the currently selected skin
    const [selectedSkin, setSelectedSkin] = useState<string>(operator.id ? operator.id : "");
    const [isImageLoading, setIsImageLoading] = useState(true);
    const [imageSrc, setImageSrc] = useState<string>("");
    const [fullscreenOpen, setFullscreenOpen] = useState(false);

    const [skins, setSkins] = useState<UISkin[]>([]);
    const [chibi, setChibi] = useState<ChibisSimplified | null>(null);

    const repoBaseUrl = "https://raw.githubusercontent.com/fexli/ArknightsResource/main/";

    // Define the fallback image URL generator
    const getFallbackImageUrl = (id: string) => `${repoBaseUrl}skin/${id}_1b.png`;

    const fetchChibi = async (id: string) => {
        const response = await fetch("/api/chibis", {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify({
                format: "simplified",
                id,
            }),
        });

        return (await response.json()) as ChibisSimplified[];
    };

    // Define fallbackSkin with useMemo to avoid recalculation on every render
    const fallbackSkin = useMemo(
        () => ({
            id: "default",
            name: "Default",
            description: "Default appearance",
            image: getFallbackImageUrl(operator.id ?? ""),
            fallbackImage: getFallbackImageUrl(operator.id ?? ""),
            obtainMethod: "Default skin",
            releaseDate: "Release",
            artists: ["Original Artist"],
            voiceLines: false,
            animations: false,
            available: true,
            isDefault: true,
        }),
        [operator.id],
    );

    useEffect(() => {
        void fetchSkins(operator.id ?? "").then(async (skins) => {
            const uiSkins: UISkin[] = skins.map((skin) => ({
                id: skin.skinId,
                name: skin.displaySkin.skinName ?? skin.displaySkin.skinGroupName ?? "Default",
                description: skin.displaySkin.description ?? skin.displaySkin.content ?? "Default",
                image: skin.images.skin,
                fallbackImage: getFallbackImageUrl(operator.id ?? ""),
                obtainMethod: skin.displaySkin.obtainApproach ?? "Default",
                releaseDate: skin.displaySkin.getTime ? new Date(skin.displaySkin.getTime * 1000).toLocaleDateString() : "Default",
                artists: skin.displaySkin.drawerList ?? [],
                voiceLines: skin.voiceId !== null,
                animations: (skin.dynIllustId?.length ?? 0) > 0,
                available: skin.isBuySkin,
                isDefault: skin.displaySkin.skinGroupName === "Default Outfit",
            }));
            setSkins(uiSkins);

            const chibi = await fetchChibi(operator.id ?? "");
            setChibi(chibi[0] ?? null);
        });
    }, [operator.id]);

    // Update image source when skin changes
    useEffect(() => {
        const skin = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;
        setImageSrc(skin.image);
        setIsImageLoading(true);
    }, [fallbackSkin, selectedSkin, skins]);

    async function fetchSkins(id: string) {
        const data = (await (
            await fetch("/api/static", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    type: "skins",
                    id,
                }),
            })
        ).json()) as { skins: Skin[] };
        return data.skins;
    }

    const handleImageLoad = () => {
        setIsImageLoading(false);
    };

    const handleImageError = () => {
        // Use fallback image if available
        if (selectedSkinData?.fallbackImage) {
            setImageSrc(selectedSkinData.fallbackImage);
        } else {
            // Fallback to original error handling if needed
            const skin = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;
            setImageSrc(skin.fallbackImage ?? getFallbackImageUrl(operator.id ?? ""));
            setIsImageLoading(false);
        }
    };

    const openFullscreen = () => {
        setFullscreenOpen(true);
    };

    const selectedSkinData = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;

    // Function to handle thumbnail image errors
    const handleThumbnailError = (skin: UISkin, e: React.SyntheticEvent<HTMLImageElement>) => {
        e.currentTarget.src = skin.fallbackImage ?? getFallbackImageUrl(operator.id ?? "");
    };

    return (
        <div className="w-full overflow-x-hidden">
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-lg font-bold sm:text-xl md:text-3xl">{operator.name} Skins</span>
            </div>

            <Separator />
            <div className="mx-auto w-full px-2 py-2 sm:px-3 sm:py-3 md:px-4 md:py-4 lg:max-w-5xl">
                {/* Main skin viewer with improved responsive design */}
                <div className="grid grid-cols-1 gap-3 sm:gap-4 md:gap-5 lg:grid-cols-[1.5fr,1fr] xl:grid-cols-[2fr,1fr]">
                    {/* Left side - Skin image with responsive heights */}
                    <div className="group relative h-[300px] min-h-[250px] w-full overflow-hidden rounded-lg border bg-black/10 backdrop-blur-sm sm:h-[350px] md:h-[400px] lg:h-[500px]">
                        {isImageLoading && (
                            <div className="absolute inset-0 flex items-center justify-center bg-card/20">
                                <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary border-t-transparent sm:h-8 sm:w-8"></div>
                            </div>
                        )}
                        <Image
                            src={imageSrc}
                            alt={`${operator.name} - ${selectedSkinData.name}`}
                            fill
                            style={{
                                objectFit: "contain",
                                opacity: isImageLoading ? 0 : 1,
                            }}
                            className="transition-opacity duration-300"
                            onLoad={handleImageLoad}
                            onError={handleImageError}
                            unoptimized
                        />

                        {/* Fullscreen button with better touch target */}
                        <button onClick={openFullscreen} className="absolute right-2 top-2 translate-x-4 rounded-full bg-black/50 p-1.5 text-white opacity-0 transition-all duration-300 ease-out hover:bg-black/70 group-hover:translate-x-0 group-hover:opacity-100 sm:p-2" aria-label="View fullscreen">
                            <Maximize2 size={16} className="" />
                        </button>

                        {/* Info overlay with improved text truncation and responsive sizes */}
                        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 via-black/50 to-transparent px-3 py-2 text-white shadow-lg backdrop-blur-sm transition-transform duration-300 ease-in-out group-hover:translate-y-full sm:px-4">
                            <h3 className="text-shadow-sm line-clamp-1 text-base font-bold sm:text-lg md:text-xl">{selectedSkinData.name}</h3>
                            <p className="line-clamp-2 text-xs opacity-95 drop-shadow-md sm:text-sm">{selectedSkinData.description}</p>
                        </div>
                    </div>

                    {/* Right side - Skin details with better mobile layout */}
                    <div className="flex w-full flex-col space-y-3 overflow-hidden rounded-lg border bg-card/50 p-3 backdrop-blur-sm sm:space-y-4 sm:p-4">
                        <h3 className="truncate border-b pb-1 text-base font-semibold sm:pb-2 sm:text-lg md:text-xl">{selectedSkinData.name}</h3>

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
                                <ChibiViewer chibi={chibi} skinId={selectedSkinData.id} repoBaseUrl={repoBaseUrl} />
                            </div>
                        )}

                        {!selectedSkinData.isDefault && !selectedSkinData.available && <div className="rounded-md border border-yellow-500/50 bg-yellow-500/10 p-1.5 text-xs sm:p-2 sm:text-sm">This skin is currently unavailable.</div>}
                    </div>
                </div>

                {/* Skin selector with improved responsive sizing */}
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

                {/* Additional information with improved responsive text sizing */}
                <div className="mt-4 rounded-lg border bg-card/30 p-3 backdrop-blur-sm sm:mt-5 sm:p-4 md:mt-6">
                    <h3 className="mb-1 text-base font-semibold sm:mb-2 sm:text-lg">How to obtain skins</h3>
                    <p className="text-xs text-muted-foreground sm:text-sm">Operator skins can be purchased from the in-game Outfit Store using Originium Prime or special outfit vouchers. Some skins are available permanently, while others are limited to special events.</p>
                </div>
            </div>

            {/* Fullscreen popup with improved mobile handling */}
            <Dialog open={fullscreenOpen} onOpenChange={setFullscreenOpen}>
                <DialogContent className="flex h-[85vh] w-[95vw] max-w-6xl items-center justify-center bg-black/90 p-0 sm:h-[90vh]">
                    <div className="relative flex h-full w-full items-center justify-center">
                        <Image src={imageSrc} alt={`${operator.name} - ${selectedSkinData.name} (Fullscreen)`} fill style={{ objectFit: "contain" }} unoptimized />
                    </div>
                </DialogContent>
            </Dialog>
        </div>
    );
}

export default SkinsContent;
