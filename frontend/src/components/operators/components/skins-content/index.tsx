import { useEffect, useState, useMemo } from "react";
import { Separator } from "~/components/ui/separator";
import type { Operator } from "~/types/impl/api/static/operator";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import type { UISkin } from "~/types/impl/frontend/impl/operators";
import { fetchSkins, fetchChibi, convertToUISkins, getFallbackSkin, getFallbackImageUrl } from "./impl/helper";
import { SkinImageViewer } from "./impl/skin-image-viewer";
import { SkinDetailsPanel } from "./impl/skin-details-panel";
import { SkinSelector } from "./impl/skin-selector";
import { FullscreenDialog } from "./impl/fullscreen-dialog";
import { InfoSection } from "./impl/info-section";

function SkinsContent({ operator }: { operator: Operator }) {
    // State for the currently selected skin
    const [selectedSkin, setSelectedSkin] = useState<string>(operator.id ? operator.id : "");
    const [isImageLoading, setIsImageLoading] = useState(true);
    const [imageSrc, setImageSrc] = useState<string>("");
    const [fullscreenOpen, setFullscreenOpen] = useState(false);

    const [skins, setSkins] = useState<UISkin[]>([]);
    const [chibi, setChibi] = useState<ChibisSimplified | null>(null);

    const repoBaseUrl = "https://raw.githubusercontent.com/fexli/ArknightsResource/main/";

    // Define fallbackSkin with useMemo to avoid recalculation on every render
    const fallbackSkin = useMemo(() => getFallbackSkin(operator, repoBaseUrl), [operator, repoBaseUrl]);

    useEffect(() => {
        const loadSkinsAndChibi = async () => {
            const skinsData = await fetchSkins(operator.id ?? "");
            const uiSkins = convertToUISkins(skinsData, operator, repoBaseUrl);
            setSkins(uiSkins);

            const chibiData = (await fetchChibi(operator.id ?? ""));
            setChibi(chibiData[0] ?? null);
        };

        void loadSkinsAndChibi();
    }, [operator, operator.id, repoBaseUrl]);

    // Update image source when skin changes
    useEffect(() => {
        const skin = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;
        setImageSrc(skin.image);
        setIsImageLoading(true);
    }, [fallbackSkin, selectedSkin, skins]);

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
            setImageSrc(skin.fallbackImage ?? getFallbackImageUrl(repoBaseUrl, operator.id ?? ""));
            setIsImageLoading(false);
        }
    };

    const openFullscreen = () => {
        setFullscreenOpen(true);
    };

    const selectedSkinData = skins.find((skin) => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;

    // Function to handle thumbnail image errors
    const handleThumbnailError = (skin: UISkin, e: React.SyntheticEvent<HTMLImageElement>) => {
        e.currentTarget.src = skin.fallbackImage ?? getFallbackImageUrl(repoBaseUrl, operator.id ?? "");
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
                    {/* Left side - Skin image */}
                    <SkinImageViewer operator={operator} selectedSkinData={selectedSkinData} imageSrc={imageSrc} isImageLoading={isImageLoading} handleImageLoad={handleImageLoad} handleImageError={handleImageError} openFullscreen={openFullscreen} />

                    {/* Right side - Skin details */}
                    <SkinDetailsPanel selectedSkinData={selectedSkinData} chibi={chibi} repoBaseUrl={repoBaseUrl} />
                </div>

                {/* Skin selector */}
                <SkinSelector skins={skins} selectedSkin={selectedSkin} setSelectedSkin={setSelectedSkin} setIsImageLoading={setIsImageLoading} handleThumbnailError={handleThumbnailError} />

                {/* Additional information */}
                <InfoSection />
            </div>

            {/* Fullscreen popup */}
            <FullscreenDialog open={fullscreenOpen} onOpenChange={setFullscreenOpen} imageSrc={imageSrc} operator={operator} selectedSkinData={selectedSkinData} />
        </div>
    );
}

export default SkinsContent;
