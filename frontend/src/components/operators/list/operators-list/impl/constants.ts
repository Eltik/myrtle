// Quick transition for toggle controls (synchronized timing)
export const TOGGLE_TRANSITION = {
    duration: 0.2,
    ease: [0.4, 0, 0.2, 1] as const,
};

// Fast transition for grid/list container switches
export const CONTAINER_TRANSITION = {
    duration: 0.2,
    ease: [0.4, 0, 0.2, 1] as const,
};
