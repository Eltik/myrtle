import { Button, PageHeader } from "frontend";

// The page header in the only two shapes a page may take: a title alone, or a
// title with a description. Both carry the same optional breadcrumb, type scale
// (24px, 30px from `sm`) and a trailing actions slot that drops under the title
// when the bar is too narrow.

/** Base design: breadcrumb and title, no description. */
export const Base = () => (
    <div className="w-full p-6">
        <PageHeader breadcrumb={["Tools", "Recruitment"]} breadcrumbLabel="Breadcrumb" title="Recruitment Calculator" />
    </div>
);

/** With-description design and a trailing action. */
export const WithDescription = () => (
    <div className="w-full p-6">
        <PageHeader
            breadcrumb={["Collection", "Operators"]}
            breadcrumbLabel="Breadcrumb"
            title="Operators"
            description="Every operator on the EN server, with stats, skills, modules and outfits. Filter by class, rarity or faction."
            actions={<Button variant="outline">Export</Button>}
        />
    </div>
);

/** A title adornment (a badge) inline after the title, and two actions. */
export const AdornmentAndActions = () => (
    <div className="w-full p-6">
        <PageHeader
            title="Tier Lists"
            titleAdornment={<span className="rounded-full border border-border px-2 py-0.5 font-mono text-[10.5px] text-muted-foreground uppercase tracking-wider">Beta</span>}
            description="Community rankings of operators, enemies, events and more."
            actions={
                <div className="flex gap-2">
                    <Button variant="outline">My lists</Button>
                    <Button>New tier list</Button>
                </div>
            }
        />
    </div>
);

/** No breadcrumb: the bare title. */
export const TitleOnly = () => (
    <div className="w-full p-6">
        <PageHeader title="Settings" />
    </div>
);
