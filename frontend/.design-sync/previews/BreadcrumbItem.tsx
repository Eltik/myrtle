import { Badge, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";
import { HomeIcon, SwordsIcon } from "lucide-react";

export const Default = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Muelsyse</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const WithIcons = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <HomeIcon className="size-3.5" />
                <BreadcrumbLink href="/">Myrtle</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <SwordsIcon className="size-3.5" />
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Skadi</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const WithTrailingBadge = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Texas the Omertosa</BreadcrumbPage>
                <Badge size="sm" variant="secondary">
                    6★
                </Badge>
                <Badge className="font-mono uppercase" size="sm" variant="info">
                    CN
                </Badge>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);
