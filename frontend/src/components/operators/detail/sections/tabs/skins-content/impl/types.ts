export interface UISkin {
    id: string;
    name: string;
    image: string;
    thumbnail: string;
    displaySkin?: {
        colorList?: string[];
        content?: string;
        description?: string;
        dialog?: string;
        getTime?: number;
        onPeriod?: number;
        onYear?: number;
        skinGroupId?: string;
        skinGroupName?: string;
        skinGroupSortIndex?: string;
        sortId?: number;
        usage?: string;
        titleList?: string[];
        skinName?: string;
        modelName?: string;
        drawerList?: string[];
        designerList?: string[];
        obtainApproach?: string;
    };
    isDefault: boolean;
}
