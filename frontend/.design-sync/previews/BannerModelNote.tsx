import { BannerModelNote } from "frontend";

// The plain-language reading of a banner's gacha model. The figures are what
// `bannerModel()` derives per rule type: LIMITED 0.35 x 2 = 70%, isolated pity,
// 300-pull exchange; DOUBLE 0.25 x 2 = 50%, standard pity, forced at 150/300;
// LINKAGE 0.50 x 1, handover by pull 120; CLASSIC 0.25 x 2, kernel pity.

export const Limited = () => (
    <div className="w-full max-w-md p-4">
        <BannerModelNote model={{ ruleType: "LIMITED", shareEach: 0.35, shareTotal: 0.7, featuredCount: 2, scope: "isolated", carryOver: false, guarantee: { kind: "none" }, spark: 300, inferred: false }} />
    </div>
);

export const StandardDouble = () => (
    <div className="w-full max-w-md p-4">
        <BannerModelNote model={{ ruleType: "DOUBLE", shareEach: 0.25, shareTotal: 0.5, featuredCount: 2, scope: "standard", carryOver: true, guarantee: { kind: "selection", first: 150, second: 300 }, spark: null, inferred: false }} />
    </div>
);

export const Collab = () => (
    <div className="w-full max-w-md p-4">
        <BannerModelNote model={{ ruleType: "LINKAGE", shareEach: 0.5, shareTotal: 0.5, featuredCount: 1, scope: "isolated", carryOver: false, guarantee: { kind: "linkage", at: 120 }, spark: null, inferred: false }} />
    </div>
);

// A brand-new pool with no published detail: the split falls back to the rule
// type and the amber note says so.
export const KernelInferred = () => (
    <div className="w-full max-w-md p-4">
        <BannerModelNote model={{ ruleType: "CLASSIC", shareEach: 0.25, shareTotal: 0.5, featuredCount: 2, scope: "kernel", carryOver: true, guarantee: { kind: "none" }, spark: null, inferred: true }} />
    </div>
);
