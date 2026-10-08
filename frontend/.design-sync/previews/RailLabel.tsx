import { CategoryChips, RailLabel, RailRow } from "frontend";
import { BookImageIcon, ClapperboardIcon, MountainIcon } from "lucide-react";

// RailLabel is the quiet label over a group of the art browser's rail: 11px,
// muted, no heading weight. It names the "Gallery" sources, the "Category"
// chips and the "Story" panel; it is never a section heading.

/** Over the gallery sources, as the rail draws it. */
export const OverSources = () => (
    <div className="flex w-64 flex-col gap-0.5">
        <RailLabel>Gallery</RailLabel>
        <RailRow active={false} onClick={() => undefined} name="Archives" icon={<BookImageIcon aria-hidden="true" />} count={324} />
        <RailRow active onClick={() => undefined} name="Story CGs" icon={<ClapperboardIcon aria-hidden="true" />} count={1256} />
        <RailRow active={false} onClick={() => undefined} name="Scenes" icon={<MountainIcon aria-hidden="true" />} count={916} />
    </div>
);

/** Over the category chips, the rail's second group. */
export const OverCategories = () => (
    <div className="w-64">
        <RailLabel>Category</RailLabel>
        <div className="px-1">
            <CategoryChips
                options={[
                    { id: "main", count: 358 },
                    { id: "side", count: 855 },
                    { id: "vignette", count: 43 },
                ]}
                chosen={["side"]}
                onToggle={() => undefined}
            />
        </div>
    </div>
);

/** The label alone. */
export const Alone = () => <RailLabel>Story</RailLabel>;
