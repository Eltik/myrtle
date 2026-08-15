import { SearchCommand } from "frontend";

const noop = () => {};

const OPERATORS = [
    { id: "char_4064_mlynar", name: "Młynar", meta: "6★ · Guard" },
    { id: "char_263_skadi", name: "Skadi", meta: "6★ · Guard" },
    { id: "char_180_amgoat", name: "Eyjafjalla", meta: "6★ · Caster" },
    { id: "char_102_texas", name: "Texas", meta: "5★ · Vanguard" },
];

const PageBehind = () => (
    <div className="mx-auto w-full max-w-2xl">
        <div className="flex items-center justify-between border-b pb-3">
            <span className="font-semibold text-lg">Operators</span>
            <span className="text-muted-foreground text-sm">312 operators · EN server</span>
        </div>
        <div className="mt-4 grid grid-cols-4 gap-3">
            {OPERATORS.map((op) => (
                <div className="rounded-lg border bg-card p-3" key={op.id}>
                    <img alt="" className="size-10 rounded-md object-cover" src={`https://api.myrtle.moe/api/avatar/${op.id}`} />
                    <div className="mt-2 truncate font-medium text-sm">{op.name}</div>
                    <div className="text-muted-foreground text-xs">{op.meta}</div>
                </div>
            ))}
        </div>
    </div>
);

/**
 * The site-wide ⌘K palette, opened over the operator index. Pages come from the
 * static registry and render normally; the Operators group is fed by a query, so
 * outside the app it shows that group's inline failure state.
 */
export const OpenWithOperatorsUnavailable = () => (
    <div>
        <PageBehind />
        <SearchCommand onOpenChange={noop} open />
    </div>
);
