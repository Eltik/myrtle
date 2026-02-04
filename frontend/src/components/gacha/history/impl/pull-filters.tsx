"use client";

import { Filter, X } from "lucide-react";
import { Button } from "~/components/ui/shadcn/button";
import { Card, CardContent, CardHeader, CardTitle } from "~/components/ui/shadcn/card";
import { Label } from "~/components/ui/shadcn/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "~/components/ui/shadcn/select";
import type { GachaHistoryParams } from "~/types/api";

interface PullFiltersProps {
    filters: GachaHistoryParams;
    onFiltersChange: (filters: GachaHistoryParams) => void;
    onApply: () => void;
    onReset: () => void;
}

export function PullFilters({ filters, onFiltersChange, onApply, onReset }: PullFiltersProps) {
    const hasActiveFilters = filters.rarity !== undefined || filters.gachaType !== undefined || filters.order !== undefined;

    return (
        <Card>
            <CardHeader>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Filter className="h-4 w-4" />
                        <CardTitle className="text-base">Filters</CardTitle>
                    </div>
                    {hasActiveFilters && (
                        <Button onClick={onReset} size="sm" variant="ghost">
                            <X className="mr-1 h-3 w-3" />
                            Clear
                        </Button>
                    )}
                </div>
            </CardHeader>
            <CardContent className="space-y-4">
                {/* Rarity Filter */}
                <div className="space-y-2">
                    <Label htmlFor="rarity-filter">Rarity</Label>
                    <Select
                        onValueChange={(value) => {
                            onFiltersChange({
                                ...filters,
                                rarity: value === "all" ? undefined : Number(value),
                            });
                        }}
                        value={filters.rarity?.toString() ?? "all"}
                    >
                        <SelectTrigger id="rarity-filter">
                            <SelectValue placeholder="All Rarities" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Rarities</SelectItem>
                            <SelectItem value="6">6-Star</SelectItem>
                            <SelectItem value="5">5-Star</SelectItem>
                            <SelectItem value="4">4-Star</SelectItem>
                            <SelectItem value="3">3-Star</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                {/* Banner Type Filter */}
                <div className="space-y-2">
                    <Label htmlFor="banner-filter">Banner Type</Label>
                    <Select
                        onValueChange={(value) => {
                            onFiltersChange({
                                ...filters,
                                gachaType: value === "all" ? undefined : (value as "limited" | "regular" | "special"),
                            });
                        }}
                        value={filters.gachaType ?? "all"}
                    >
                        <SelectTrigger id="banner-filter">
                            <SelectValue placeholder="All Banners" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="all">All Banners</SelectItem>
                            <SelectItem value="limited">Limited Headhunting</SelectItem>
                            <SelectItem value="regular">Regular Headhunting</SelectItem>
                            <SelectItem value="special">Special Headhunting</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                {/* Sort Order */}
                <div className="space-y-2">
                    <Label htmlFor="order-filter">Sort Order</Label>
                    <Select
                        onValueChange={(value) => {
                            onFiltersChange({
                                ...filters,
                                order: value as "asc" | "desc",
                            });
                        }}
                        value={filters.order ?? "desc"}
                    >
                        <SelectTrigger id="order-filter">
                            <SelectValue placeholder="Sort Order" />
                        </SelectTrigger>
                        <SelectContent>
                            <SelectItem value="desc">Newest First</SelectItem>
                            <SelectItem value="asc">Oldest First</SelectItem>
                        </SelectContent>
                    </Select>
                </div>

                <Button className="w-full" onClick={onApply} size="lg">
                    Apply Filters
                </Button>
            </CardContent>
        </Card>
    );
}
