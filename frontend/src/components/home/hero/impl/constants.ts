export const HERO_KEYWORDS = ["operators", "roster", "recruitment", "profile", "progress"] as const;

export const HERO_IMAGES_PRIMARY = ["/images/cg1.png", "/images/cg2.png", "/images/cg3.png", "/images/cg4.png", "/images/cg5.png", "/images/cg6.png", "/images/cg7.png", "/images/cg8.png", "/images/cg9.png"] as const;

export const HERO_IMAGES_SECONDARY = ["/images/cg10.png", "/images/cg11.png", "/images/cg12.png", "/images/cg13.png", "/images/cg14.png", "/images/cg15.png", "/images/cg16.png", "/images/cg17.png"] as const;

export const ANIMATION_VARIANTS = {
    headline: {
        hidden: { opacity: 0, y: 30, scale: 0.95, filter: "blur(10px)" },
        visible: { opacity: 1, y: 0, scale: 1, filter: "blur(0px)" },
    },
    subtitle: {
        hidden: { opacity: 0, y: 20 },
        visible: { opacity: 1, y: 0 },
    },
    buttons: {
        hidden: { opacity: 0, y: 20 },
        visible: { opacity: 1, y: 0 },
    },
} as const;

export const ANIMATION_TRANSITIONS = {
    headline: { duration: 0.6, ease: "easeOut" },
    subtitle: { duration: 0.6, ease: "easeOut", delay: 0.2 },
    buttons: { duration: 0.6, ease: "easeOut", delay: 0.4 },
} as const;
