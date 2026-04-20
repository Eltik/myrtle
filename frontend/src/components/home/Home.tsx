import { useCommand } from "#/lib/command-context";
import FeatureStrip from "./impl/FeatureStrip";
import Hero from "./impl/Hero";
import TierLists from "./impl/TierLists";

export default function Home() {
    const { open: openCmd } = useCommand();

    return (
        <main style={{ position: "relative" }}>
            <Hero onOpenCommand={openCmd} />
            <FeatureStrip />
            <TierLists />
        </main>
    );
}
