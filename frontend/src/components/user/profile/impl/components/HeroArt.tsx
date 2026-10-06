import { type ReactNode, useState } from "react";
import type { ProfileBackground } from "#/types/generated/ProfileBackground";
import { artStyle, backgroundSources } from "../background";
import styles from "./HeroArt.module.css";

interface IHeroArtProps {
    background: ProfileBackground;
    /** The header's plain look: drawn under the art, so it shows through a cut-out figure, while the art loads, and alone when no source loads. */
    fallback: ReactNode;
}

/**
 * The owner's art behind the header, under a scrim mixed from the card colour. Absolutely
 * positioned inside the header, so neither loading nor failing moves anything. Each source
 * is tried in turn (see `backgroundSources`); when none loads the header keeps its plain
 * look. Mount it with a `key` per picture, so a new pick starts over at the first source.
 *
 * The background editor renders this same component (through `Hero`) in its preview and
 * reads the drawn `img` by its `.art` class, so the art it crops is this art.
 */
export function HeroArt({ background, fallback }: IHeroArtProps) {
    const sources = backgroundSources(background);
    const [index, setIndex] = useState(0);
    const [loaded, setLoaded] = useState(false);
    const src = sources[index];
    if (src === undefined) return fallback;
    return (
        <>
            {fallback}
            <div className={styles.frame} aria-hidden="true">
                <img
                    key={src}
                    src={src}
                    alt=""
                    decoding="async"
                    fetchPriority="low"
                    draggable={false}
                    data-loaded={loaded}
                    // An image the browser finished before hydration fires no load event React sees.
                    ref={(img) => {
                        if (!img?.complete) return;
                        if (img.naturalWidth > 0) setLoaded(true);
                        else setIndex((i) => (sources[i] === img.getAttribute("src") ? i + 1 : i));
                    }}
                    className={styles.art}
                    style={artStyle(background)}
                    onLoad={(e) => {
                        if (e.currentTarget.naturalWidth === 0) setIndex((i) => i + 1);
                        else setLoaded(true);
                    }}
                    onError={() => setIndex((i) => i + 1)}
                />
            </div>
            <div className={styles.scrim} aria-hidden="true" />
        </>
    );
}
