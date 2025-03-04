import Image from "next/image";
import { useEffect, useState } from "react";
import { ScrollArea, ScrollBar } from "~/components/ui/scroll-area";
import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";
import type { Skin } from "~/types/impl/api/static/skins";

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
    const [selectedSkin, setSelectedSkin] = useState<string>("default");
    const [isImageLoading, setIsImageLoading] = useState(true);
    const [imageSrc, setImageSrc] = useState<string>("");

    const [skins, setSkins] = useState<UISkin[]>([]);

    // Define the fallback image URL generator
    const getFallbackImageUrl = (id: string) => `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${id}_1b.png`;

    useEffect(() => {
        void fetchSkins(operator.id ?? "").then((skins) => {
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
        });
    }, [operator.id]);

    // Update image source when skin changes
    useEffect(() => {
        const skin = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;
        setImageSrc(skin.image);
        setIsImageLoading(true);
    }, [selectedSkin, skins]);

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
        // Instead of trying to modify the src directly, update the imageSrc state
        const skin = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;
        setImageSrc(skin.fallbackImage ?? getFallbackImageUrl(operator.id ?? ""));
        setIsImageLoading(false);
    };

    // Define a fallback skin in case none are found
    const fallbackSkin = {
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
    };

    const selectedSkinData = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;

    // Function to handle thumbnail image errors
    const handleThumbnailError = (skin: UISkin, e: React.SyntheticEvent<HTMLImageElement>) => {
        e.currentTarget.src = skin.fallbackImage ?? getFallbackImageUrl(operator.id ?? "");
    };

    return (
        <>
            <div className="p-2 px-4 backdrop-blur-2xl">
                <span className="text-xl font-bold md:text-3xl">{operator.name} Skins</span>
            </div>
            <Separator />
            <div className="flex flex-col space-y-6 p-6">
                {/* Main skin viewer */}
                <div className="grid grid-cols-1 gap-6 md:grid-cols-3">
                    {/* Left side - Skin image */}
                    <div className="relative h-[500px] overflow-hidden rounded-lg border bg-black/10 backdrop-blur-sm md:col-span-2">
                        {isImageLoading && (
                            <div className="absolute inset-0 flex items-center justify-center bg-card/20">
                                <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent"></div>
                            </div>
                        )}
                        <Image src={imageSrc} alt={`${operator.name} - ${selectedSkinData.name}`} layout="fill" objectFit="contain" className="transition-opacity duration-300" style={{ opacity: isImageLoading ? 0 : 1 }} onLoad={handleImageLoad} onError={handleImageError} unoptimized />

                        {/* Info overlay */}
                        <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/70 to-transparent p-4 text-white">
                            <h3 className="text-xl font-bold">{selectedSkinData.name}</h3>
                            <p className="opacity-90">{selectedSkinData.description}</p>
                        </div>
                    </div>

                    {/* Right side - Skin details */}
                    <div className="flex flex-col space-y-4 rounded-lg border bg-card/50 p-4 backdrop-blur-sm">
                        <h3 className="border-b pb-2 text-xl font-semibold">{selectedSkinData.name}</h3>

                        <div className="flex flex-col space-y-2 text-sm">
                            <div className="grid grid-cols-3 gap-2">
                                <span className="text-muted-foreground">Obtain:</span>
                                <span className="col-span-2">{selectedSkinData.obtainMethod}</span>
                            </div>

                            <div className="grid grid-cols-3 gap-2">
                                <span className="text-muted-foreground">Released:</span>
                                <span className="col-span-2">{selectedSkinData.releaseDate}</span>
                            </div>

                            <div className="grid grid-cols-3 gap-2">
                                <span className="text-muted-foreground">Artist:</span>
                                <span className="col-span-2">{selectedSkinData.artists.join(", ")}</span>
                            </div>

                            <div className="grid grid-cols-3 gap-2">
                                <span className="text-muted-foreground">Voice Lines:</span>
                                <span className="col-span-2">{selectedSkinData.voiceLines ? "New voice lines" : "Default"}</span>
                            </div>

                            <div className="grid grid-cols-3 gap-2">
                                <span className="text-muted-foreground">Animations:</span>
                                <span className="col-span-2">{selectedSkinData.animations ? "Custom animations" : "Standard animations"}</span>
                            </div>
                        </div>

                        <div className="mt-auto pt-4">{!selectedSkinData.isDefault && !selectedSkinData.available && <div className="rounded-md border border-yellow-500/50 bg-yellow-500/10 p-2 text-sm">This skin is currently unavailable or not yet released.</div>}</div>
                    </div>
                </div>

                {/* Skin selector */}
                <div className="pt-4">
                    <h3 className="mb-4 text-lg font-semibold">Available Skins</h3>
                    <ScrollArea className="w-full">
                        <div className="flex space-x-4 pb-4">
                            {skins.map((skin) => (
                                <div
                                    key={skin.id}
                                    className={`relative h-32 w-32 flex-shrink-0 cursor-pointer overflow-hidden rounded-lg border-2 transition-all hover:scale-105 ${selectedSkin === skin.id ? "border-primary shadow-lg" : "border-transparent hover:border-primary/50"}`}
                                    onClick={() => {
                                        setIsImageLoading(true);
                                        setSelectedSkin(skin.id);
                                    }}
                                >
                                    <Image src={skin.image} alt={skin.name} layout="fill" objectFit="cover" className="transition-opacity hover:opacity-90" onError={(e) => handleThumbnailError(skin, e)} unoptimized />
                                    <div className="absolute bottom-0 left-0 right-0 line-clamp-1 bg-black/50 p-1 text-xs text-white">{skin.name}</div>
                                    {!skin.available && !skin.isDefault && (
                                        <div className="absolute right-1 top-1 flex h-4 w-4 items-center justify-center rounded-full bg-yellow-500">
                                            <span className="text-xs">!</span>
                                        </div>
                                    )}
                                </div>
                            ))}
                        </div>
                        <ScrollBar orientation="horizontal" />
                    </ScrollArea>
                </div>

                {/* Additional information about how to obtain skins */}
                <div className="mt-6 rounded-lg border bg-card/30 p-4 backdrop-blur-sm">
                    <h3 className="mb-2 text-lg font-semibold">How to obtain skins</h3>
                    <p className="text-sm text-muted-foreground">Operator skins can be purchased from the in-game Outfit Store using Originium Prime or special outfit vouchers. Some skins are available permanently, while others are limited to special events or seasonal availability. Elite 2 skins are unlocked by promoting the operator to Elite 2.</p>
                </div>
            </div>
        </>
    );
}

export default SkinsContent;
