import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";

export const Default = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages">Stages</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">1-7</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const OnlyPage = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Settings</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const MonospacedCode = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages">Stages</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages/side/nl">Near Light</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-mono font-medium tracking-tight">S4-1</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const LongTitle = () => (
    <div className="max-w-sm">
        <Breadcrumb>
            <BreadcrumbList className="flex-nowrap">
                <BreadcrumbItem className="shrink-0">
                    <BreadcrumbLink href="/tier-lists">Tier lists</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator className="shrink-0" />
                <BreadcrumbItem className="min-w-0">
                    <BreadcrumbPage className="truncate font-medium">Best 6★ Guards for Integrated Strategies #5</BreadcrumbPage>
                </BreadcrumbItem>
            </BreadcrumbList>
        </Breadcrumb>
    </div>
);
