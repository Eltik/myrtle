import Image from "next/image";
import { useState } from "react";
import { ScrollArea, ScrollBar } from "~/components/ui/scroll-area";
import type { Operator } from "~/types/impl/api/static/operator";

function SkinsContent({ operator }: { operator: Operator }) {
    // State for the currently selected skin
    const [selectedSkin, setSelectedSkin] = useState<string>("default");
    const [isImageLoading, setIsImageLoading] = useState(true);

    // Define skin types and variations for this operator
    // In a real implementation, this would come from the API based on SKIN_TABLE data
    const skins = [
        {
            id: "default",
            name: "Default",
            description: "Default appearance",
            image: `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id}_1b.png`,
            obtainMethod: "Default skin",
            releaseDate: "Release",
            artists: ["Original Artist"],
            voiceLines: false,
            animations: false,
            available: true,
        },
        ...(operator.phases.length > 2 ? [
            {
                id: "e2",
                name: "Elite 2",
                description: "Elite 2 Promotion appearance",
                image: `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id}_2b.png`,
                obtainMethod: "Elite 2 Promotion",
                releaseDate: "Release",
                artists: ["Original Artist"],
                voiceLines: false,
                animations: false,
                available: true,
            }
        ] : []),
        // Simulate additional skins - in a real implementation, these would come from the API
        ...(["summer", "winter", "special"].includes(operator.id?.substring(5, 8) ?? "") ? [
            {
                id: "special",
                name: "Special Outfit",
                description: "Limited edition appearance for special events",
                image: `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id}_1+.png`,
                obtainMethod: "Outfit Store or Event Reward",
                releaseDate: "Limited Event",
                artists: ["Specialty Artist"],
                voiceLines: true,
                animations: true,
                available: false,
            }
        ] : [])
    ];

    const handleImageLoad = () => {
        setIsImageLoading(false);
    };

    const handleImageError = (e: React.SyntheticEvent<HTMLImageElement>) => {
        // Handle broken image links
        e.currentTarget.src = `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id}_1b.png`;
        setIsImageLoading(false);
    };

    // Define a fallback skin in case none are found
    const fallbackSkin = {
        id: "default",
        name: "Default",
        description: "Default appearance",
        image: `https://raw.githubusercontent.com/yuanyan3060/ArknightsGameResource/main/skin/${operator.id}_1b.png`,
        obtainMethod: "Default skin",
        releaseDate: "Release",
        artists: ["Original Artist"],
        voiceLines: false,
        animations: false,
        available: true,
    };

    const selectedSkinData = skins.find(skin => skin.id === selectedSkin) ?? skins[0] ?? fallbackSkin;

    return (
        <div className="p-6 flex flex-col space-y-6">
            <div className="text-2xl font-bold">{operator.name} Skins</div>
            
            {/* Main skin viewer */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                {/* Left side - Skin image */}
                <div className="md:col-span-2 relative rounded-lg overflow-hidden h-[500px] bg-black/10 backdrop-blur-sm border">
                    {isImageLoading && (
                        <div className="absolute inset-0 flex items-center justify-center bg-card/20">
                            <div className="animate-spin h-8 w-8 border-2 border-primary border-t-transparent rounded-full"></div>
                        </div>
                    )}
                    <Image 
                        src={selectedSkinData.image}
                        alt={`${operator.name} - ${selectedSkinData.name}`}
                        layout="fill"
                        objectFit="contain"
                        className="transition-opacity duration-300"
                        style={{ opacity: isImageLoading ? 0 : 1 }}
                        onLoad={handleImageLoad}
                        onError={handleImageError}
                    />
                    
                    {/* Info overlay */}
                    <div className="absolute bottom-0 left-0 right-0 p-4 bg-gradient-to-t from-black/70 to-transparent text-white">
                        <h3 className="text-xl font-bold">{selectedSkinData.name}</h3>
                        <p className="opacity-90">{selectedSkinData.description}</p>
                    </div>
                </div>
                
                {/* Right side - Skin details */}
                <div className="flex flex-col space-y-4 p-4 border rounded-lg bg-card/50 backdrop-blur-sm">
                    <h3 className="text-xl font-semibold border-b pb-2">{selectedSkinData.name}</h3>
                    
                    <div className="flex flex-col space-y-2">
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
                            <span className="col-span-2">{selectedSkinData.voiceLines ? "New voice lines" : "Original voice lines"}</span>
                        </div>
                        
                        <div className="grid grid-cols-3 gap-2">
                            <span className="text-muted-foreground">Animations:</span>
                            <span className="col-span-2">{selectedSkinData.animations ? "Custom animations" : "Standard animations"}</span>
                        </div>
                    </div>
                    
                    <div className="mt-auto pt-4">
                        {!selectedSkinData.available && (
                            <div className="border border-yellow-500/50 bg-yellow-500/10 p-2 rounded-md text-sm">
                                This skin is currently unavailable or not yet released.
                            </div>
                        )}
                    </div>
                </div>
            </div>
            
            {/* Skin selector */}
            <div className="pt-4">
                <h3 className="text-lg font-semibold mb-4">Available Skins</h3>
                <ScrollArea className="w-full">
                    <div className="flex space-x-4 pb-4">
                        {skins.map((skin) => (
                            <div 
                                key={skin.id}
                                className={`relative flex-shrink-0 w-32 h-32 border-2 rounded-lg overflow-hidden cursor-pointer transition-all hover:scale-105 ${
                                    selectedSkin === skin.id 
                                        ? "border-primary shadow-lg" 
                                        : "border-transparent hover:border-primary/50"
                                }`}
                                onClick={() => {
                                    setIsImageLoading(true);
                                    setSelectedSkin(skin.id);
                                }}
                            >
                                <Image 
                                    src={skin.image} 
                                    alt={skin.name} 
                                    layout="fill" 
                                    objectFit="cover"
                                    className="hover:opacity-90 transition-opacity"
                                />
                                <div className="absolute bottom-0 left-0 right-0 p-1 bg-black/50 text-white text-xs">
                                    {skin.name}
                                </div>
                                {!skin.available && (
                                    <div className="absolute top-1 right-1 w-4 h-4 bg-yellow-500 rounded-full flex items-center justify-center">
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
            <div className="mt-6 p-4 border rounded-lg bg-card/30 backdrop-blur-sm">
                <h3 className="text-lg font-semibold mb-2">How to obtain skins</h3>
                <p className="text-sm text-muted-foreground">
                    Operator skins can be purchased from the in-game Outfit Store using Originium Prime or special outfit vouchers.
                    Some skins are available permanently, while others are limited to special events or seasonal availability.
                    Elite 2 skins are unlocked by promoting the operator to Elite 2.
                </p>
            </div>
        </div>
    );
}

export default SkinsContent;