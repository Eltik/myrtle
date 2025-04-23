import { type AnimationType, type AnimationData, type FormattedChibis, type FormattedSkin, type SkinData } from "./types";

/**
 * Get the full URL for an asset
 */
function getAssetURL(assetPath: string): string {
    // The backend now provides full paths, so we use them directly
    return assetPath;
}

/**
 * Get the skin data for the selected operator and skin
 * @param operators Array of operators with their skins
 * @param selectedOperator Selected operator code
 * @param selectedSkin Selected skin name or path
 * @param selectedView Selected view type
 * @returns Skin data object or null if not found
 */
export function getSkinData(
    operators: FormattedChibis[],
    selectedOperator: string | null,
    selectedSkin: string | null,
    selectedView: AnimationType
): SkinData | null {
    console.log("Input params:", { operators, selectedOperator, selectedSkin, selectedView });
    
    if (!operators || operators.length === 0) {
        console.warn("No operators data provided");
        return null;
    }
    
    // Find the selected operator or use the first one
    const operator = selectedOperator
        ? operators.find((op) => op.operatorCode === selectedOperator)
        : operators[0];
        
    if (!operator) {
        console.warn(`Operator ${selectedOperator} not found`);
        return null;
    }
    
    // Find the selected skin
    let skin: FormattedSkin | undefined;
    
    if (selectedSkin) {
        // Check if selectedSkin is a name or a path
        skin = operator.skins.find((s: FormattedSkin) => s.name === selectedSkin);
        
        // If not found by name, try to find by path (dorm, front, or back path)
        if (!skin) {
            skin = operator.skins.find((s: FormattedSkin) => 
                s.dorm.path === selectedSkin || 
                s.front.path === selectedSkin || 
                s.back.path === selectedSkin
            );
        }
    }
    
    // If still not found, use default skin or first available
    if (!skin) {
        skin = operator.skins.find((s: FormattedSkin) => s.name === "Default") ?? operator.skins[0];
    }
        
    if (!skin) {
        console.warn(`Skin ${selectedSkin} not found for operator ${operator.operatorCode}`);
        return null;
    }
    
    // Get the view data with fallbacks
    const viewData = getViewData(skin, selectedView);
    if (!viewData) {
        console.warn(`No valid view data found for ${selectedView} view`);
        return null;
    }
    
    console.log("Selected skin data:", { operator, skin, viewData });
    
    return buildSkinData(operator, skin, viewData, selectedView);
}

/**
 * Helper function to get view data with fallback options
 */
function getViewData(skin: FormattedSkin, view: AnimationType): AnimationData | null {
    // Try the selected view first
    if (skin[view]?.atlas && skin[view]?.png && skin[view]?.skel) {
        return skin[view];
    }
    
    // Fallback order: dorm -> front -> back
    const fallbacks: AnimationType[] = ["dorm", "front", "back"];
    
    for (const fallback of fallbacks) {
        if (fallback !== view && skin[fallback]?.atlas && 
            skin[fallback]?.png && skin[fallback]?.skel) {
            console.log(`Falling back to ${fallback} view`);
            return skin[fallback];
        }
    }
    
    return null;
}

/**
 * Build the skin data object with correctly formatted URLs
 */
function buildSkinData(
    operator: FormattedChibis,
    skin: FormattedSkin,
    viewData: AnimationData,
    view: AnimationType
): SkinData {
    return {
        atlas: getAssetURL(viewData.atlas),
        png: getAssetURL(viewData.png),
        skel: getAssetURL(viewData.skel),
        name: skin.name,
        code: operator.operatorCode,
        path: viewData.path,
        view: view
    };
}
