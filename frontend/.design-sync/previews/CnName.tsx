import { AutoTag, CnName, Tag } from "frontend";

// How the release planner names a CN row. An official EN name wins, with the CN
// original underneath; failing that, an auto-translated name with its source tag;
// failing that, the CN text itself, marked zh-CN so the browser can translate it.

// Official EN name: EN title on top, CN original as the muted second line.
export const OfficialEnglish = () => (
    <div className="w-full max-w-md p-4">
        <CnName cn="太阳甩在身后·复刻" en="Adventure That Cannot Wait for the Sun - Rerun" primaryClassName="font-sans font-semibold text-[14px] text-foreground" />
    </div>
);

// No EN name yet: the planner's auto-translation, tagged with where it came from.
export const AutoTranslated = () => (
    <div className="w-full max-w-md p-4">
        <CnName cn="人偶的歌谣" en={null} auto={{ text: "Cantilena Puppae", source: "memory" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground" />
    </div>
);

// Nothing to translate from: the CN name stands alone.
export const ChineseOnly = () => (
    <div className="w-full max-w-md p-4">
        <CnName cn="焰烬曙明完结庆祝" en={null} auto={null} primaryClassName="font-sans font-semibold text-[14px] text-foreground" />
    </div>
);

// With trailing chips, as the schedule detail renders a row title.
export const WithTags = () => (
    <div className="w-full max-w-md p-4">
        <CnName cn="相变临界" en={null} auto={{ text: "Critical Phase Transition", source: "appellation" }} primaryClassName="font-sans font-semibold text-[14px] text-foreground">
            <Tag className="text-sky-600 dark:text-sky-400">Event</Tag>
            <Tag>Main story</Tag>
        </CnName>
    </div>
);

// Compact: one truncated line, the original and source moved into the title.
export const Compact = () => (
    <div className="flex w-64 flex-col gap-1.5 p-4 font-sans text-[12.5px] text-foreground">
        <CnName compact cn="太阳甩在身后·复刻" en="Adventure That Cannot Wait for the Sun - Rerun" />
        <CnName compact cn="人偶的歌谣" auto={{ text: "Cantilena Puppae", source: "memory" }}>
            <AutoTag source="memory" />
        </CnName>
        <CnName compact cn="焰烬曙明完结庆祝" />
    </div>
);
