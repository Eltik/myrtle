import { Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";

export const TruncatedPath = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/">Home</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbEllipsis />
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages/side/nl">Near Light</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-mono font-medium">S4-1</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const CollapsedMiddle = () => (
    <Breadcrumb>
        <BreadcrumbList className="text-xs">
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbEllipsis className="text-muted-foreground" />
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Modules</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);
