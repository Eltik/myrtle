import { useState, useEffect } from "react";
import { Card, CardContent } from "~/components/ui/card";
import { Input } from "~/components/ui/input";
import { Button } from "~/components/ui/button";
import { Skeleton } from "~/components/ui/skeleton";
import type { ChibisSimplified } from "~/types/impl/api/impl/chibis";
import { PixiRenderer } from "./impl/PixiRenderer";
import Image from "next/image";

export function ChibiViewer() {
    const [chibis, setChibis] = useState<ChibisSimplified[]>([]);
    const [loading, setLoading] = useState(true);
    const [searchTerm, setSearchTerm] = useState("");
    const [selectedOperator, setSelectedOperator] = useState<ChibisSimplified | null>(null);
    const [selectedSkin, setSelectedSkin] = useState<string | null>(null);
    const [renderMode, setRenderMode] = useState<"static" | "animated">("animated");

    useEffect(() => {
        const fetchChibis = async () => {
            try {
                setLoading(true);
                const response = await fetch("/api/chibis", {
                    method: "POST",
                    headers: {
                        "Content-Type": "application/json",
                    },
                    body: JSON.stringify({
                        format: "simplified",
                    }),
                });

                const data = (await response.json()) as ChibisSimplified[];
                setChibis(data);

                // Select the first operator by default
                if (data.length > 0) {
                    const firstOperator = data[0];
                    if (firstOperator) {
                        setSelectedOperator(firstOperator);
                        if (firstOperator.skins && firstOperator.skins.length > 0 && firstOperator.skins[0]?.path) {
                            setSelectedSkin(firstOperator.skins[0].path);
                        }
                    }
                }
            } catch (error) {
                console.error("Error fetching chibis:", error);
            } finally {
                setLoading(false);
            }
        };

        void fetchChibis();
    }, []);

    const filteredChibis = chibis.filter((chibi) => chibi.name.toLowerCase().includes(searchTerm.toLowerCase()) || chibi.operatorCode.toLowerCase().includes(searchTerm.toLowerCase()));

    const handleOperatorSelect = (operator: ChibisSimplified) => {
        setSelectedOperator(operator);
        if (operator.skins && operator.skins.length > 0 && operator.skins[0]?.path) {
            setSelectedSkin(operator.skins[0].path);
        } else {
            setSelectedSkin(null);
        }
    };

    // GitHub raw content URL base for the repository
    const repoBaseUrl = "https://raw.githubusercontent.com/fexli/ArknightsResource/main/";

    // Function to get asset URL from the path
    const getAssetUrl = (path: string) => {
        // Remove the initial "./" if present
        const normalizedPath = path.startsWith("./") ? path.substring(2) : path;
        return `${repoBaseUrl}${normalizedPath}`;
    };

    return (
        <div className="container mx-auto py-8">
            <div className="flex flex-col gap-8 md:flex-row">
                {/* Operator List */}
                <div className="w-full space-y-4 md:w-1/3">
                    <div className="sticky top-4">
                        <Input placeholder="Search operators..." value={searchTerm} onChange={(e) => setSearchTerm(e.target.value)} className="mb-4" />

                        <div className="h-[70vh] overflow-y-auto rounded-lg border bg-card">
                            {loading ? (
                                Array.from({ length: 10 }).map((_, i) => (
                                    <div key={i} className="border-b p-3">
                                        <Skeleton className="h-12 w-full" />
                                    </div>
                                ))
                            ) : filteredChibis.length > 0 ? (
                                <div className="divide-y">
                                    {filteredChibis.map((chibi) => (
                                        <Button key={chibi.operatorCode} variant={selectedOperator?.operatorCode === chibi.operatorCode ? "default" : "ghost"} className="h-auto w-full justify-start p-3" onClick={() => handleOperatorSelect(chibi)}>
                                            <div className="text-left">
                                                <div className="font-medium">{chibi.name}</div>
                                                <div className="text-sm text-muted-foreground">{chibi.operatorCode}</div>
                                            </div>
                                        </Button>
                                    ))}
                                </div>
                            ) : (
                                <div className="p-8 text-center text-muted-foreground">No operators found</div>
                            )}
                        </div>
                    </div>
                </div>

                {/* Chibi Viewer */}
                <div className="w-full md:w-2/3">
                    <Card>
                        <CardContent className="p-6">
                            {selectedOperator ? (
                                <div className="space-y-6">
                                    <div className="flex flex-col items-start justify-between gap-4 sm:flex-row sm:items-center">
                                        <div>
                                            <h2 className="text-2xl font-bold">{selectedOperator.name}</h2>
                                            <p className="text-muted-foreground">{selectedOperator.operatorCode}</p>
                                        </div>

                                        <div className="flex flex-wrap gap-2">
                                            {/* Skin selector */}
                                            {selectedOperator.skins && selectedOperator.skins.length > 0 && (
                                                <div className="flex flex-wrap gap-2">
                                                    {selectedOperator.skins.map((skin, index) => (
                                                        <Button key={index} variant={selectedSkin === skin.path ? "default" : "outline"} onClick={() => setSelectedSkin(skin.path)} size="sm">
                                                            Skin {index + 1}
                                                        </Button>
                                                    ))}
                                                </div>
                                            )}

                                            {/* Render mode toggle */}
                                            {selectedSkin && selectedOperator.skins?.find((skin) => skin.path === selectedSkin)?.hasSpineData && (
                                                <div className="ml-2 flex gap-2">
                                                    <Button variant={renderMode === "static" ? "default" : "outline"} onClick={() => setRenderMode("static")} size="sm">
                                                        Static
                                                    </Button>
                                                    <Button variant={renderMode === "animated" ? "default" : "outline"} onClick={() => setRenderMode("animated")} size="sm">
                                                        Animated
                                                    </Button>
                                                </div>
                                            )}
                                        </div>
                                    </div>

                                    {selectedSkin ? (
                                        <div className="flex justify-center rounded-lg bg-black/5 p-8">
                                            {selectedOperator.skins?.find((skin) => skin.path === selectedSkin)?.hasSpineData ? (
                                                renderMode === "static" ? (
                                                    <div className="text-center">
                                                        {(() => {
                                                            const selectedSkinData = selectedOperator.skins.find((skin) => skin.path === selectedSkin);
                                                            const imageUrl = getAssetUrl(selectedSkinData?.spineFiles?.png ?? "");
                                                            return (
                                                                <>
                                                                    <Image
                                                                        src={imageUrl}
                                                                        alt={`${selectedOperator.name} chibi`}
                                                                        className="mx-auto object-contain"
                                                                        width={400}
                                                                        height={400}
                                                                        onError={(e) => {
                                                                            // Next Image doesn't support onError the same way, use this pattern instead
                                                                            const target = e.target as HTMLImageElement;
                                                                            target.src = "/placeholder-chibi.png";
                                                                        }}
                                                                    />
                                                                    <p className="mt-2 text-muted-foreground">Chibi sprite sheet</p>
                                                                </>
                                                            );
                                                        })()}
                                                    </div>
                                                ) : (
                                                    (() => {
                                                        const selectedSkinData = selectedOperator.skins.find((skin) => skin.path === selectedSkin);
                                                        return (
                                                            <PixiRenderer 
                                                                atlasUrl={getAssetUrl(selectedSkinData?.spineFiles?.atlas ?? "")} 
                                                                skelUrl={getAssetUrl(selectedSkinData?.spineFiles?.skel ?? "")} 
                                                                imageUrl={getAssetUrl(selectedSkinData?.spineFiles?.png ?? "")} 
                                                                operatorName={selectedOperator.name} 
                                                            />
                                                        );
                                                    })()
                                                )
                                            ) : (
                                                <div className="p-8 text-center text-muted-foreground">No spine data available for this skin</div>
                                            )}
                                        </div>
                                    ) : (
                                        <div className="p-8 text-center text-muted-foreground">No skins available</div>
                                    )}

                                    {selectedSkin && selectedOperator.skins?.find((skin) => skin.path === selectedSkin)?.hasSpineData && (
                                        <div className="space-y-2 text-sm">
                                            <div className="font-medium">Spine Data Files:</div>
                                            <div className="grid grid-cols-1 gap-4 md:grid-cols-3">
                                                <div className="rounded-lg border p-3">
                                                    <div className="font-medium">Atlas:</div>
                                                    <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.path === selectedSkin)?.spineFiles?.atlas ?? "N/A"}</div>
                                                </div>
                                                <div className="rounded-lg border p-3">
                                                    <div className="font-medium">Skeleton:</div>
                                                    <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.path === selectedSkin)?.spineFiles?.skel ?? "N/A"}</div>
                                                </div>
                                                <div className="rounded-lg border p-3">
                                                    <div className="font-medium">Image:</div>
                                                    <div className="truncate text-xs text-muted-foreground">{selectedOperator.skins.find((skin) => skin.path === selectedSkin)?.spineFiles?.png ?? "N/A"}</div>
                                                </div>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            ) : (
                                <div className="p-8 text-center text-muted-foreground">{loading ? "Loading..." : "Select an operator to view their chibi"}</div>
                            )}
                        </CardContent>
                    </Card>
                </div>
            </div>
        </div>
    );
}
