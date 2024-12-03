import dpsSpecialTags from "./dps-special-tags.json";

export function checkSpecs(tag: string, spec: string) {
    if (tag in dpsSpecialTags && spec in dpsSpecialTags[tag as keyof typeof dpsSpecialTags]) return dpsSpecialTags[tag as keyof typeof dpsSpecialTags][spec as keyof (typeof dpsSpecialTags)[keyof typeof dpsSpecialTags]];
    else return false;
}
