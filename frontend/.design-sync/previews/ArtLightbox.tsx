import { useEffect } from "react";
import { ArtLightbox } from "frontend";

// ArtLightbox shows one story illustration at full size over whatever opened
// it (the Illustrations panel's grid, the chapter archive's gallery): the
// asset's name as the title, the picture contained in a tinted well, and a
// Close button. It renders nothing while `view` is null.

const noop = () => undefined;

/** Clears the modal's auto-focus ring once the open transition has placed it (60/180/400 ms, see NOTES). */
function useBlurAfterOpen() {
    useEffect(() => {
        const timers = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => timers.forEach(clearTimeout);
    }, []);
}

/** A Shatterpoint CG, open. */
export const CG = () => {
    useBlurAfterOpen();
    return (
        <div style={{ minHeight: 640 }}>
            <ArtLightbox view={{ name: "27_i01", url: "https://api.myrtle.moe/api/assets/textures/avg/imgs/avg_img_27_0/27_i01.png" }} onClose={noop} />
        </div>
    );
};

/** A background plate, open. */
export const Background = () => {
    useBlurAfterOpen();
    return (
        <div style={{ minHeight: 640 }}>
            <ArtLightbox view={{ name: "27_g3_minearea_abandoned", url: "https://api.myrtle.moe/api/assets/textures/avg/bg/avg_bkg_h1_27_0/27_g3_minearea_abandoned.png" }} onClose={noop} />
        </div>
    );
};
