import { Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator } from "frontend";
import { SlashIcon } from "lucide-react";

export const DefaultChevron = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators">Operators</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbLink href="/operators?class=medic">Medic</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator />
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Kal'tsit</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const SlashSeparator = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages">Stages</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator>
                <SlashIcon />
            </BreadcrumbSeparator>
            <BreadcrumbItem>
                <BreadcrumbLink href="/stages/main/8">Chapter 8</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator>
                <SlashIcon />
            </BreadcrumbSeparator>
            <BreadcrumbItem>
                <BreadcrumbPage className="font-mono font-medium">8-16</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);

export const DotSeparator = () => (
    <Breadcrumb>
        <BreadcrumbList>
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools">Tools</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="text-muted-foreground">·</BreadcrumbSeparator>
            <BreadcrumbItem>
                <BreadcrumbLink href="/tools/gacha">Gacha tracker</BreadcrumbLink>
            </BreadcrumbItem>
            <BreadcrumbSeparator className="text-muted-foreground">·</BreadcrumbSeparator>
            <BreadcrumbItem>
                <BreadcrumbPage className="font-medium">Pull history</BreadcrumbPage>
            </BreadcrumbItem>
        </BreadcrumbList>
    </Breadcrumb>
);
