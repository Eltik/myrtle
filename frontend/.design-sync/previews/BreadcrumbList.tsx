import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";

export const Default = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators?class=caster">Caster</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Eyjafjalla</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const Compact = () => (
    <Breadcrumb>
        <BreadcrumbList className="text-xs">
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools">Tools</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools/birthdays">Birthdays</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">March 2026</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const WrappingPath = () => (
    <div className="max-w-xs">
        <Breadcrumb>
            <BreadcrumbList>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/">Home</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                    <BreadcrumbLink href="/stages">Stages</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                    <BreadcrumbLink href="/stages/side">Side story</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                    <BreadcrumbLink href="/stages/side/nl">Near Light</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                    <BreadcrumbPage className="font-medium">S4-1 — Kazimierz Major</BreadcrumbPage>
                </BreadcrumbItem>
            </BreadcrumbList>
        </Breadcrumb>
    </div>
);
