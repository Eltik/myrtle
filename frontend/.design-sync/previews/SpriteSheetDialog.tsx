import { SpriteSheetDialog } from "frontend";
import { type ReactNode, useEffect } from "react";

// One story character's expression sheet, over the Characters grid: the title
// row (primary name + folder id) and a body that loads the sprite's detail
// through a query. The detail endpoint is stubbed in the design bundle, so the
// body renders its real load-failure line under the real header and footer.

const noop = () => {};

/** Full-viewport stage; the close button takes initial focus, blurred after the open transition. */
const Stage = ({ children }: { children: ReactNode }) => {
    useEffect(() => {
        const ids = [60, 180, 400].map((ms) => setTimeout(() => (document.activeElement as HTMLElement | null)?.blur(), ms));
        return () => ids.forEach(clearTimeout);
    }, []);
    return <div className="min-h-dvh">{children}</div>;
};

/** An operator's sprite folder (Kal'tsit, `avg_003_kalts_1`). */
export const Operator = () => (
    <Stage>
        <SpriteSheetDialog base="avg_003_kalts_1" name="Kal'tsit" onClose={noop} />
    </Stage>
);

/** An NPC folder named by its most-used script name. */
export const Npc = () => (
    <Stage>
        <SpriteSheetDialog base="avg_npc_053" name="Sarkaz Mercenary" onClose={noop} />
    </Stage>
);
