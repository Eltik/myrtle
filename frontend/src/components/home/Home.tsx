import { useState } from "react";
import { useCommand } from "#/lib/command-context";
import Hero from "./impl/Hero";
import FeatureStrip from "./impl/FeatureStrip";
import TierLists from "./impl/TierLists";
import OperatorDrawer from "./impl/OperatorDrawer";
import type { Operator } from "./impl/data";

export default function Home() {
    const { open: openCmd } = useCommand();
    const [selectedOp, setSelectedOp] = useState<Operator | null>(null);

    return (
        <main style={{ position: "relative" }}>
            <Hero onOpenCommand={openCmd} />
            <FeatureStrip />
            <TierLists onSelect={setSelectedOp} />
            {selectedOp && <OperatorDrawer op={selectedOp} onClose={() => setSelectedOp(null)} />}
        </main>
    );
}
