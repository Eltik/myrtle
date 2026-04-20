import { useState } from "react";
import { useCommand } from "#/lib/command-context";
import type { Operator } from "./impl/data";
import FeatureStrip from "./impl/FeatureStrip";
import Hero from "./impl/Hero";
import OperatorDrawer from "./impl/OperatorDrawer";
import TierLists from "./impl/TierLists";

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
