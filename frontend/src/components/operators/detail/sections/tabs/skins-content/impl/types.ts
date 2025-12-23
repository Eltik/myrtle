export interface UISkin {
    id: string;
    name: string;
    image: string;
    thumbnail: string;
    displaySkin?: {
        skinName?: string;
        modelName?: string;
        drawerList?: string[];
        designerList?: string[];
        obtainApproach?: string;
    };
    isDefault: boolean;
}
