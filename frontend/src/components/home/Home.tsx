import { useCommand } from "#/lib/command-context";
import styles from "./impl/Digest.module.css";
import HappeningNow from "./impl/HappeningNow";
import Hero from "./impl/Hero";
import LatestUpdates from "./impl/LatestUpdates";
import shared from "./impl/shared.module.css";
import TopDoctors from "./impl/TopDoctors";

/** The section the hero's scroll cue lands on. */
const LIVE_SECTION_ID = "happening-now";

export default function Home() {
    const { open: openCmd } = useCommand();

    return (
        <main className="relative flex flex-1 flex-col">
            <Hero onOpenCommand={openCmd} scrollTargetId={LIVE_SECTION_ID} />
            <HappeningNow id={LIVE_SECTION_ID} />
            <section className={`${shared.container} ${styles.section}`}>
                <TopDoctors />
                <LatestUpdates />
            </section>
        </main>
    );
}
