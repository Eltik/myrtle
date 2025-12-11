import { Badge } from "~/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/card";
import type { Item } from "~/types/impl/api/static/material";
import type { MaterialCost } from "~/types/impl/frontend/impl/operators";
import { getGridColumns } from "./helper";
import { MaterialItem } from "./material-item";

interface ElitePromotionTabProps {
    elitePromotionCosts: {
        elite: string;
        materials: MaterialCost[];
    }[];
    materials: Item[];
}

export const ElitePromotionTab = ({ elitePromotionCosts, materials }: ElitePromotionTabProps) => {
    return (
        <div className="space-y-4">
            <h2 className="font-bold text-xl">Elite Promotion Costs</h2>

            {elitePromotionCosts.length === 0 ? (
                <p className="text-muted-foreground">No promotion costs available for this operator.</p>
            ) : (
                <div className="grid gap-4 md:grid-cols-2">
                    {elitePromotionCosts.map((promotion) => (
                        <Card key={promotion.elite}>
                            <CardHeader className="pb-2">
                                <CardTitle className="text-lg">
                                    <Badge className="mr-2" variant="outline">
                                        {promotion.elite}
                                    </Badge>
                                    Promotion
                                </CardTitle>
                            </CardHeader>
                            <CardContent>
                                {promotion.materials && promotion.materials.length > 0 ? (
                                    <div className={`grid gap-4 ${getGridColumns(promotion.materials.length)}`}>
                                        {promotion.materials.map((material) => (
                                            <MaterialItem key={material.material.itemId} material={material} materials={materials} />
                                        ))}
                                    </div>
                                ) : (
                                    <p className="text-muted-foreground">No materials required.</p>
                                )}
                            </CardContent>
                        </Card>
                    ))}
                </div>
            )}
        </div>
    );
};
